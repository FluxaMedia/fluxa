import AVFoundation

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
    private var loadedItem: FluxaPlaybackItem?
    private var startupTimeoutTask: Task<Void, Never>?

    private static let startupTimeoutNanoseconds: UInt64 = 10_000_000_000

    override init() {
        super.init()
        player.automaticallyWaitsToMinimizeStalling = true
        playerLayer.player = player
        playerLayer.videoGravity = .resizeAspect
    }

    func attach(to surface: FluxaPlayerSurfaceView) {
        surface.host(playerLayer)
    }

    func load(_ item: FluxaPlaybackItem) {
        detachItemObservers()
        startupTimeoutTask?.cancel()
        loadedItem = item
        var options: [String: Any] = [:]
        if !item.headers.isEmpty {
            options["AVURLAssetHTTPHeaderFieldsKey"] = item.headers
        }
        let asset = AVURLAsset(url: item.url, options: options)
        let playerItem = AVPlayerItem(asset: asset)
        pendingStartPosition = item.startPosition
        tracks = []
        trackOptions = [:]
        state = FluxaPlaybackState()
        state.phase = .loading
        state.isBuffering = true
        publishState()
        publishTracks()
        player.replaceCurrentItem(with: playerItem)
        attachItemObservers(playerItem)
        attachTimeObserver()
        startupTimeoutTask = Task { @MainActor [weak self] in
            do {
                try await Task.sleep(nanoseconds: Self.startupTimeoutNanoseconds)
            } catch {
                return
            }
            guard let self, !Task.isCancelled, self.state.phase == .loading else { return }
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
        if let item = loadedItem, item.url.path.hasSuffix("/remux") {
            let wasPlaying = player.timeControlStatus != .paused
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
                load(restarted)
                if wasPlaying {
                    play()
                }
            }
            return
        }
        let time = CMTime(seconds: max(0, position), preferredTimescale: 600)
        player.seek(to: time, toleranceBefore: .zero, toleranceAfter: .zero)
        state.position = max(0, position)
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
            item.select(nil, in: group)
            return
        }
        item.select(trackOptions[track.id], in: group)
    }

    func tearDown() {
        detachItemObservers()
        startupTimeoutTask?.cancel()
        startupTimeoutTask = nil
        loadedItem = nil
        player.pause()
        player.replaceCurrentItem(with: nil)
        playerLayer.player = nil
        playerLayer.removeFromSuperlayer()
    }

    private func attachTimeObserver() {
        guard timeObserver == nil else { return }
        timeObserver = player.addPeriodicTimeObserver(
            forInterval: CMTime(seconds: 0.25, preferredTimescale: 600),
            queue: .main
        ) { [weak self] time in
            MainActor.assumeIsolated {
                self?.handleTick(time)
            }
        }
    }

    private func attachItemObservers(_ item: AVPlayerItem) {
        observations.append(item.observe(\.status, options: [.initial, .new]) { [weak self] item, _ in
            MainActor.assumeIsolated { self?.handleStatus(item) }
        })
        observations.append(player.observe(\.timeControlStatus, options: [.initial, .new]) { [weak self] _, _ in
            MainActor.assumeIsolated { self?.handleTransportChange() }
        })
        observations.append(item.observe(\.isPlaybackLikelyToKeepUp, options: [.new]) { [weak self] _, _ in
            MainActor.assumeIsolated { self?.handleTransportChange() }
        })
        endObserver = NotificationCenter.default.addObserver(
            forName: .AVPlayerItemDidPlayToEndTime,
            object: item,
            queue: .main
        ) { [weak self] _ in
            MainActor.assumeIsolated {
                guard let self else { return }
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
            if pendingStartPosition > 0 {
                let target = pendingStartPosition
                pendingStartPosition = 0
                player.seek(to: CMTime(seconds: target, preferredTimescale: 600))
                state.position = target
            }
            state.duration = finiteSeconds(item.duration)
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
            state.phase = .failed(FluxaPlaybackFailure(reason: reason, isRecoverable: true))
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
        state.position = max(0, finiteSeconds(time))
        state.duration = finiteSeconds(item.duration)
        state.buffered = finiteSeconds(item.loadedTimeRanges.last?.timeRangeValue.end ?? .zero)
        publishState()
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
        publishTracks()
    }

    private func mediaSelectionGroup(for kind: FluxaTrackKind, in asset: AVAsset) -> AVMediaSelectionGroup? {
        let characteristic: AVMediaCharacteristic = kind == .audio ? .audible : .legible
        return asset.mediaSelectionGroup(forMediaCharacteristic: characteristic)
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
