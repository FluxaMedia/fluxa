#if FLUXA_FFMPEG
import AVFoundation
import CoreMedia
import Foundation

@MainActor
final class FluxaFFmpegEngine: FluxaPlaybackEngine {
    weak var delegate: FluxaPlaybackEngineDelegate?

    private let pipeline = FluxaFFmpegPipeline()
    private var state = FluxaPlaybackState()
    private var timeObserver: Any?
    private var desiredRate: Float = 1
    private var startPosition: TimeInterval = 0

    init() {
        pipeline.onLoaded = { [weak self] streams, duration, _ in
            Task { @MainActor in self?.handleLoaded(streams: streams, duration: duration) }
        }
        pipeline.onFailure = { [weak self] reason in
            Task { @MainActor in self?.fail(reason: reason) }
        }
    }

    func attach(to surface: FluxaPlayerSurfaceView) {
        surface.host(pipeline.displayLayer)
    }

    func load(_ item: FluxaPlaybackItem) {
        removeTimeObserver()
        pipeline.close()
        startPosition = item.startPosition
        state = FluxaPlaybackState()
        state.phase = .loading
        state.isBuffering = true
        publishState()
        pipeline.open(item)
    }

    func play() {
        guard state.failure == nil else { return }
        desiredRate = state.rate == 0 ? 1 : state.rate
        pipeline.setRate(desiredRate, at: pipeline.currentTime)
        state.phase = .playing
        publishState()
    }

    func pause() {
        pipeline.setRate(0, at: pipeline.currentTime)
        state.phase = .paused
        publishState()
    }

    func seek(to position: TimeInterval) {
        let target = max(0, position)
        state.position = target
        state.isBuffering = true
        publishState()

        let rate = state.isPlaying ? desiredRate : 0
        pipeline.seek(to: target) { [weak self] in
            Task { @MainActor in
                guard let self else { return }
                self.pipeline.setRate(rate, at: CMTime(seconds: target, preferredTimescale: 600))
                self.state.isBuffering = false
                self.publishState()
            }
        }
    }

    func setRate(_ rate: Float) {
        desiredRate = rate
        state.rate = rate
        if state.isPlaying {
            pipeline.setRate(rate, at: pipeline.currentTime)
        }
        publishState()
    }

    func setVolume(_ volume: Float) {
        pipeline.setVolume(volume)
    }

    func selectTrack(_ track: FluxaTrack?, kind: FluxaTrackKind) {
        guard kind == .audio,
              let track,
              let index = Int32(track.id.split(separator: ".").last ?? "") else { return }
        pipeline.selectAudioStream(index: index)
    }

    func tearDown() {
        removeTimeObserver()
        pipeline.close()
        pipeline.displayLayer.removeFromSuperlayer()
    }

    private func handleLoaded(streams: [FluxaFFmpegStream], duration: TimeInterval) {
        let tracks = streams.compactMap { stream -> FluxaTrack? in
            guard let kind = stream.kind else { return nil }
            return FluxaTrack(
                id: "\(kind.rawValue).\(stream.index)",
                kind: kind,
                label: stream.title ?? stream.language ?? "Track \(stream.index)",
                languageCode: stream.language,
                isForced: stream.isForced
            )
        }
        delegate?.engine(self, didUpdate: tracks)

        state.duration = duration
        state.isSeekable = duration > 0
        state.phase = .paused
        state.isBuffering = false
        publishState()

        installTimeObserver()
        pipeline.start(from: startPosition)
    }

    private func installTimeObserver() {
        removeTimeObserver()
        timeObserver = pipeline.synchronizer.addPeriodicTimeObserver(
            forInterval: CMTime(seconds: 0.25, preferredTimescale: 600),
            queue: .main
        ) { [weak self] time in
            MainActor.assumeIsolated { self?.handleTick(time) }
        }
    }

    private func removeTimeObserver() {
        guard let timeObserver else { return }
        pipeline.synchronizer.removeTimeObserver(timeObserver)
        self.timeObserver = nil
    }

    private func handleTick(_ time: CMTime) {
        guard time.isValid else { return }
        state.position = max(0, time.seconds)
        if pipeline.isDrained, state.phase == .playing {
            state.phase = .ended
            pipeline.setRate(0, at: time)
        }
        publishState()
    }

    private func fail(reason: String) {
        state.phase = .failed(FluxaPlaybackFailure(reason: reason, isRecoverable: false))
        state.isBuffering = false
        publishState()
    }

    private func publishState() {
        delegate?.engine(self, didUpdate: state)
    }
}
#endif
