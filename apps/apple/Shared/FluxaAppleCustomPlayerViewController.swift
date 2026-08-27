import FluxaPlayerKit
import SwiftUI
import UIKit

@MainActor
final class FluxaAppleCustomPlayerChrome: ObservableObject {
    @Published var controlsVisible = true
    @Published var controlsEnabled = true
}

@MainActor
final class FluxaAppleCustomPlayerViewController: UIViewController {
    let player: FluxaPlayer
    let titleText: String
    let chrome = FluxaAppleCustomPlayerChrome()
    var onWatchParty: (() -> Void)?

    private let videoView = FluxaPlayerSurfaceView()
    private var host: UIHostingController<FluxaApplePlayerOverlay>?
    private var hideTask: Task<Void, Never>?

    init(player: FluxaPlayer, title: String) {
        self.player = player
        self.titleText = title
        super.init(nibName: nil, bundle: nil)
        modalPresentationStyle = .fullScreen
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) { fatalError("init(coder:) has not been implemented") }

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .black
        videoView.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(videoView)
        NSLayoutConstraint.activate([
            videoView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            videoView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            videoView.topAnchor.constraint(equalTo: view.topAnchor),
            videoView.bottomAnchor.constraint(equalTo: view.bottomAnchor)
        ])
        player.attach(to: videoView)

        let overlay = FluxaApplePlayerOverlay(
            title: titleText,
            player: player,
            chrome: chrome,
            onPlayPause: { [weak self] in self?.togglePlayback() },
            onSeek: { [weak self] value in self?.seek(to: value) },
            onSkip: { [weak self] seconds in self?.skip(seconds: seconds) },
            onClose: { [weak self] in self?.dismiss(animated: true) },
            onWatchParty: { [weak self] in self?.onWatchParty?() },
            onInteraction: { [weak self] in self?.scheduleHide() }
        )
        let host = UIHostingController(rootView: overlay)
        host.view.backgroundColor = .clear
        host.view.translatesAutoresizingMaskIntoConstraints = false
        addChild(host)
        view.addSubview(host.view)
        NSLayoutConstraint.activate([
            host.view.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            host.view.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            host.view.topAnchor.constraint(equalTo: view.topAnchor),
            host.view.bottomAnchor.constraint(equalTo: view.bottomAnchor)
        ])
        host.didMove(toParent: self)
        self.host = host
        scheduleHide()
    }

    #if os(iOS)
    override var prefersStatusBarHidden: Bool { true }
    #endif

    func setControlsEnabled(_ enabled: Bool) {
        chrome.controlsEnabled = enabled
    }

    private func togglePlayback() {
        guard chrome.controlsEnabled else { return }
        player.togglePlayback()
        scheduleHide()
    }

    private func seek(to value: Double) {
        guard chrome.controlsEnabled else { return }
        player.seek(to: value)
        scheduleHide()
    }

    private func skip(seconds: Double) {
        guard chrome.controlsEnabled else { return }
        player.skip(by: seconds)
        scheduleHide()
    }

    private func scheduleHide() {
        chrome.controlsVisible = true
        hideTask?.cancel()
        hideTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: .seconds(4))
            guard !Task.isCancelled else { return }
            self?.chrome.controlsVisible = false
        }
    }

    deinit { hideTask?.cancel() }
}

private struct FluxaApplePlayerOverlay: View {
    let title: String
    @ObservedObject var player: FluxaPlayer
    @ObservedObject var chrome: FluxaAppleCustomPlayerChrome
    let onPlayPause: () -> Void
    let onSeek: (Double) -> Void
    let onSkip: (Double) -> Void
    let onClose: () -> Void
    let onWatchParty: () -> Void
    let onInteraction: () -> Void

    var body: some View {
        ZStack {
            Color.clear
                .contentShape(Rectangle())
                .onTapGesture { onInteraction() }
            if player.state.isBuffering { ProgressView().tint(.white).scaleEffect(1.35) }
            if let subtitleText = player.subtitleText, !subtitleText.isEmpty {
                Text(subtitleText)
                    .multilineTextAlignment(.center)
                    .font(.title3.weight(.medium))
                    .padding(.horizontal, 18)
                    .padding(.vertical, 8)
                    .background(.black.opacity(0.72))
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                    .padding(.horizontal, 32)
                    .padding(.bottom, chrome.controlsVisible ? 92 : 28)
            }
            if let failure = player.state.failure {
                Text(failure.reason).foregroundStyle(.white).padding(18).background(.black.opacity(0.7)).clipShape(RoundedRectangle(cornerRadius: 12))
            }
            if chrome.controlsVisible {
                VStack(spacing: 0) {
                    topBar
                    Spacer()
                    centerControls
                    Spacer()
                    bottomBar
                }
                .background(
                    LinearGradient(colors: [.black.opacity(0.72), .clear, .black.opacity(0.86)], startPoint: .top, endPoint: .bottom)
                        .allowsHitTesting(false)
                )
            }
        }
        .foregroundStyle(.white)
        .onAppear { onInteraction() }
    }

    private var topBar: some View {
        HStack(spacing: 14) {
            Button(action: onClose) { Image(systemName: "xmark").font(.title3.bold()) }
            Text(title).font(.headline).lineLimit(1)
            Spacer()
            if !player.tracks(of: .audio).isEmpty {
                trackMenu(kind: .audio, icon: "waveform")
            }
            if !player.tracks(of: .subtitle).isEmpty {
                trackMenu(kind: .subtitle, icon: "captions.bubble")
            }
            Button(action: onWatchParty) { Image(systemName: "person.2.fill") }
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 28)
        .padding(.top, 24)
    }

    @ViewBuilder
    private func trackMenu(kind: FluxaTrackKind, icon: String) -> some View {
        Menu {
            ForEach(player.tracks(of: kind)) { track in
                Button {
                    player.selectTrack(track, kind: kind)
                } label: {
                    HStack {
                        Text(track.label)
                        if let languageCode = track.languageCode, !languageCode.isEmpty {
                            Text(languageCode.uppercased())
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
            if kind == .subtitle {
                Button {
                    player.selectTrack(nil, kind: .subtitle)
                } label: {
                    Image(systemName: "nosign")
                }
            }
        } label: {
            Image(systemName: icon)
        }
    }

    private var centerControls: some View {
        HStack(spacing: 36) {
            Button { onSkip(-10) } label: { Image(systemName: "gobackward.10").font(.title2) }
            Button(action: onPlayPause) {
                Image(systemName: player.state.isPlaying ? "pause.fill" : "play.fill").font(.system(size: 34, weight: .bold))
                    .frame(width: 72, height: 72).background(.white.opacity(0.16)).clipShape(Circle())
            }
            Button { onSkip(10) } label: { Image(systemName: "goforward.10").font(.title2) }
        }
        .buttonStyle(.plain)
        .disabled(!chrome.controlsEnabled)
    }

    private var bottomBar: some View {
        VStack(spacing: 8) {
            GeometryReader { proxy in
                ZStack(alignment: .leading) {
                    Capsule().fill(.white.opacity(0.25))
                    Capsule()
                        .fill(.white.opacity(0.42))
                        .frame(width: proxy.size.width * progress(player.state.buffered))
                    #if os(tvOS)
                    ProgressView(value: player.state.position, total: max(player.state.duration, 1))
                        .tint(.white)
                    #else
                    Slider(
                        value: Binding(get: { player.state.position }, set: onSeek),
                        in: 0...max(player.state.duration, 1)
                    )
                    .tint(.white)
                    .disabled(!chrome.controlsEnabled)
                    #endif
                }
            }
            .frame(height: 22)
            HStack {
                Text(formatTime(player.state.position))
                Spacer()
                Text(formatTime(player.state.duration))
            }.font(.caption.monospacedDigit()).foregroundStyle(.white.opacity(0.86))
        }
        .padding(.horizontal, 28).padding(.bottom, 24)
    }

    private func progress(_ value: Double) -> CGFloat {
        guard player.state.duration > 0 else { return 0 }
        return CGFloat(min(1, max(0, value / player.state.duration)));
    }

    private func formatTime(_ seconds: Double) -> String {
        guard seconds.isFinite else { return "00:00" }
        let total = max(0, Int(seconds.rounded()))
        return String(format: "%02d:%02d", total / 60, total % 60)
    }
}
