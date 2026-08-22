import AVFoundation
import UIKit

/// tvOS entry point for the same custom transport surface used by iOS.
/// The catalog/detail layer can hand this presenter a resolved stream without
/// ever falling back to AVPlayerViewController's native controls.
@MainActor
final class FluxaTvosPlaybackPresenter: NSObject, UIAdaptivePresentationControllerDelegate {
    static let shared = FluxaTvosPlaybackPresenter()

    private var activePlayer: AVPlayer?
    private weak var activeController: FluxaAppleCustomPlayerViewController?
    private let audioSessionCoordinator = FluxaAppleAudioSessionCoordinator()

    func present(url: URL, title: String, resumePosition: Double = 0) {
        guard let presenter = topViewController() else { return }
        audioSessionCoordinator.activate()
        let player = AVPlayer(url: url)
        let controller = FluxaAppleCustomPlayerViewController(player: player, title: title)
        activePlayer = player
        activeController = controller
        presenter.present(controller, animated: true) {
            controller.presentationController?.delegate = self
            if resumePosition > 0 {
                player.seek(to: CMTime(seconds: resumePosition, preferredTimescale: 600))
            }
            player.play()
        }
    }

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        activePlayer?.pause()
        activePlayer = nil
        activeController = nil
        audioSessionCoordinator.deactivate()
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
