import Combine
import Foundation

public enum FluxaPlaybackBackend: String, Sendable {
    case avFoundation
    case ffmpeg
}

@MainActor
public final class FluxaPlayer: ObservableObject {
    @Published public private(set) var state = FluxaPlaybackState()
    @Published public private(set) var tracks: [FluxaTrack] = []
    @Published public private(set) var backend: FluxaPlaybackBackend = .avFoundation

    public var onStateChange: ((FluxaPlaybackState) -> Void)?
    public var forcedBackend: FluxaPlaybackBackend?

    public private(set) var item: FluxaPlaybackItem?

    private var engine: FluxaPlaybackEngine?
    private weak var surface: FluxaPlayerSurfaceView?
    #if FLUXA_FFMPEG
    private var fallbackAttempted = false
    #endif
    #if !os(macOS)
    private let audioSession = FluxaAudioSession()
    #endif

    public init() {}

    public func attach(to surface: FluxaPlayerSurfaceView) {
        self.surface = surface
        engine?.attach(to: surface)
    }

    public func load(_ item: FluxaPlaybackItem) {
        self.item = item
        #if FLUXA_FFMPEG
        fallbackAttempted = false
        #endif
        #if !os(macOS)
        audioSession.activate()
        #endif
        let engine = makeEngine(for: item)
        engine.delegate = self
        self.engine = engine
        if let surface {
            engine.attach(to: surface)
        }
        engine.load(item)
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

    public func setRate(_ rate: Float) { engine?.setRate(rate) }

    public func setVolume(_ volume: Float) { engine?.setVolume(volume) }

    public func selectTrack(_ track: FluxaTrack?, kind: FluxaTrackKind) {
        engine?.selectTrack(track, kind: kind)
    }

    public func tracks(of kind: FluxaTrackKind) -> [FluxaTrack] {
        tracks.filter { $0.kind == kind }
    }

    public func stop() {
        engine?.tearDown()
        engine = nil
        item = nil
        #if FLUXA_FFMPEG
        fallbackAttempted = false
        #endif
        surface?.unhost()
        tracks = []
        state = FluxaPlaybackState()
        onStateChange?(state)
        #if !os(macOS)
        audioSession.deactivate()
        #endif
    }

    private func makeEngine(for item: FluxaPlaybackItem) -> FluxaPlaybackEngine {
        #if FLUXA_FFMPEG
        if forcedBackend == .ffmpeg {
            backend = .ffmpeg
            return FluxaFFmpegEngine()
        }
        #endif
        backend = .avFoundation
        return FluxaAVFoundationEngine()
    }

    private func clamp(_ position: TimeInterval) -> TimeInterval {
        guard state.duration > 0 else { return max(0, position) }
        return min(max(0, position), state.duration)
    }
}

extension FluxaPlayer: FluxaPlaybackEngineDelegate {
    func engine(_ engine: FluxaPlaybackEngine, didUpdate state: FluxaPlaybackState) {
        guard engine === self.engine else { return }
        #if FLUXA_FFMPEG
        if case .failed(let failure) = state.phase,
           failure.isRecoverable,
           !fallbackAttempted,
           forcedBackend != .ffmpeg,
           let currentItem = item,
           let fallbackURL = currentItem.fallbackURL,
           fallbackURL != currentItem.url {
            fallbackAttempted = true
            let fallbackItem = FluxaPlaybackItem(
                url: fallbackURL,
                fallbackURL: nil,
                title: currentItem.title,
                headers: currentItem.headers,
                startPosition: max(currentItem.startPosition, state.position),
                subtitleUrls: currentItem.subtitleUrls
            )
            engine.tearDown()
            let fallbackEngine = FluxaFFmpegEngine()
            fallbackEngine.delegate = self
            self.engine = fallbackEngine
            backend = .ffmpeg
            if let surface {
                fallbackEngine.attach(to: surface)
            }
            fallbackEngine.load(fallbackItem)
            fallbackEngine.play()
            self.item = fallbackItem
            return
        }
        #endif
        self.state = state
        onStateChange?(state)
    }

    func engine(_ engine: FluxaPlaybackEngine, didUpdate tracks: [FluxaTrack]) {
        guard engine === self.engine else { return }
        self.tracks = tracks
    }
}
