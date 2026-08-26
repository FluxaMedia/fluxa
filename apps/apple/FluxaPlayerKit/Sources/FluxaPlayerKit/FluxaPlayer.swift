import Combine
import Foundation

public enum FluxaPlaybackBackend: String, Sendable {
    case avFoundation
}

@MainActor
public final class FluxaPlayer: ObservableObject {
    @Published public private(set) var state = FluxaPlaybackState()
    @Published public private(set) var tracks: [FluxaTrack] = []
    @Published public private(set) var backend: FluxaPlaybackBackend = .avFoundation

    public var onStateChange: ((FluxaPlaybackState) -> Void)?

    public private(set) var item: FluxaPlaybackItem?

    private var engine: FluxaPlaybackEngine?
    private weak var surface: FluxaPlayerSurfaceView?
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
        surface?.unhost()
        tracks = []
        state = FluxaPlaybackState()
        onStateChange?(state)
        #if !os(macOS)
        audioSession.deactivate()
        #endif
    }

    private func makeEngine(for item: FluxaPlaybackItem) -> FluxaPlaybackEngine {
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
        self.state = state
        onStateChange?(state)
    }

    func engine(_ engine: FluxaPlaybackEngine, didUpdate tracks: [FluxaTrack]) {
        guard engine === self.engine else { return }
        self.tracks = tracks
    }
}
