import Foundation

public enum FluxaPlaybackPhase: Sendable, Equatable {
    case idle
    case loading
    case playing
    case paused
    case ended
    case failed(FluxaPlaybackFailure)
}

public struct FluxaPlaybackFailure: Sendable, Equatable {
    public var reason: String
    public var isRecoverable: Bool

    public init(reason: String, isRecoverable: Bool) {
        self.reason = reason
        self.isRecoverable = isRecoverable
    }
}

public struct FluxaPlaybackState: Sendable, Equatable {
    public var phase: FluxaPlaybackPhase = .idle
    public var position: TimeInterval = 0
    public var duration: TimeInterval = 0
    public var buffered: TimeInterval = 0
    public var rate: Float = 1
    public var isBuffering = false
    public var isSeekable = true

    public var isPlaying: Bool { phase == .playing }

    public var failure: FluxaPlaybackFailure? {
        if case .failed(let failure) = phase { return failure }
        return nil
    }

    public init() {}
}
