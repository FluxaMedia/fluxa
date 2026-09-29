import AVFoundation
import MediaPlayer

@MainActor
final class FluxaNowPlaying {
    var onCommand: ((String, Double) -> Void)?

    private var targets: [(MPRemoteCommand, Any)] = []
    private var interruptionObserver: NSObjectProtocol?
    private var routeObserver: NSObjectProtocol?
    private var resumeAfterInterruption = false
    private var active = false

    func update(_ plan: [String: Any]) {
        if !active { activate() }
        var info: [String: Any] = [
            MPMediaItemPropertyTitle: plan["title"] as? String ?? "",
            MPMediaItemPropertyPlaybackDuration: (plan["durationMs"] as? Double ?? 0) / 1000,
            MPNowPlayingInfoPropertyElapsedPlaybackTime: (plan["positionMs"] as? Double ?? 0) / 1000,
            MPNowPlayingInfoPropertyDefaultPlaybackRate: 1.0,
            MPNowPlayingInfoPropertyPlaybackRate: plan["state"] as? String == "playing" ? (plan["speed"] as? Double ?? 1) : 0,
        ]
        if let subtitle = plan["subtitle"] as? String, !subtitle.isEmpty {
            info[MPMediaItemPropertyArtist] = subtitle
        }
        let center = MPNowPlayingInfoCenter.default()
        center.nowPlayingInfo = info
        center.playbackState = plan["state"] as? String == "playing" ? .playing : .paused
        let actions = plan["actions"] as? [String] ?? []
        let commands = MPRemoteCommandCenter.shared()
        commands.nextTrackCommand.isEnabled = actions.contains("next")
        commands.changePlaybackPositionCommand.isEnabled = actions.contains("seekTo")
    }

    func deactivate() {
        guard active else { return }
        active = false
        targets.forEach { $0.0.removeTarget($0.1) }
        targets.removeAll()
        for observer in [interruptionObserver, routeObserver].compactMap({ $0 }) {
            NotificationCenter.default.removeObserver(observer)
        }
        interruptionObserver = nil
        routeObserver = nil
        MPNowPlayingInfoCenter.default().nowPlayingInfo = nil
        try? AVAudioSession.sharedInstance().setActive(false, options: .notifyOthersOnDeactivation)
    }

    private func activate() {
        active = true
        let session = AVAudioSession.sharedInstance()
        try? session.setCategory(.playback, mode: .moviePlayback)
        try? session.setActive(true)
        let commands = MPRemoteCommandCenter.shared()
        bind(commands.playCommand, "play")
        bind(commands.pauseCommand, "pause")
        bind(commands.togglePlayPauseCommand, "toggle")
        bind(commands.stopCommand, "stop")
        bind(commands.nextTrackCommand, "next")
        bind(commands.previousTrackCommand, "previous")
        commands.skipForwardCommand.preferredIntervals = [10]
        commands.skipBackwardCommand.preferredIntervals = [10]
        bind(commands.skipForwardCommand, "seekForward") { ($0 as? MPSkipIntervalCommandEvent)?.interval ?? 10 }
        bind(commands.skipBackwardCommand, "seekBackward") { ($0 as? MPSkipIntervalCommandEvent)?.interval ?? 10 }
        bind(commands.changePlaybackPositionCommand, "seekTo") {
            ($0 as? MPChangePlaybackPositionCommandEvent)?.positionTime ?? 0
        }
        interruptionObserver = NotificationCenter.default.addObserver(
            forName: AVAudioSession.interruptionNotification,
            object: nil,
            queue: .main
        ) { [weak self] note in
            let raw = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt
            let options = note.userInfo?[AVAudioSessionInterruptionOptionKey] as? UInt
            MainActor.assumeIsolated { self?.interruption(raw, options) }
        }
        routeObserver = NotificationCenter.default.addObserver(
            forName: AVAudioSession.routeChangeNotification,
            object: nil,
            queue: .main
        ) { [weak self] note in
            let raw = note.userInfo?[AVAudioSessionRouteChangeReasonKey] as? UInt
            MainActor.assumeIsolated {
                if raw == AVAudioSession.RouteChangeReason.oldDeviceUnavailable.rawValue {
                    self?.onCommand?("pause", 0)
                }
            }
        }
    }

    private func interruption(_ raw: UInt?, _ options: UInt?) {
        switch raw.flatMap(AVAudioSession.InterruptionType.init(rawValue:)) {
        case .began:
            resumeAfterInterruption = true
            onCommand?("pause", 0)
        case .ended:
            let resume = options.map(AVAudioSession.InterruptionOptions.init(rawValue:))?.contains(.shouldResume) ?? false
            if resumeAfterInterruption && resume { onCommand?("play", 0) }
            resumeAfterInterruption = false
        default:
            break
        }
    }

    private func bind(_ command: MPRemoteCommand, _ name: String, value: @escaping (MPRemoteCommandEvent) -> Double = { _ in 0 }) {
        command.isEnabled = true
        let token = command.addTarget { [weak self] event in
            let amount = value(event)
            Task { @MainActor in self?.onCommand?(name, amount) }
            return .success
        }
        targets.append((command, token))
    }
}
