import Foundation

public struct FluxaDiscordPresenceSnapshot: Sendable, Equatable {
    public var title: String
    public var episodeLine: String
    public var status: String
    public var position: TimeInterval
    public var duration: TimeInterval
    public var artworkURL: URL?

    public init(
        title: String,
        episodeLine: String = "",
        status: String,
        position: TimeInterval,
        duration: TimeInterval,
        artworkURL: URL? = nil
    ) {
        self.title = title
        self.episodeLine = episodeLine
        self.status = status
        self.position = position
        self.duration = duration
        self.artworkURL = artworkURL
    }
}
