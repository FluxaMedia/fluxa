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

/// Owns the Apple AVPlayer source policy and the lifetime of its local transport.
final class FluxaAppleAVPlayerStreamAdapter: @unchecked Sendable {
    private let lock = NSLock()
    private var localServerId: String?
    private var torrentServerRunning = false
    private var preparationGeneration = 0

    func beginPreparation() -> Int {
        lock.withLock {
            preparationGeneration += 1
            return preparationGeneration
        }
    }

    func invalidate() {
        lock.withLock {
            preparationGeneration += 1
            stopLocked()
        }
    }

    func prepare(url: URL, headers: [String: String], title: String, generation: Int? = nil) -> URL? {
        lock.withLock {
            if let generation, generation != preparationGeneration { return nil }
            stopLocked()
            if isTorrent(url) {
                return startTorrentLocked(link: url.absoluteString, headers: headers, title: title)
            }
            guard requiresRemux(url) else { return url }
            return startLocalProxyLocked(url: url.absoluteString, headers: headers)
                .map { $0.appendingPathComponent("remux") }
        }
    }

    func stop() {
        invalidate()
    }

    private func startLocalProxyLocked(url: String, headers: [String: String]) -> URL? {
        guard let response = withCStrings(url, headersJSON(headers), operation: fluxaStreamingStartLocalStreamServer),
              let server = try? JSONDecoder().decode(LocalServerResponse.self, from: Data(response.utf8)),
              !server.id.isEmpty else {
            return nil
        }
        localServerId = server.id
        guard let proxyURL = URL(string: server.url) else {
            stopLocked()
            return nil
        }
        return proxyURL
    }

    private func startTorrentLocked(link: String, headers: [String: String], title: String) -> URL? {
        let cacheDirectory = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)
            .first?
            .appendingPathComponent("fluxa_torrent_cache", isDirectory: true)
            .path ?? ""
        guard let response = withCStrings(cacheDirectory, "", operation: fluxaStreamingStartTorrentServer),
              let server = try? JSONDecoder().decode(TorrentServerResponse.self, from: Data(response.utf8)),
              var components = URLComponents(string: server.url) else {
            return nil
        }
        torrentServerRunning = true
        components.path = components.path.appending("/stream/fname")
        components.queryItems = [
            URLQueryItem(name: "link", value: link),
            URLQueryItem(name: "title", value: title)
        ]
        guard let torrentURL = components.url,
              let proxyURL = startLocalProxyLocked(url: torrentURL.absoluteString, headers: headers) else {
            stopLocked()
            return nil
        }
        return proxyURL.appendingPathComponent("remux")
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

    private func requiresRemux(_ url: URL) -> Bool {
        let components = URLComponents(url: url, resolvingAgainstBaseURL: false)
        let queryValues = components?.queryItems?.compactMap(\.value) ?? []
        if queryValues.contains(where: {
            let value = $0.lowercased()
            return value.contains(".m3u8") || value.contains("hls")
        }) {
            return false
        }
        let queryCandidates = components?.queryItems?
            .filter { ["filename", "file", "name", "path"].contains($0.name.lowercased()) }
            .compactMap(\.value) ?? []
        let candidates = [url.path] + queryCandidates
        let remuxExtensions = Set(["mkv", "matroska", "webm", "avi", "flv", "wmv", "ogv"])
        if candidates.contains(where: { candidate in
            let pathExtension = URL(string: candidate)?.pathExtension
                ?? URL(fileURLWithPath: candidate).pathExtension
            return remuxExtensions.contains(pathExtension.lowercased())
        }) {
            return true
        }
        let nativeExtensions = Set(["mp4", "m4v", "mov", "m3u8", "m3u", "ts", "mp3", "aac", "ac3", "eac3"])
        if candidates.contains(where: { candidate in
            let pathExtension = URL(string: candidate)?.pathExtension
                ?? URL(fileURLWithPath: candidate).pathExtension
            return nativeExtensions.contains(pathExtension.lowercased())
        }) {
            return false
        }
        // Addon URLs frequently omit the media extension. Adapt unknown HTTP
        // sources so Matroska streams do not bypass the AVPlayer remux path.
        return ["http", "https"].contains(url.scheme?.lowercased())
    }

    private func isTorrent(_ url: URL) -> Bool {
        let value = url.absoluteString.lowercased()
        return value.hasPrefix("magnet:") ||
            value.hasPrefix("stremio://torrent/") ||
            value.hasSuffix(".torrent")
    }

    private func headersJSON(_ headers: [String: String]) -> String {
        guard let data = try? JSONSerialization.data(withJSONObject: headers),
              let value = String(data: data, encoding: .utf8) else { return "{}" }
        return value
    }

    private func withCStrings(
        _ first: String,
        _ second: String,
        operation: (UnsafePointer<CChar>, UnsafePointer<CChar>, Int32) -> UnsafeMutablePointer<CChar>?
    ) -> String? {
        first.withCString { firstPointer in
            second.withCString { secondPointer in
                guard let result = operation(firstPointer, secondPointer, 0) else { return nil }
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
                guard let result = operation(firstPointer, 0, secondPointer) else { return nil }
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
