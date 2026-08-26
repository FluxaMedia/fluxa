import FluxaPlayerKit
import UIKit

/// tvOS entry point for the same custom transport surface used by iOS.
/// The catalog/detail layer can hand this presenter a resolved stream without
/// ever falling back to AVPlayerViewController's native controls.
@MainActor
final class FluxaTvosPlaybackPresenter: NSObject, UIAdaptivePresentationControllerDelegate {
    static let shared = FluxaTvosPlaybackPresenter()

    private var activePlayer: FluxaPlayer?
    private weak var activeController: FluxaAppleCustomPlayerViewController?

    func present(url: URL, title: String, resumePosition: Double = 0) {
        guard let presenter = topViewController() else { return }
        let player = FluxaPlayer()
        let controller = FluxaAppleCustomPlayerViewController(player: player, title: title)
        activePlayer = player
        activeController = controller
        presenter.present(controller, animated: true) {
            controller.presentationController?.delegate = self
            player.load(
                FluxaPlaybackItem(url: url, title: title, startPosition: max(0, resumePosition))
            )
            player.play()
        }
    }

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        activePlayer?.stop()
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
