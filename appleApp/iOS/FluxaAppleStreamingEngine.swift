import Foundation

@_silgen_name("fluxa_streaming_start_local_stream_server")
private func fluxaStreamingStartLocalStreamServer(
    _ targetUrl: UnsafePointer<CChar>,
    _ headersJson: UnsafePointer<CChar>,
    _ preferredPort: Int32
) -> UnsafeMutablePointer<CChar>?

@_silgen_name("fluxa_streaming_stop_local_stream_server")
private func fluxaStreamingStopLocalStreamServer(_ serverId: UnsafePointer<CChar>) -> Bool

@_silgen_name("fluxa_streaming_start_torrent_server")
private func fluxaStreamingStartTorrentServer(
    _ cacheDirectory: UnsafePointer<CChar>,
    _ preferredPort: Int32,
    _ accessToken: UnsafePointer<CChar>
) -> UnsafeMutablePointer<CChar>?

@_silgen_name("fluxa_streaming_stop_torrent_server")
private func fluxaStreamingStopTorrentServer() -> Bool

@_silgen_name("fluxa_streaming_string_free")
private func fluxaStreamingStringFree(_ value: UnsafeMutablePointer<CChar>)

final class FluxaAppleStreamingEngine: @unchecked Sendable {
    static let shared = FluxaAppleStreamingEngine()

    private let lock = NSLock()
    private var localServerId: String?
    private var torrentServerRunning = false

    func prepare(url: String, requestHeadersJson: String, title: String) -> URL? {
        lock.withLock {
            stopLocked()
            if isTorrent(url) {
                return startTorrentLocked(link: url, title: title)
            }
            return startLocalProxyLocked(url: url, requestHeadersJson: requestHeadersJson)
        }
    }

    func stop() {
        lock.withLock {
            stopLocked()
        }
    }

    private func startLocalProxyLocked(url: String, requestHeadersJson: String) -> URL? {
        guard let response = withCStrings(url, requestHeadersJson, operation: fluxaStreamingStartLocalStreamServer),
              let server = try? JSONDecoder().decode(LocalServerResponse.self, from: Data(response.utf8)),
              let proxyUrl = URL(string: server.url) else {
            return nil
        }
        localServerId = server.id
        return proxyUrl
    }

    private func startTorrentLocked(link: String, title: String) -> URL? {
        let cacheDirectory = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)
            .first?
            .appendingPathComponent("fluxa_torrent_cache", isDirectory: true)
            .path ?? ""
        guard let response = withCStrings(cacheDirectory, "", operation: fluxaStreamingStartTorrentServer),
              let server = try? JSONDecoder().decode(TorrentServerResponse.self, from: Data(response.utf8)),
              var components = URLComponents(string: server.url) else {
            return nil
        }
        components.path = components.path.appending("/stream/fname")
        components.queryItems = [
            URLQueryItem(name: "link", value: link),
            URLQueryItem(name: "title", value: title)
        ]
        torrentServerRunning = true
        return components.url
    }

    private func stopLocked() {
        if let localServerId {
            localServerId.withCString { _ = fluxaStreamingStopLocalStreamServer($0) }
            self.localServerId = nil
        }
        if torrentServerRunning {
            _ = fluxaStreamingStopTorrentServer()
            torrentServerRunning = false
        }
    }

    private func isTorrent(_ url: String) -> Bool {
        let normalized = url.lowercased()
        return normalized.hasPrefix("magnet:") ||
            normalized.hasPrefix("stremio://torrent/") ||
            normalized.hasSuffix(".torrent")
    }

    private func withCStrings(
        _ first: String,
        _ second: String,
        operation: (UnsafePointer<CChar>, UnsafePointer<CChar>, Int32) -> UnsafeMutablePointer<CChar>?
    ) -> String? {
        first.withCString { firstPointer in
            second.withCString { secondPointer in
                guard let result = operation(firstPointer, secondPointer, 0) else {
                    return nil
                }
                defer { fluxaStreamingStringFree(result) }
                return String(cString: result)
            }
        }
    }

    private func withCStrings(
        _ first: String,
        _ second: String,
        operation: (UnsafePointer<CChar>, Int32, UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
    ) -> String? {
        first.withCString { firstPointer in
            second.withCString { secondPointer in
                guard let result = operation(firstPointer, 0, secondPointer) else {
                    return nil
                }
                defer { fluxaStreamingStringFree(result) }
                return String(cString: result)
            }
        }
    }

    private struct LocalServerResponse: Decodable {
        let id: String
        let url: String
    }

    private struct TorrentServerResponse: Decodable {
        let url: String
    }
}
