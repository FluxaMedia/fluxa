#if !os(macOS)
import AVFoundation

@MainActor
final class FluxaAudioSession {
    private var observers: [NSObjectProtocol] = []
    private var active = false

    func activate() {
        let session = AVAudioSession.sharedInstance()
        active = false
        removeObservers()
        do {
            #if os(iOS)
            try session.setCategory(
                .playback,
                mode: .moviePlayback,
                options: [.allowAirPlay, .allowBluetoothA2DP]
            )
            #else
            try session.setCategory(.playback, mode: .moviePlayback, options: [.allowAirPlay])
            #endif
            try session.setActive(true, options: [])
        } catch {
            return
        }
        active = true

        let center = NotificationCenter.default
        observers.append(center.addObserver(
            forName: AVAudioSession.routeChangeNotification,
            object: session,
            queue: .main
        ) { [weak self] _ in
            MainActor.assumeIsolated { self?.reactivate() }
        })
        observers.append(center.addObserver(
            forName: AVAudioSession.interruptionNotification,
            object: session,
            queue: .main
        ) { [weak self] notification in
            guard let typeValue = notification.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt,
                  let type = AVAudioSession.InterruptionType(rawValue: typeValue) else { return }
            guard type == .ended else { return }
            MainActor.assumeIsolated { self?.reactivate() }
        })
        observers.append(center.addObserver(
            forName: AVAudioSession.mediaServicesWereResetNotification,
            object: session,
            queue: .main
        ) { [weak self] _ in
            MainActor.assumeIsolated { self?.reconfigureAndReactivate() }
        })
    }

    func deactivate() {
        active = false
        removeObservers()
        try? AVAudioSession.sharedInstance().setActive(false, options: [.notifyOthersOnDeactivation])
    }

    private func reactivate() {
        guard active else { return }
        try? AVAudioSession.sharedInstance().setActive(true, options: [])
    }

    private func reconfigureAndReactivate() {
        guard active else { return }
        activate()
    }

    private func removeObservers() {
        let center = NotificationCenter.default
        observers.forEach { center.removeObserver($0) }
        observers.removeAll()
    }
}
#endif
