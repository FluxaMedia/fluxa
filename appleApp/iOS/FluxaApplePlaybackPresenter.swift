import AVFoundation
import AVKit
import FluxaShared
import UIKit

@MainActor
final class FluxaApplePlaybackPresenter: NSObject, UIAdaptivePresentationControllerDelegate {
    static let shared = FluxaApplePlaybackPresenter()

    func present(request: ApplePlaybackRequestSnapshot) {
        guard let presenter = topViewController() else {
            return
        }
        let originalUrl = request.playableUrl
        let requestHeadersJson = request.requestHeadersJson
        let title = request.title
        let resumePositionMs = request.resumePositionMs
        Task {
            let playbackUrl = await Task.detached {
                FluxaAppleStreamingEngine.shared.prepare(
                    url: originalUrl,
                    requestHeadersJson: requestHeadersJson,
                    title: title
                ) ?? URL(string: originalUrl)
            }.value
            guard let playbackUrl else {
                return
            }
            let asset = AVURLAsset(url: playbackUrl)
            let player = AVPlayer(playerItem: AVPlayerItem(asset: asset))
            let controller = AVPlayerViewController()
            controller.player = player
            presenter.present(controller, animated: true) {
                controller.presentationController?.delegate = self
                let position = CMTime(seconds: Double(resumePositionMs) / 1000, preferredTimescale: 600)
                if resumePositionMs > 0 {
                    player.seek(to: position)
                }
                player.play()
            }
        }
    }

    private func topViewController() -> UIViewController? {
        guard let scene = UIApplication.shared.connectedScenes
            .compactMap({ $0 as? UIWindowScene })
            .first(where: { $0.activationState == .foregroundActive }),
              let root = scene.windows.first(where: { $0.isKeyWindow })?.rootViewController else {
            return nil
        }
        var current = root
        while let presented = current.presentedViewController {
            current = presented
        }
        return current
    }

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        FluxaAppleStreamingEngine.shared.stop()
    }

}
