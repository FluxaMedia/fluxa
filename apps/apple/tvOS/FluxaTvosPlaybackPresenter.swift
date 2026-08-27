import FluxaPlayerKit
import Foundation
import UIKit

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

private final class FluxaTvosStreamingAdapter {
    private var serverId: String?
    private var torrentServerRunning = false

    func prepare(_ url: URL, headers: [String: String]) -> URL {
        if isTorrent(url) {
            return prepareTorrent(url, headers: headers)
        }
        let path = url.path.lowercased()
        guard path.hasSuffix(".mkv") || path.hasSuffix(".matroska") else { return url }
        return prepareRemux(url, headers: headers)
    }

    private func prepareRemux(_ url: URL, headers: [String: String]) -> URL {
        stopLocal()
        let raw = url.absoluteString
        let headersJson = (try? JSONSerialization.data(withJSONObject: headers))
            .flatMap { String(data: $0, encoding: .utf8) } ?? "{}"
        guard let response = raw.withCString({ target in
            headersJson.withCString { headers in
                fluxaStreamingStartLocalStreamServer(target, headers, 0)
            }
        }) else { return url }
        defer { fluxaStreamingStringFree(response) }
        guard let data = String(cString: response).data(using: .utf8),
              let payload = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let id = payload["id"] as? String,
              let proxy = payload["url"] as? String,
              let proxyUrl = URL(string: proxy) else {
            return url
        }
        serverId = id
        return proxyUrl.appendingPathComponent("remux")
    }

    private func prepareTorrent(_ url: URL, headers: [String: String]) -> URL {
        stop()
        let cacheDirectory = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)
            .first?.appendingPathComponent("fluxa_torrent_cache", isDirectory: true).path ?? ""
        guard let response = cacheDirectory.withCString({ cache in
            "".withCString { token in
                fluxaStreamingStartTorrentServer(cache, 0, token)
            }
        }) else { return url }
        defer { fluxaStreamingStringFree(response) }
        guard let data = String(cString: response).data(using: .utf8),
              let payload = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let base = payload["url"] as? String,
              var components = URLComponents(string: base) else { return url }
        components.path = components.path.appending("/stream/fname")
        components.queryItems = [
            URLQueryItem(name: "link", value: url.absoluteString),
            URLQueryItem(name: "title", value: url.lastPathComponent)
        ]
        guard let torrentURL = components.url else { return url }
        // Keep the torrent service alive while the local proxy is replaced;
        // prepareRemux() stops only the previous local proxy.
        torrentServerRunning = true
        let prepared = prepareRemux(torrentURL, headers: headers)
        torrentServerRunning = true
        return prepared
    }

    func stop() {
        stopLocal()
        if torrentServerRunning {
            _ = fluxaStreamingStopTorrentServer()
            torrentServerRunning = false
        }
    }

    private func stopLocal() {
        guard let serverId else { return }
        _ = serverId.withCString { fluxaStreamingStopLocalStreamServer($0) }
        self.serverId = nil
    }

    private func isTorrent(_ url: URL) -> Bool {
        let value = url.absoluteString.lowercased()
        return value.hasPrefix("magnet:") ||
            value.hasPrefix("stremio://torrent/") ||
            value.hasSuffix(".torrent")
    }
}

/// tvOS entry point for the same custom transport surface used by iOS.
/// The catalog/detail layer can hand this presenter a resolved stream without
/// ever falling back to AVPlayerViewController's native controls.
@MainActor
final class FluxaTvosPlaybackPresenter: NSObject, UIAdaptivePresentationControllerDelegate {
    static let shared = FluxaTvosPlaybackPresenter()

    private var activePlayer: FluxaPlayer?
    private weak var activeController: FluxaAppleCustomPlayerViewController?
    private let streamingAdapter = FluxaTvosStreamingAdapter()

    func present(options: [FluxaTvosHomeModel.Playback], title: String) {
        guard let presenter = topViewController() else { return }
        guard options.count > 1 else {
            if let option = options.first {
                present(option: option)
            }
            return
        }
        let alert = UIAlertController(title: title, message: nil, preferredStyle: .actionSheet)
        for option in options {
            alert.addAction(UIAlertAction(title: option.streamTitle, style: .default) { [weak self] _ in
                self?.present(option: option)
            })
        }
        alert.addAction(UIAlertAction(title: nil, style: .cancel))
        presenter.present(alert, animated: true)
    }

    func present(
        url: URL,
        title: String,
        headers: [String: String] = [:],
        subtitleUrls: [URL] = [],
        resumePosition: Double = 0
    ) {
        guard let presenter = topViewController() else { return }
        let playbackURL = streamingAdapter.prepare(url, headers: headers)
        let player = FluxaPlayer()
        let controller = FluxaAppleCustomPlayerViewController(player: player, title: title)
        activePlayer = player
        activeController = controller
        presenter.present(controller, animated: true) {
            controller.presentationController?.delegate = self
            player.load(
                FluxaPlaybackItem(
                    url: playbackURL,
                    title: title,
                    headers: headers,
                    startPosition: max(0, resumePosition),
                    subtitleUrls: subtitleUrls
                )
            )
            player.play()
        }
    }

    private func present(option: FluxaTvosHomeModel.Playback) {
        present(
            url: option.url,
            title: option.title,
            headers: option.headers,
            subtitleUrls: option.subtitleUrls
        )
    }

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        activePlayer?.stop()
        streamingAdapter.stop()
        activePlayer = nil
        activeController = nil
    }

    private func topViewController() -> UIViewController? {
        guard let window = UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene })
            .flatMap(\.windows)
            .first(where: { $0.isKeyWindow }),
              let root = window.rootViewController else { return nil }
        var current = root
        while let presented = current.presentedViewController { current = presented }
        return current
    }
}
