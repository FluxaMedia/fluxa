import AVFoundation
import SwiftUI
import UIKit

@MainActor
final class FluxaAppleCustomPlayerState: ObservableObject {
    @Published var position: Double = 0
    @Published var duration: Double = 0
    @Published var buffered: Double = 0
    @Published var isPlaying = false
    @Published var isBuffering = false
    @Published var errorMessage: String?
    @Published var controlsVisible = true
    @Published var controlsEnabled = true
}

final class FluxaApplePlayerLayerView: UIView {
    override class var layerClass: AnyClass { AVPlayerLayer.self }

    var playerLayer: AVPlayerLayer { layer as! AVPlayerLayer }
}

@MainActor
final class FluxaAppleCustomPlayerViewController: UIViewController {
    let player: AVPlayer
    let titleText: String
    let state = FluxaAppleCustomPlayerState()
    var onWatchParty: (() -> Void)?

    private let videoView = FluxaApplePlayerLayerView()
    private var host: UIHostingController<FluxaApplePlayerOverlay>?
    private var hideTask: Task<Void, Never>?

    init(player: AVPlayer, title: String) {
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
        videoView.playerLayer.player = player
        videoView.playerLayer.videoGravity = .resizeAspect
        view.addSubview(videoView)
        NSLayoutConstraint.activate([
            videoView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            videoView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            videoView.topAnchor.constraint(equalTo: view.topAnchor),
            videoView.bottomAnchor.constraint(equalTo: view.bottomAnchor)
        ])

        let overlay = FluxaApplePlayerOverlay(
            title: titleText,
            state: state,
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

    override var prefersStatusBarHidden: Bool { true }

    func update(position: Double, duration: Double, buffered: Double, isPlaying: Bool, isBuffering: Bool, errorMessage: String?) {
        state.position = max(0, position)
        state.duration = max(0, duration)
        state.buffered = max(0, buffered)
        state.isPlaying = isPlaying
        state.isBuffering = isBuffering
        state.errorMessage = errorMessage
    }

    func setControlsEnabled(_ enabled: Bool) {
        state.controlsEnabled = enabled
    }

    private func togglePlayback() {
        guard state.controlsEnabled else { return }
        if state.isPlaying { player.pause() } else { player.play() }
        scheduleHide()
    }

    private func seek(to value: Double) {
        guard state.controlsEnabled else { return }
        player.seek(to: CMTime(seconds: value, preferredTimescale: 600))
        scheduleHide()
    }

    private func skip(seconds: Double) {
        guard state.controlsEnabled else { return }
        let target = min(max(0, player.currentTime().seconds + seconds), state.duration)
        seek(to: target)
    }

    private func scheduleHide() {
        state.controlsVisible = true
        hideTask?.cancel()
        hideTask = Task { @MainActor [weak self] in
            try? await Task.sleep(for: .seconds(4))
            guard !Task.isCancelled else { return }
            self?.state.controlsVisible = false
        }
    }

    deinit { hideTask?.cancel() }
}

private struct FluxaApplePlayerOverlay: View {
    let title: String
    @ObservedObject var state: FluxaAppleCustomPlayerState
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
            if state.isBuffering { ProgressView().tint(.white).scaleEffect(1.35) }
            if let errorMessage = state.errorMessage {
                Text(errorMessage).foregroundStyle(.white).padding(18).background(.black.opacity(0.7)).clipShape(RoundedRectangle(cornerRadius: 12))
            }
            if state.controlsVisible {
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
            Button(action: onWatchParty) { Image(systemName: "person.2.fill") }
        }
        .buttonStyle(.plain)
        .padding(.horizontal, 28)
        .padding(.top, 24)
    }

    private var centerControls: some View {
        HStack(spacing: 36) {
            Button { onSkip(-10) } label: { Image(systemName: "gobackward.10").font(.title2) }
            Button(action: onPlayPause) {
                Image(systemName: state.isPlaying ? "pause.fill" : "play.fill").font(.system(size: 34, weight: .bold))
                    .frame(width: 72, height: 72).background(.white.opacity(0.16)).clipShape(Circle())
            }
            Button { onSkip(10) } label: { Image(systemName: "goforward.10").font(.title2) }
        }
        .buttonStyle(.plain)
        .disabled(!state.controlsEnabled)
    }

    private var bottomBar: some View {
        VStack(spacing: 8) {
            GeometryReader { proxy in
                ZStack(alignment: .leading) {
                    Capsule().fill(.white.opacity(0.25))
                    Capsule()
                        .fill(.white.opacity(0.42))
                        .frame(width: proxy.size.width * progress(state.buffered))
                    Slider(value: Binding(get: { state.position }, set: onSeek), in: 0...max(state.duration, 1))
                        .tint(.white).disabled(!state.controlsEnabled)
                }
            }
            .frame(height: 22)
            HStack {
                Text(formatTime(state.position))
                Spacer()
                Text(formatTime(state.duration))
            }.font(.caption.monospacedDigit()).foregroundStyle(.white.opacity(0.86))
        }
        .padding(.horizontal, 28).padding(.bottom, 24)
    }

    private func progress(_ value: Double) -> CGFloat {
        guard state.duration > 0 else { return 0 }
        return CGFloat(min(1, max(0, value / state.duration)));
    }

    private func formatTime(_ seconds: Double) -> String {
        guard seconds.isFinite else { return "00:00" }
        let total = max(0, Int(seconds.rounded()))
        return String(format: "%02d:%02d", total / 60, total % 60)
    }
}
