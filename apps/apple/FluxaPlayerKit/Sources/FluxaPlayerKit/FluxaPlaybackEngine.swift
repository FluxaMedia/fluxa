import Foundation

@MainActor
protocol FluxaPlaybackEngineDelegate: AnyObject {
    func engine(_ engine: FluxaPlaybackEngine, didUpdate state: FluxaPlaybackState)
    func engine(_ engine: FluxaPlaybackEngine, didUpdate tracks: [FluxaTrack])
}

@MainActor
protocol FluxaPlaybackEngine: AnyObject {
    var delegate: FluxaPlaybackEngineDelegate? { get set }

    func attach(to surface: FluxaPlayerSurfaceView)
    func load(_ item: FluxaPlaybackItem)
    func play()
    func pause()
    func seek(to position: TimeInterval)
    func setRate(_ rate: Float)
    func setVolume(_ volume: Float)
    func selectTrack(_ track: FluxaTrack?, kind: FluxaTrackKind)
    func tearDown()
}
