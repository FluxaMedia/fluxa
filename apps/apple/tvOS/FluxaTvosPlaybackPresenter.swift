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

@_silgen_name("fluxa_streaming_string_free")
private func fluxaStreamingStringFree(_ value: UnsafeMutablePointer<CChar>)

private final class FluxaTvosStreamingAdapter {
    private var serverId: String?

    func prepare(_ url: URL) -> URL {
        let path = url.path.lowercased()
        guard path.hasSuffix(".mkv") || path.hasSuffix(".matroska") else { return url }
        stop()
        let raw = url.absoluteString
        guard let response = raw.withCString({ target in
            "{}".withCString { headers in
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

    func stop() {
        guard let serverId else { return }
        _ = serverId.withCString { fluxaStreamingStopLocalStreamServer($0) }
        self.serverId = nil
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

    func present(url: URL, title: String, resumePosition: Double = 0) {
        guard let presenter = topViewController() else { return }
        let playbackURL = streamingAdapter.prepare(url)
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
                    startPosition: max(0, resumePosition),
                    fallbackURL: playbackURL == url ? nil : url
                )
            )
            player.play()
        }
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
