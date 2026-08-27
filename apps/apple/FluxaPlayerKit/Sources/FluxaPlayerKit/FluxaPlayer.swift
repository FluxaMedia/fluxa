import Combine
import Foundation

@MainActor
public final class FluxaPlayer: ObservableObject {
    @Published public private(set) var state = FluxaPlaybackState()
    @Published public private(set) var tracks: [FluxaTrack] = []
    @Published public private(set) var subtitleText: String?
    public var onStateChange: ((FluxaPlaybackState) -> Void)?

    public private(set) var item: FluxaPlaybackItem?

    private var engine: FluxaPlaybackEngine?
    private var subtitleCues: [FluxaSubtitleCue] = []
    private var subtitleTask: Task<Void, Never>?
    private var subtitleLoadGeneration = 0
    private var externalSubtitleURLs: [URL] = []
    private var subtitleHeaders: [String: String] = [:]
    private var embeddedSubtitlesSuppressed = false
    private var shouldResumeAfterInterruption = false
    private weak var surface: FluxaPlayerSurfaceView?
    #if !os(macOS)
    private let audioSession = FluxaAudioSession()
    #endif
    #if canImport(MediaPlayer)
    private let nowPlaying = FluxaNowPlaying()
    #endif

    public init() {
        #if !os(macOS)
        audioSession.onInterruptionBegan = { [weak self] in
            self?.shouldResumeAfterInterruption = self?.state.isPlaying ?? false
        }
        audioSession.onInterruptionEnded = { [weak self] shouldResume in
            guard let self, shouldResume, self.shouldResumeAfterInterruption else { return }
            self.shouldResumeAfterInterruption = false
            self.play()
        }
        #endif
    }

    public func attach(to surface: FluxaPlayerSurfaceView) {
        self.surface = surface
        engine?.attach(to: surface)
    }

    public func load(_ item: FluxaPlaybackItem) {
        engine?.tearDown()
        subtitleTask?.cancel()
        subtitleTask = nil
        subtitleLoadGeneration += 1
        subtitleCues = []
        subtitleText = nil
        externalSubtitleURLs = item.subtitleUrls
        subtitleHeaders = item.headers
        embeddedSubtitlesSuppressed = false
        shouldResumeAfterInterruption = false
        self.item = item
        #if canImport(MediaPlayer)
        nowPlaying.begin(title: item.title)
        #endif
        #if !os(macOS)
        audioSession.activate()
        #endif
        let engine = makeEngine(for: item)
        engine.delegate = self
        self.engine = engine
        tracks = externalSubtitleTracks()
        if let surface {
            engine.attach(to: surface)
        }
        engine.load(item)
        if !externalSubtitleURLs.isEmpty {
            engine.selectTrack(nil, kind: .subtitle)
        }
        loadExternalSubtitle(at: 0)
    }

    public func play() { engine?.play() }

    public func pause() { engine?.pause() }

    public func togglePlayback() {
        state.isPlaying ? pause() : play()
    }

    public func seek(to position: TimeInterval) {
        engine?.seek(to: clamp(position))
    }

    public func skip(by seconds: TimeInterval) {
        seek(to: state.position + seconds)
    }

    public func setRate(_ rate: Float) {
        engine?.setRate(rate.isFinite ? max(0, rate) : 1)
    }

    public func setVolume(_ volume: Float) { engine?.setVolume(volume) }

    public func selectTrack(_ track: FluxaTrack?, kind: FluxaTrackKind) {
        if kind == .subtitle, let track,
           let index = externalSubtitleIndex(for: track.id) {
            // Do not let an embedded legible track render beneath the custom
            // external-subtitle overlay.
            engine?.selectTrack(nil, kind: .subtitle)
            embeddedSubtitlesSuppressed = true
            loadExternalSubtitle(at: index)
            return
        }
        if kind == .subtitle {
            embeddedSubtitlesSuppressed = false
            subtitleTask?.cancel()
            subtitleTask = nil
            subtitleLoadGeneration += 1
            subtitleCues = []
            subtitleText = nil
        }
        engine?.selectTrack(track, kind: kind)
    }

    public func tracks(of kind: FluxaTrackKind) -> [FluxaTrack] {
        tracks.filter { $0.kind == kind }
    }

    public func stop() {
        subtitleTask?.cancel()
        subtitleTask = nil
        subtitleLoadGeneration += 1
        subtitleCues = []
        subtitleText = nil
        externalSubtitleURLs = []
        subtitleHeaders = [:]
        embeddedSubtitlesSuppressed = false
        shouldResumeAfterInterruption = false
        engine?.tearDown()
        engine = nil
        item = nil
        surface?.unhost()
        tracks = []
        state = FluxaPlaybackState()
        #if canImport(MediaPlayer)
        nowPlaying.clear()
        #endif
        onStateChange?(state)
        #if !os(macOS)
        audioSession.deactivate()
        #endif
    }

    private func makeEngine(for item: FluxaPlaybackItem) -> FluxaPlaybackEngine {
        return FluxaAVFoundationEngine()
    }

    private func clamp(_ position: TimeInterval) -> TimeInterval {
        guard position.isFinite else { return 0 }
        guard state.duration > 0 else { return max(0, position) }
        return min(max(0, position), state.duration)
    }
}

extension FluxaPlayer: FluxaPlaybackEngineDelegate {
    func engine(_ engine: FluxaPlaybackEngine, didUpdate state: FluxaPlaybackState) {
        guard engine === self.engine else { return }
        if !externalSubtitleURLs.isEmpty,
           !embeddedSubtitlesSuppressed,
           state.phase != .loading {
            engine.selectTrack(nil, kind: .subtitle)
            embeddedSubtitlesSuppressed = true
        }
        self.state = state
        updateSubtitleText(at: state.position)
        #if canImport(MediaPlayer)
        nowPlaying.update(state)
        #endif
        onStateChange?(state)
    }

    private func updateSubtitleText(at position: TimeInterval) {
        subtitleText = subtitleCues.first(where: { position >= $0.start && position < $0.end })?.text
    }

    private func loadExternalSubtitle(at index: Int) {
        guard externalSubtitleURLs.indices.contains(index) else { return }
        subtitleTask?.cancel()
        subtitleLoadGeneration += 1
        let generation = subtitleLoadGeneration
        subtitleCues = []
        subtitleText = nil
        let subtitleURL = externalSubtitleURLs[index]
        var request = URLRequest(url: subtitleURL)
        for (field, value) in subtitleHeaders {
            request.setValue(value, forHTTPHeaderField: field)
        }
        subtitleTask = Task { [weak self] in
            guard let (data, _) = try? await URLSession.shared.data(for: request) else { return }
            guard !Task.isCancelled else { return }
            let cues = FluxaSubtitleParser.parse(data)
            guard !cues.isEmpty else { return }
            guard let self, self.subtitleLoadGeneration == generation else { return }
            self.subtitleCues = cues
            self.updateSubtitleText(at: self.state.position)
        }
    }

    private func externalSubtitleIndex(for id: String) -> Int? {
        guard id.hasPrefix("external.subtitle.") else { return nil }
        return Int(id.dropFirst("external.subtitle.".count))
    }

    private func externalSubtitleTracks() -> [FluxaTrack] {
        externalSubtitleURLs.enumerated().map { index, url in
            FluxaTrack(
                id: "external.subtitle.\(index)",
                kind: .subtitle,
                label: url.lastPathComponent.isEmpty ? url.absoluteString : url.lastPathComponent,
                languageCode: nil
            )
        }
    }

    func engine(_ engine: FluxaPlaybackEngine, didUpdate tracks: [FluxaTrack]) {
        guard engine === self.engine else { return }
        self.tracks = tracks + externalSubtitleTracks()
    }
}
