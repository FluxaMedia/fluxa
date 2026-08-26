import Foundation

public struct FluxaPlaybackItem: Sendable, Equatable {
    public var url: URL
    /// Original source for a decoder fallback when `url` is an adapted local
    /// stream (for example MKV -> fMP4). Kept separate so AVPlayer can use the
    /// Apple-friendly URL while FFmpeg can reopen the real source.
    public var fallbackURL: URL?
    public var title: String
    public var headers: [String: String]
    public var startPosition: TimeInterval
    public var subtitleUrls: [URL]

    public init(
        url: URL,
        fallbackURL: URL? = nil,
        title: String = "",
        headers: [String: String] = [:],
        startPosition: TimeInterval = 0,
        subtitleUrls: [URL] = []
    ) {
        self.url = url
        self.fallbackURL = fallbackURL
        self.title = title
        self.headers = headers
        self.startPosition = startPosition
        self.subtitleUrls = subtitleUrls
    }
}
