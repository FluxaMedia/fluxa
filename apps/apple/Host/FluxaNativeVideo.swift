import AVFoundation
import FluxaRendererFFI
import UIKit

@MainActor
final class FluxaNativeVideo {
    let layer = AVPlayerLayer()
    var onPlayingChanged: ((Bool) -> Void)?

    private var player: AVPlayer?
    private var audioLanguage = ""
    private var subtitleLanguage = ""
    private var error: String?

    init() {
        layer.videoGravity = .resizeAspect
        layer.backgroundColor = UIColor.black.cgColor
        layer.isHidden = true
    }

    func tick() {
        if let value = fluxa_renderer_poll_video() {
            let json = String(cString: value)
            fluxa_renderer_string_free(value)
            let requests = (try? JSONSerialization.jsonObject(with: Data(json.utf8))) as? [[String: Any]] ?? []
            requests.forEach(handle)
        }
        report()
    }

    private func handle(_ request: [String: Any]) {
        switch request["type"] as? String {
        case "configure":
            audioLanguage = request["audioLanguage"] as? String ?? ""
            subtitleLanguage = request["subtitleLanguage"] as? String ?? ""
        case "load":
            guard let raw = request["url"] as? String, let url = URL(string: raw) else { return }
            load(url)
        case "stop":
            stop()
        case "togglePause":
            guard let player else { return }
            player.timeControlStatus == .paused ? player.play() : player.pause()
        case "seek":
            guard let player, let seconds = request["seconds"] as? Double else { return }
            seek(player.currentTime().seconds + seconds)
        case "seekTo":
            guard let seconds = request["seconds"] as? Double else { return }
            seek(seconds)
        case "toggleMute":
            player?.isMuted.toggle()
        default:
            break
        }
    }

    private func load(_ url: URL) {
        stop()
        error = nil
        try? AVAudioSession.sharedInstance().setCategory(.playback, mode: .moviePlayback)
        try? AVAudioSession.sharedInstance().setActive(true)
        let item = AVPlayerItem(url: url)
        let player = AVPlayer(playerItem: item)
        self.player = player
        layer.player = player
        layer.isHidden = false
        Task { await selectLanguages(item) }
        player.play()
        onPlayingChanged?(true)
    }

    private func stop() {
        guard let player else { return }
        player.pause()
        layer.player = nil
        layer.isHidden = true
        self.player = nil
        onPlayingChanged?(false)
    }

    private func seek(_ seconds: Double) {
        player?.seek(to: CMTime(seconds: max(0, seconds), preferredTimescale: 600))
    }

    private func selectLanguages(_ item: AVPlayerItem) async {
        for (characteristic, language) in [(AVMediaCharacteristic.audible, audioLanguage), (.legible, subtitleLanguage)] {
            guard !language.isEmpty,
                  let group = try? await item.asset.loadMediaSelectionGroup(for: characteristic) else { continue }
            let options = AVMediaSelectionGroup.mediaSelectionOptions(
                from: group.options,
                with: Locale(identifier: language)
            )
            if let option = options.first {
                item.select(option, in: group)
            }
        }
    }

    private func report() {
        guard let player, let item = player.currentItem else { return }
        if item.status == .failed {
            error = item.error?.localizedDescription ?? "playback failed"
        }
        let duration = item.duration.seconds
        let position = player.currentTime().seconds
        let buffering: Float = player.timeControlStatus == .waitingToPlayAtSpecifiedRate ? 0 : -1
        fluxa_renderer_video_status(
            position.isFinite ? position : 0,
            duration.isFinite ? duration : 0,
            player.timeControlStatus == .paused,
            item.status == .readyToPlay,
            buffering,
            error
        )
    }
}
