import AVFoundation
import AVKit
import FluxaShared
import UIKit

@MainActor
final class FluxaApplePlaybackPresenter: NSObject, UIAdaptivePresentationControllerDelegate {
    static let shared = FluxaApplePlaybackPresenter()
    private var activePlayer: AVPlayer?
    private var timeObserver: Any?
    private lazy var stateBridge = NativePlayerStateBridge(
        callbacks: NativePlayerCommandCallbacks(
            setPlaying: { [weak self] playing in
                playing ? self?.activePlayer?.play() : self?.activePlayer?.pause()
            },
            seekTo: { [weak self] positionMs in
                self?.activePlayer?.seek(to: CMTime(seconds: Double(positionMs) / 1000, preferredTimescale: 600))
            },
            setVolume: { [weak self] volume in self?.activePlayer?.volume = volume },
            setSubtitleEnabled: { _ in },
            stop: { [weak self] in self?.activePlayer?.pause() }
        )
    )

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
            self.activePlayer = player
            self.stateBridge.setContent(
                content: PlayerContentUiModel(
                    id: originalUrl,
                    type: "",
                    title: title,
                    subtitle: "",
                    logoUrl: nil,
                    backgroundUrl: nil,
                    streamLabel: title,
                    releaseInfo: nil,
                    runtime: nil
                )
            )
            self.observe(player)
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
        stopObserving()
        stateBridge.stop()
        stateBridge.setContent(content: nil)
        activePlayer = nil
        FluxaAppleStreamingEngine.shared.stop()
    }

    private func observe(_ player: AVPlayer) {
        stopObserving()
        timeObserver = player.addPeriodicTimeObserver(
            forInterval: CMTime(seconds: 0.5, preferredTimescale: 600),
            queue: .main
        ) { [weak self, weak player] time in
            guard let self, let player else {
                return
            }
            let durationSeconds = player.currentItem?.duration.seconds ?? 0
            let bufferedSeconds = player.currentItem?.loadedTimeRanges.last?.timeRangeValue.end.seconds ?? 0
            self.stateBridge.updatePlayback(
                isPlaying: player.timeControlStatus == .playing,
                isBuffering: player.timeControlStatus == .waitingToPlayAtSpecifiedRate,
                positionMs: Int64(max(0, time.seconds) * 1000),
                durationMs: Int64(max(0, durationSeconds.isFinite ? durationSeconds : 0) * 1000),
                bufferedPositionMs: Int64(max(0, bufferedSeconds.isFinite ? bufferedSeconds : 0) * 1000),
                errorKey: player.currentItem?.error == nil ? nil : "error.generic_message"
            )
        }
    }

    private func stopObserving() {
        if let timeObserver, let activePlayer {
            activePlayer.removeTimeObserver(timeObserver)
        }
        timeObserver = nil
    }

}
