import FluxaPlayerKit
import Foundation
import UIKit

/// tvOS entry point for the same custom transport surface used by iOS.
/// The catalog/detail layer can hand this presenter a resolved stream without
/// ever falling back to AVPlayerViewController's native controls.
@MainActor
final class FluxaTvosPlaybackPresenter: NSObject, UIAdaptivePresentationControllerDelegate {
    static let shared = FluxaTvosPlaybackPresenter()

    private var activePlayer: FluxaPlayer?
    private weak var activeController: FluxaAppleCustomPlayerViewController?
    private let streamingAdapter = FluxaAppleAVPlayerStreamAdapter()

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
        guard let playbackURL = streamingAdapter.prepare(url: url, headers: headers, title: title) else { return }
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
