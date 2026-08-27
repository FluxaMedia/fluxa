import Foundation
import FluxaPlayerKit
import FluxaShared
import UIKit

@MainActor
final class FluxaApplePlaybackPresenter: NSObject, UIAdaptivePresentationControllerDelegate {
    static let shared = FluxaApplePlaybackPresenter()

    private struct ExternalPlaybackContext {
        let originalUrl: String
        let title: String
        let resumePositionMs: Int64
    }

    private var activePlayer: FluxaPlayer?
    private var externalPlaybackContext: ExternalPlaybackContext?
    private var externalPlaybackRequest: ApplePlaybackRequestSnapshot?
    private weak var activePlayerController: FluxaAppleCustomPlayerViewController?
    private var watchState: AppleWatchTogetherStateSnapshot?
    private var pendingWatchRoomPresentation = false
    private lazy var stateBridge = NativePlayerStateBridge(
        callbacks: NativePlayerCommandCallbacks(
            setPlaying: { [weak self] playing in
                playing.boolValue ? self?.activePlayer?.play() : self?.activePlayer?.pause()
            },
            seekTo: { [weak self] positionMs in
                self?.activePlayer?.seek(to: Double(positionMs) / 1000)
            },
            setVolume: { [weak self] volume in self?.activePlayer?.setVolume(volume.floatValue) },
            setSubtitleEnabled: { _ in },
            stop: { [weak self] in self?.activePlayer?.pause() }
        )
    )
    private lazy var watchBridge = AppleWatchTogetherBridge(
        callbacks: AppleWatchTogetherCallbacks(
            setPlaying: { [weak self] playing in
                playing.boolValue ? self?.activePlayer?.play() : self?.activePlayer?.pause()
            },
            seekTo: { [weak self] positionMs in
                self?.activePlayer?.seek(to: Double(positionMs) / 1000.0)
            },
            setSpeed: { [weak self] speed in
                self?.activePlayer?.setRate(speed.floatValue)
            },
            stateChanged: { [weak self] state in
                self?.handleWatchState(state)
            }
        )
    )

    func present(request: ApplePlaybackRequestSnapshot) {
        if shouldUseInfuse(), canLaunchInfuse() {
            presentInInfuse(request: request)
        } else {
            presentInApp(request: request)
        }
    }

    func handleOpenURL(_ url: URL) {
        guard url.scheme?.lowercased() == "fluxaapple" else { return }
        if url.host?.lowercased() == "infuse-error" {
            let fallback = externalPlaybackRequest
            externalPlaybackRequest = nil
            externalPlaybackContext = nil
            stateBridge.setContent(content: nil)
            FluxaAppleStreamingEngine.shared.stop()
            if let fallback { presentInApp(request: fallback) }
            return
        }
        guard url.host?.lowercased() == "infuse-complete" else { return }
        let components = URLComponents(url: url, resolvingAgainstBaseURL: false)
        let queryItems = components?.queryItems ?? []
        let reportedPosition = queryItems.first(where: { $0.name == "position" || $0.name == "lastPosition" })?.value
        let rawPosition = Double(reportedPosition ?? "") ?? 0
        let positionMs = Int64(rawPosition * 1000.0)
        let durationMs: Int64 = 0
        stateBridge.updatePlayback(
            isPlaying: false,
            isBuffering: false,
            positionMs: positionMs,
            durationMs: durationMs,
            bufferedPositionMs: positionMs,
            errorKey: nil
        )
        stateBridge.stop()
        stateBridge.setContent(content: nil)
        externalPlaybackContext = nil
        externalPlaybackRequest = nil
        FluxaAppleStreamingEngine.shared.stop()
    }

    private func presentInApp(request: ApplePlaybackRequestSnapshot) {
        guard let presenter = topViewController() else {
            return
        }
        let originalUrl = request.playableUrl
        let requestHeadersJson = request.requestHeadersJson
        let requestHeaders = decodeHeaders(requestHeadersJson)
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
            let fallbackURL: URL?
            if playbackUrl.path.hasSuffix("/remux") {
                // Remux failures must fall back to the local HTTP source. A
                // magnet URI cannot be consumed by the FFmpeg engine.
                fallbackURL = playbackUrl.deletingLastPathComponent()
            } else {
                fallbackURL = URL(string: originalUrl)
            }
            let player = FluxaPlayer()
            self.activePlayer = player
            self.watchBridge.attachPlayback(
                contentId: request.contentId.isEmpty ? originalUrl : request.contentId,
                contentType: request.contentType,
                videoId: request.videoId,
                title: title
            )
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
                    runtime: nil,
                    seasonsCount: nil
                )
            )
            self.observe(player)
            let controller = FluxaAppleCustomPlayerViewController(player: player, title: title)
            controller.onWatchParty = { [weak self] in
                if let state = self?.watchState, state.inRoom {
                    self?.presentWatchRoom(state)
                } else {
                    self?.presentWatchStartMenu()
                }
            }
            self.activePlayerController = controller
            presenter.present(controller, animated: true) {
                controller.presentationController?.delegate = self
                player.load(
                    FluxaPlaybackItem(
                        url: playbackUrl,
                        title: title,
                        headers: requestHeaders,
                        startPosition: Double(resumePositionMs) / 1000,
                        fallbackURL: fallbackURL,
                        subtitleUrls: request.subtitleUrls.compactMap { URL(string: $0) }
                    )
                )
                player.play()
            }
        }
    }

    private func presentInInfuse(request: ApplePlaybackRequestSnapshot) {
        watchBridge.leaveRoom()
        watchBridge.detachPlayback()
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
            guard let playbackUrl,
                  let infuseUrl = self.buildInfuseUrl(
                      playbackUrl: playbackUrl,
                      title: title,
                      resumePositionMs: Int64(resumePositionMs),
                      subtitleUrl: request.subtitleUrls.first
                  ) else {
                self.presentInApp(request: request)
                return
            }
            self.externalPlaybackContext = ExternalPlaybackContext(
                originalUrl: originalUrl,
                title: title,
                resumePositionMs: Int64(resumePositionMs)
            )
            self.externalPlaybackRequest = request
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
                    runtime: nil,
                    seasonsCount: nil
                )
            )
            self.stateBridge.updatePlayback(
                isPlaying: true,
                isBuffering: false,
                positionMs: Int64(resumePositionMs),
                durationMs: 0,
                bufferedPositionMs: Int64(resumePositionMs),
                errorKey: nil
            )
            UIApplication.shared.open(infuseUrl, options: [:]) { success in
                if !success {
                    self.externalPlaybackContext = nil
                    self.externalPlaybackRequest = nil
                    self.stateBridge.setContent(content: nil)
                    self.presentInApp(request: request)
                }
            }
        }
    }

    private func decodeHeaders(_ json: String) -> [String: String] {
        guard let data = json.data(using: .utf8),
              let object = try? JSONSerialization.jsonObject(with: data),
              let values = object as? [String: Any] else {
            return [:]
        }
        return values.compactMapValues { $0 as? String }
    }

    private func buildInfuseUrl(
        playbackUrl: URL,
        title: String,
        resumePositionMs: Int64,
        subtitleUrl: String?
    ) -> URL? {
        var components = URLComponents()
        components.scheme = "infuse"
        components.host = "x-callback-url"
        components.path = "/play"
        var queryItems = [
            URLQueryItem(name: "url", value: playbackUrl.absoluteString),
            URLQueryItem(name: "filename", value: title),
            URLQueryItem(name: "x-success", value: "fluxaapple://infuse-complete"),
            URLQueryItem(name: "x-error", value: "fluxaapple://infuse-error")
        ]
        if let subtitleUrl, !subtitleUrl.isEmpty {
            queryItems.append(URLQueryItem(name: "sub", value: subtitleUrl))
        }
        if resumePositionMs > 0 {
            queryItems.append(URLQueryItem(name: "position", value: String(Int64(resumePositionMs) / 1000)))
        }
        components.queryItems = queryItems
        return components.url
    }

    private func shouldUseInfuse() -> Bool {
        let defaults = UserDefaults.standard
        let preferred = defaults.string(forKey: "fluxa.apple.settings.preferredPlayer")?.lowercased() ?? "internal"
        guard preferred == "external" else { return false }
        let target = defaults.string(forKey: "fluxa.apple.settings.externalPlayerTarget")?.lowercased() ?? ""
        return target == "infuse"
    }

    private func canLaunchInfuse() -> Bool {
        guard let url = URL(string: "infuse://x-callback-url/ping") else { return false }
        return UIApplication.shared.canOpenURL(url)
    }

    private func presentWatchStartMenu() {
        guard let presenter = topViewController() else { return }
        let alert = UIAlertController(title: "Watch Party", message: "Create a room or join with a code.", preferredStyle: .actionSheet)
        alert.addAction(UIAlertAction(title: "Create room", style: .default) { [weak self] _ in
            self?.presentWatchConnectionForm(joining: false)
        })
        alert.addAction(UIAlertAction(title: "Join with code", style: .default) { [weak self] _ in
            self?.presentWatchConnectionForm(joining: true)
        })
        alert.addAction(UIAlertAction(title: "Cancel", style: .cancel))
        if let popover = alert.popoverPresentationController, let view = activePlayerController?.view {
            popover.sourceView = view
            popover.sourceRect = CGRect(x: view.bounds.midX, y: 40, width: 1, height: 1)
        }
        presenter.present(alert, animated: true)
    }

    private func presentWatchConnectionForm(joining: Bool) {
        guard let presenter = topViewController() else { return }
        let savedConfig = watchBridge.loadConfiguration(defaultDisplayName: "Guest")
        let alert = UIAlertController(
            title: joining ? "Join Watch Party" : "Create Watch Party",
            message: "Use your self-hosted Fluxa Watch Together server.",
            preferredStyle: .alert
        )
        alert.addTextField { field in
            field.placeholder = "https://watch.example.com"
            field.text = savedConfig.serverUrl
            field.keyboardType = .URL
            field.autocapitalizationType = .none
        }
        alert.addTextField { field in
            field.placeholder = "Server secret (optional)"
            field.text = savedConfig.serverSecret
            field.autocapitalizationType = .none
        }
        alert.addTextField { field in
            field.placeholder = "Display name"
            field.text = savedConfig.displayName
        }
        if joining {
            alert.addTextField { field in
                field.placeholder = "Room code"
                field.autocapitalizationType = .allCharacters
            }
        }
        alert.addAction(UIAlertAction(title: "Cancel", style: .cancel))
        alert.addAction(UIAlertAction(title: joining ? "Join" : "Create", style: .default) { [weak self, weak alert] _ in
            guard let self, let fields = alert?.textFields, fields.count >= 3 else { return }
            let server = fields[0].text?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
            let secret = fields[1].text ?? ""
            let name = fields[2].text?.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty == false ? fields[2].text! : "Guest"
            guard !server.isEmpty else {
                self.presentWatchError("Enter your self-hosted server URL.")
                return
            }
            self.watchBridge.configure(serverUrl: server, serverSecret: secret, displayName: name)
            self.pendingWatchRoomPresentation = true
            if joining {
                let code = fields.count > 3 ? (fields[3].text ?? "") : ""
                self.watchBridge.joinRoom(code: code.uppercased())
            } else {
                self.watchBridge.createRoom()
            }
        })
        presenter.present(alert, animated: true)
    }

    private func handleWatchState(_ state: AppleWatchTogetherStateSnapshot) {
        watchState = state
        // Guests follow the authoritative host; disable local transport controls to prevent
        // play/pause/seek fights until they leave the room or become host.
        activePlayerController?.setControlsEnabled(!state.inRoom || state.isHost)
        if pendingWatchRoomPresentation, state.inRoom {
            pendingWatchRoomPresentation = false
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak self] in
                self?.presentWatchRoom(state)
            }
        } else if pendingWatchRoomPresentation, let error = state.errorMessage, !error.isEmpty {
            pendingWatchRoomPresentation = false
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) { [weak self] in
                self?.presentWatchError(error)
            }
        }
    }

    private func presentWatchRoom(_ state: AppleWatchTogetherStateSnapshot) {
        guard let presenter = topViewController() else { return }
        let members = state.memberNames.isEmpty ? "" : "\n\nParticipants\n" + state.memberNames.joined(separator: "\n")
        let role = state.isHost ? "You are the host. Share this code; your playback controls the room." : "Connected. Playback follows the host."
        let alert = UIAlertController(
            title: "Watch Party · \(state.roomCode ?? "")",
            message: role + members,
            preferredStyle: .alert
        )
        alert.addAction(UIAlertAction(title: "Leave room", style: .destructive) { [weak self] _ in
            self?.watchBridge.leaveRoom()
        })
        alert.addAction(UIAlertAction(title: "Done", style: .cancel))
        presenter.present(alert, animated: true)
    }

    private func presentWatchError(_ message: String) {
        guard let presenter = topViewController() else { return }
        let alert = UIAlertController(title: "Watch Party", message: message, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "OK", style: .default))
        presenter.present(alert, animated: true)
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
        activePlayer?.stop()
        watchBridge.leaveRoom()
        watchBridge.detachPlayback()
        activePlayerController = nil
        stateBridge.stop()
        stateBridge.setContent(content: nil)
        activePlayer = nil
        FluxaAppleStreamingEngine.shared.stop()
    }

    private func observe(_ player: FluxaPlayer) {
        player.onStateChange = { [weak self] state in
            self?.publish(state)
        }
    }

    private func publish(_ state: FluxaPlaybackState) {
        let positionMs = Int64(state.position * 1000)
        let durationMs = Int64(state.duration * 1000)
        let bufferedMs = Int64(state.buffered * 1000)
        stateBridge.updatePlayback(
            isPlaying: state.isPlaying,
            isBuffering: state.isBuffering,
            positionMs: positionMs,
            durationMs: durationMs,
            bufferedPositionMs: bufferedMs,
            errorKey: state.failure == nil ? nil : "error.generic_message"
        )
        watchBridge.updatePlayback(
            positionMs: positionMs,
            durationMs: durationMs,
            isPlaying: state.isPlaying,
            isBuffering: state.isBuffering
        )
    }
}
