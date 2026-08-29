import Foundation

public struct FluxaPlaybackItem: Sendable, Equatable {
    public var url: URL
    public var title: String
    public var headers: [String: String]
    public var startPosition: TimeInterval
    public var subtitleUrls: [URL]
    public var presenceTitle: String?
    public var presenceEpisodeLine: String?
    public var presenceArtworkURL: URL?

    public init(
        url: URL,
        title: String = "",
        headers: [String: String] = [:],
        startPosition: TimeInterval = 0,
        subtitleUrls: [URL] = [],
        presenceTitle: String? = nil,
        presenceEpisodeLine: String? = nil,
        presenceArtworkURL: URL? = nil
    ) {
        self.url = url
        self.title = title
        self.headers = headers
        self.startPosition = startPosition
        self.subtitleUrls = subtitleUrls
        self.presenceTitle = presenceTitle
        self.presenceEpisodeLine = presenceEpisodeLine
        self.presenceArtworkURL = presenceArtworkURL
    }
}
