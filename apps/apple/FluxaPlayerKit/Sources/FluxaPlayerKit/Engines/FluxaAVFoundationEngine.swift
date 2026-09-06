import AVFoundation

#if os(iOS)
import AVKit
#endif

@MainActor
final class FluxaAVFoundationEngine: NSObject, FluxaPlaybackEngine {
    weak var delegate: FluxaPlaybackEngineDelegate?

    private let player = AVPlayer()
    private let playerLayer = AVPlayerLayer()
    private var state = FluxaPlaybackState()
    private var tracks: [FluxaTrack] = []
    private var trackOptions: [String: AVMediaSelectionOption] = [:]
    private var timeObserver: Any?
    private var observations: [NSKeyValueObservation] = []
    private var endObserver: NSObjectProtocol?
    private var pendingStartPosition: TimeInterval = 0
    private var timelineOffset: TimeInterval = 0
    private var loadedItem: FluxaPlaybackItem?
    private var startupTimeoutTask: Task<Void, Never>?
    private var pendingSeekPosition: TimeInterval?
    private var seekGeneration = 0
    private var itemGeneration = 0
    private var selectedAudioTrackID: String?
    private var selectedSubtitleTrackID: String?

    private static let startupTimeoutNanoseconds: UInt64 = 10_000_000_000

    override init() {
        super.init()
        player.automaticallyWaitsToMinimizeStalling = true
        player.allowsExternalPlayback = true
        #if !os(macOS)
        player.usesExternalPlaybackWhileExternalScreenIsActive = true
        #endif
        playerLayer.player = player
        playerLayer.videoGravity = .resizeAspect
    }

    func attach(to surface: FluxaPlayerSurfaceView) {
        surface.host(playerLayer)
    }

    func load(_ item: FluxaPlaybackItem) {
        load(item, preservingTrackSelection: false)
    }

    private func load(_ item: FluxaPlaybackItem, preservingTrackSelection: Bool) {
        detachItemObservers()
        startupTimeoutTask?.cancel()
        if !preservingTrackSelection {
            selectedAudioTrackID = nil
            selectedSubtitleTrackID = nil
        }
        itemGeneration += 1
        let generation = itemGeneration
        var effectiveItem = item
        if isRemuxURL(item.url), item.startPosition > 0 {
            var components = URLComponents(url: item.url, resolvingAgainstBaseURL: false)
            var queryItems = components?.queryItems ?? []
            queryItems.removeAll { $0.name == "start" }
            queryItems.append(URLQueryItem(name: "start", value: String(item.startPosition)))
            components?.queryItems = queryItems
            if let url = components?.url {
                effectiveItem.url = url
            }
        }
        loadedItem = effectiveItem
        timelineOffset = remuxStart(from: effectiveItem.url)
        pendingSeekPosition = nil
        seekGeneration += 1
        var options: [String: Any] = [:]
        if !effectiveItem.headers.isEmpty {
            options["AVURLAssetHTTPHeaderFieldsKey"] = effectiveItem.headers
        }
        let asset = AVURLAsset(url: effectiveItem.url, options: options)
        let playerItem = AVPlayerItem(asset: asset)
        pendingStartPosition = timelineOffset > 0 ? 0 : effectiveItem.startPosition
        tracks = []
        trackOptions = [:]
        state = FluxaPlaybackState()
        state.phase = .loading
        state.isBuffering = true
        publishState()
        publishTracks()
        player.replaceCurrentItem(with: playerItem)
        attachItemObservers(playerItem)
        attachTimeObserver(for: generation)
        startupTimeoutTask = Task { @MainActor [weak self] in
            do {
                try await Task.sleep(nanoseconds: Self.startupTimeoutNanoseconds)
            } catch {
                return
            }
            guard let self,
                  !Task.isCancelled,
                  self.itemGeneration == generation,
                  self.state.phase == .loading else { return }
            self.state.phase = .failed(
                FluxaPlaybackFailure(
                    reason: "Playback did not become ready in time",
                    isRecoverable: true
                )
            )
            self.state.isBuffering = false
            self.publishState()
        }
    }

    func play() {
        guard player.currentItem != nil else { return }
        player.play()
        player.rate = state.rate
    }

    func pause() {
        player.pause()
    }

    func seek(to position: TimeInterval) {
        guard player.currentItem != nil else { return }
        if let item = loadedItem, isRemuxURL(item.url) {
            let wasPlaying = player.timeControlStatus != .paused
            let playbackRate = state.rate
            var components = URLComponents(url: item.url, resolvingAgainstBaseURL: false)
            var queryItems = components?.queryItems ?? []
            queryItems.removeAll { $0.name == "start" }
            queryItems.append(URLQueryItem(
                name: "start",
                value: String(max(0, position))
            ))
            components?.queryItems = queryItems
            if let url = components?.url {
                var restarted = item
                restarted.url = url
                restarted.startPosition = max(0, position)
                load(restarted, preservingTrackSelection: true)
                state.rate = playbackRate
                if wasPlaying {
                    play()
                } else {
                    publishState()
                }
            }
            return
        }
        let target = max(0, position)
        seekPlayer(to: target)
        state.position = target
        publishState()
    }

    func setRate(_ rate: Float) {
        state.rate = rate
        if player.timeControlStatus != .paused {
            player.rate = rate
        }
        publishState()
    }

    func setVolume(_ volume: Float) {
        player.volume = volume
    }

    func selectTrack(_ track: FluxaTrack?, kind: FluxaTrackKind) {
        guard let item = player.currentItem,
              let group = mediaSelectionGroup(for: kind, in: item.asset) else { return }
        guard let track else {
            if kind == .audio {
                selectedAudioTrackID = nil
            } else {
                selectedSubtitleTrackID = nil
            }
            item.select(nil, in: group)
            return
        }
        guard let option = trackOptions[track.id] else { return }
        if kind == .audio {
            selectedAudioTrackID = track.id
        } else {
            selectedSubtitleTrackID = track.id
        }
        item.select(option, in: group)
    }

    #if os(iOS)
    func makePictureInPictureController() -> AVPictureInPictureController? {
        guard AVPictureInPictureController.isPictureInPictureSupported() else { return nil }
        return AVPictureInPictureController(playerLayer: playerLayer)
    }
    #endif

    func tearDown() {
        detachItemObservers()
        startupTimeoutTask?.cancel()
        startupTimeoutTask = nil
        loadedItem = nil
        timelineOffset = 0
        pendingSeekPosition = nil
        seekGeneration += 1
        itemGeneration += 1
        player.pause()
        player.replaceCurrentItem(with: nil)
        playerLayer.player = nil
        playerLayer.removeFromSuperlayer()
    }

    private func attachTimeObserver(for generation: Int) {
        guard timeObserver == nil else { return }
        timeObserver = player.addPeriodicTimeObserver(
            forInterval: CMTime(seconds: 0.25, preferredTimescale: 600),
            queue: .main
        ) { [weak self] time in
            Task { @MainActor [weak self] in
                guard let self, self.itemGeneration == generation else { return }
                self.handleTick(time)
            }
        }
    }

    private func attachItemObservers(_ item: AVPlayerItem) {
        observations.append(item.observe(\.status, options: [.initial, .new]) { [weak self] item, _ in
            Task { @MainActor [weak self] in
                guard let self, self.player.currentItem === item else { return }
                self.handleStatus(item)
            }
        })
        observations.append(player.observe(\.timeControlStatus, options: [.initial, .new]) { [weak self] _, _ in
            Task { @MainActor [weak self] in
                guard let self, self.player.currentItem === item else { return }
                self.handleTransportChange()
            }
        })
        observations.append(item.observe(\.isPlaybackLikelyToKeepUp, options: [.new]) { [weak self] _, _ in
            Task { @MainActor [weak self] in
                guard let self, self.player.currentItem === item else { return }
                self.handleTransportChange()
            }
        })
        endObserver = NotificationCenter.default.addObserver(
            forName: .AVPlayerItemDidPlayToEndTime,
            object: item,
            queue: .main
        ) { [weak self] _ in
            Task { @MainActor [weak self] in
                guard let self, self.player.currentItem === item else { return }
                self.state.phase = .ended
                self.state.isBuffering = false
                self.publishState()
            }
        }
    }

    private func detachItemObservers() {
        if let timeObserver {
            player.removeTimeObserver(timeObserver)
            self.timeObserver = nil
        }
        observations.forEach { $0.invalidate() }
        observations.removeAll()
        if let endObserver {
            NotificationCenter.default.removeObserver(endObserver)
            self.endObserver = nil
        }
    }

    private func handleStatus(_ item: AVPlayerItem) {
        switch item.status {
        case .readyToPlay:
            let initialPosition: TimeInterval
            if pendingStartPosition > 0 {
                let target = pendingStartPosition
                pendingStartPosition = 0
                seekPlayer(to: target)
                initialPosition = target
            } else {
                initialPosition = timelineOffset
            }
            state.duration = timelineOffset + finiteSeconds(item.duration)
            state.position = initialPosition
            state.isSeekable = !item.seekableTimeRanges.isEmpty || state.duration > 0
            loadTracks(from: item)
            if player.timeControlStatus == .paused {
                state.phase = .paused
                state.isBuffering = false
            }
            handleTransportChange()
        case .failed:
            startupTimeoutTask?.cancel()
            startupTimeoutTask = nil
            let reason = item.error?.localizedDescription ?? "Unknown playback error"
            // AVPlayer has rejected this source. The Apple surface has no
            // software-video fallback, so callers must choose another source
            // or report a terminal playback failure.
            state.phase = .failed(FluxaPlaybackFailure(reason: reason, isRecoverable: false))
            state.isBuffering = false
            publishState()
        default:
            break
        }
    }

    private func handleTransportChange() {
        if case .failed = state.phase { return }
        switch player.timeControlStatus {
        case .playing:
            startupTimeoutTask?.cancel()
            startupTimeoutTask = nil
            state.phase = .playing
            state.isBuffering = false
        case .waitingToPlayAtSpecifiedRate:
            state.isBuffering = true
        case .paused:
            if state.phase != .ended && state.phase != .loading {
                state.phase = .paused
            }
            state.isBuffering = false
        @unknown default:
            break
        }
        publishState()
    }

    private func handleTick(_ time: CMTime) {
        guard let item = player.currentItem else { return }
        if let pendingSeekPosition {
            state.position = pendingSeekPosition
        } else {
            state.position = timelineOffset + finiteSeconds(time)
        }
        state.duration = timelineOffset + finiteSeconds(item.duration)
        state.buffered = timelineOffset + finiteSeconds(item.loadedTimeRanges.last?.timeRangeValue.end ?? .zero)
        publishState()
    }

    private func isRemuxURL(_ url: URL) -> Bool {
        url.path.hasSuffix("/remux")
    }

    private func remuxStart(from url: URL) -> TimeInterval {
        guard isRemuxURL(url),
              let start = URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems?
                .first(where: { $0.name == "start" })?.value,
              let seconds = Double(start), seconds.isFinite else {
            return 0
        }
        return max(0, seconds)
    }

    private func loadTracks(from item: AVPlayerItem) {
        let asset = item.asset
        var collected: [FluxaTrack] = []
        var options: [String: AVMediaSelectionOption] = [:]
        for kind in [FluxaTrackKind.audio, .subtitle] {
            guard let group = mediaSelectionGroup(for: kind, in: asset) else { continue }
            for (index, option) in group.options.enumerated() {
                let id = "\(kind.rawValue).\(index)"
                options[id] = option
                collected.append(
                    FluxaTrack(
                        id: id,
                        kind: kind,
                        label: option.displayName,
                        languageCode: option.extendedLanguageTag,
                        isForced: option.hasMediaCharacteristic(.containsOnlyForcedSubtitles)
                    )
                )
            }
        }
        tracks = collected
        trackOptions = options
        if let selectedAudioTrackID, let option = options[selectedAudioTrackID],
           let group = mediaSelectionGroup(for: .audio, in: asset) {
            item.select(option, in: group)
        }
        if let selectedSubtitleTrackID, let option = options[selectedSubtitleTrackID],
           let group = mediaSelectionGroup(for: .subtitle, in: asset) {
            item.select(option, in: group)
        }
        publishTracks()
    }

    private func mediaSelectionGroup(for kind: FluxaTrackKind, in asset: AVAsset) -> AVMediaSelectionGroup? {
        let characteristic: AVMediaCharacteristic = kind == .audio ? .audible : .legible
        return asset.mediaSelectionGroup(forMediaCharacteristic: characteristic)
    }

    private func seekPlayer(to target: TimeInterval) {
        seekGeneration += 1
        let generation = seekGeneration
        pendingSeekPosition = target
        player.cancelPendingSeeks()
        player.seek(
            to: CMTime(seconds: target, preferredTimescale: 600),
            toleranceBefore: .zero,
            toleranceAfter: .zero
        ) { [weak self] _ in
            Task { @MainActor [weak self] in
                guard let self, self.seekGeneration == generation else { return }
                self.pendingSeekPosition = nil
                self.state.position = self.timelineOffset + self.finiteSeconds(self.player.currentTime())
                self.publishState()
            }
        }
    }

    private func finiteSeconds(_ time: CMTime) -> TimeInterval {
        let seconds = time.seconds
        return seconds.isFinite ? max(0, seconds) : 0
    }

    private func publishState() {
        delegate?.engine(self, didUpdate: state)
    }

    private func publishTracks() {
        delegate?.engine(self, didUpdate: tracks)
    }
}
