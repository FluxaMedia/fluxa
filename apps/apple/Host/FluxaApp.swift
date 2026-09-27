import SwiftUI
import UIKit

@main
struct FluxaApp: App {
    var body: some Scene {
        WindowGroup {
            FluxaRootView()
                .ignoresSafeArea()
                .onOpenURL { NotificationCenter.default.post(name: FluxaHostView.openURL, object: $0) }
        }
    }
}

struct FluxaRootView: UIViewRepresentable {
    func makeUIView(context: Context) -> FluxaHostView {
        FluxaHostView()
    }

    func updateUIView(_ view: FluxaHostView, context: Context) {}
}

@MainActor
final class FluxaHostView: UIView {
    private let renderer = FluxaNativeRendererView()

    override init(frame: CGRect) {
        super.init(frame: frame)
        backgroundColor = .black
        layer.addSublayer(renderer.video.layer)
        addSubview(renderer)
        renderer.video.onPlayingChanged = { playing in
            UIApplication.shared.isIdleTimerDisabled = playing
        }
        renderer.onActions = { print("Unhandled native actions: \($0)") }
        let dataDir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("fluxa-native")
        try? FileManager.default.createDirectory(at: dataDir, withIntermediateDirectories: true)
        renderer.startSession(dataDir: dataDir.path)
        NotificationCenter.default.addObserver(
            forName: FluxaHostView.openURL,
            object: nil,
            queue: .main
        ) { [weak self] note in
            guard let url = note.object as? URL else { return }
            MainActor.assumeIsolated { self?.open(url) }
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) is not supported")
    }

    static let openURL = Notification.Name("FluxaOpenURL")

    override func layoutSubviews() {
        super.layoutSubviews()
        renderer.frame = bounds
        renderer.video.layer.frame = bounds
    }

    private func open(_ url: URL) {
        let items = URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems ?? []
        let value = { (name: String) in items.first { $0.name == name }?.value ?? "" }
        let id = value("id")
        let type = value("type")
        guard !id.isEmpty, !type.isEmpty,
              let data = try? JSONSerialization.data(withJSONObject: ["type": "detail", "id": id, "itemType": type]),
              let json = String(data: data, encoding: .utf8) else { return }
        renderer.pushAction(json)
    }
}
