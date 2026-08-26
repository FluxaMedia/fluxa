#if FLUXA_FFMPEG
import AVFoundation
import CoreMedia
import Foundation

final class FluxaFFmpegPipeline: @unchecked Sendable {
    let displayLayer = AVSampleBufferDisplayLayer()
    let audioRenderer = AVSampleBufferAudioRenderer()
    let synchronizer = AVSampleBufferRenderSynchronizer()

    var onLoaded: (@Sendable ([FluxaFFmpegStream], TimeInterval, Bool) -> Void)?
    var onFailure: (@Sendable (String) -> Void)?

    private let demuxQueue = DispatchQueue(label: "media.fluxa.ffmpeg.demux")
    private let videoQueue = DispatchQueue(label: "media.fluxa.ffmpeg.video")
    private let audioQueue = DispatchQueue(label: "media.fluxa.ffmpeg.audio")

    private let demuxer = FluxaFFmpegDemuxer()
    private let audioDecoder = FluxaFFmpegAudioDecoder()
    private let store = FluxaFFmpegBufferStore()

    private var videoFormatDescription: CMFormatDescription?
    private var videoNeedsLengthPrefix = false
    private var started = false

    init() {
        displayLayer.videoGravity = .resizeAspect
        synchronizer.addRenderer(displayLayer)
        synchronizer.addRenderer(audioRenderer)
    }

    var currentTime: CMTime { synchronizer.currentTime() }

    var isDrained: Bool { store.reachedEnd && store.isEmpty }

    func open(_ item: FluxaPlaybackItem) {
        demuxQueue.async { [self] in
            do {
                try demuxer.open(url: item.url, headers: item.headers)
            } catch {
                onFailure?("Could not open this stream")
                return
            }

            if let videoStream = demuxer.streams.first(where: { $0.index == demuxer.videoStreamIndex }) {
                videoFormatDescription = FluxaSampleBufferFactory.videoFormatDescription(for: videoStream)
                videoNeedsLengthPrefix = !FluxaSampleBufferFactory.isLengthPrefixedExtradata(videoStream.extradata)
            }
            let audioStream = demuxer.streams.first { $0.index == demuxer.audioStreamIndex }
            let audioReady = audioStream.map { audioDecoder.open($0) } ?? false
            let hasVideo = demuxer.videoStreamIndex >= 0

            // A video stream that has no sample-buffer format description is
            // not playable by this compressed-sample path. Do not report a
            // successful audio-only load for a movie and leave the user with
            // a black screen; surface the failure so the caller can stop or
            // choose another backend.
            guard (!hasVideo || videoFormatDescription != nil),
                  videoFormatDescription != nil || audioReady else {
                onFailure?("No playable track in this stream")
                return
            }
            onLoaded?(demuxer.streams, demuxer.duration, audioReady)
        }
    }

    func start(from position: TimeInterval) {
        guard !started else { return }
        started = true

        displayLayer.requestMediaDataWhenReady(on: videoQueue) { [self] in
            while displayLayer.isReadyForMoreMediaData {
                guard let buffer = store.nextVideo() else { break }
                displayLayer.enqueue(buffer)
            }
            requestFill()
        }
        audioRenderer.requestMediaDataWhenReady(on: audioQueue) { [self] in
            while audioRenderer.isReadyForMoreMediaData {
                guard let buffer = store.nextAudio() else { break }
                audioRenderer.enqueue(buffer)
            }
            requestFill()
        }

        demuxQueue.async { [self] in
            if position > 0 {
                demuxer.seek(to: position)
            }
            fill()
        }
    }

    func setRate(_ rate: Float, at time: CMTime) {
        synchronizer.setRate(rate, time: time)
    }

    func setVolume(_ volume: Float) {
        audioRenderer.volume = volume
    }

    func seek(to position: TimeInterval, completion: @escaping @Sendable () -> Void) {
        synchronizer.setRate(0, time: synchronizer.currentTime())
        displayLayer.flush()
        audioRenderer.flush()
        store.reset()

        demuxQueue.async { [self] in
            demuxer.seek(to: position)
            audioDecoder.flush()
            fill()
            completion()
        }
    }

    func selectAudioStream(index: Int32) {
        demuxQueue.async { [self] in
            guard let stream = demuxer.streams.first(where: { $0.index == index }) else { return }
            demuxer.selectAudioStream(index: index)
            _ = audioDecoder.open(stream)
        }
    }

    func close() {
        synchronizer.setRate(0, time: .zero)
        displayLayer.stopRequestingMediaData()
        audioRenderer.stopRequestingMediaData()
        displayLayer.flush()
        audioRenderer.flush()
        store.reset()
        started = false
        demuxQueue.async { [self] in
            videoFormatDescription = nil
            audioDecoder.close()
            demuxer.close()
        }
    }

    private func requestFill() {
        guard store.needsFill, !store.reachedEnd else { return }
        demuxQueue.async { [self] in fill() }
    }

    private func fill() {
        while store.needsFill {
            guard let packet = demuxer.readPacket() else {
                store.reachedEnd = true
                return
            }
            if packet.streamIndex == demuxer.videoStreamIndex {
                appendVideo(packet)
            } else {
                store.appendAudio(audioDecoder.decode(packet))
            }
        }
    }

    private func appendVideo(_ packet: FluxaFFmpegPacket) {
        guard let videoFormatDescription else { return }
        let payload = videoNeedsLengthPrefix
            ? FluxaSampleBufferFactory.lengthPrefixed(annexB: packet.data)
            : packet.data
        guard let buffer = FluxaSampleBufferFactory.sampleBuffer(
            data: payload,
            formatDescription: videoFormatDescription,
            presentationTime: packet.pts,
            decodeTime: packet.dts,
            duration: packet.duration
        ) else { return }
        store.appendVideo(buffer)
    }
}

final class FluxaFFmpegBufferStore: @unchecked Sendable {
    private let lock = NSLock()
    private var video: [CMSampleBuffer] = []
    private var audio: [CMSampleBuffer] = []
    private var endReached = false
    private let highWaterMark = 240

    var reachedEnd: Bool {
        get { lock.withLock { endReached } }
        set { lock.withLock { endReached = newValue } }
    }

    var needsFill: Bool {
        lock.withLock { !endReached && (video.count < highWaterMark || audio.count < highWaterMark) }
    }

    var isEmpty: Bool {
        lock.withLock { video.isEmpty && audio.isEmpty }
    }

    func appendVideo(_ buffer: CMSampleBuffer) {
        lock.withLock { video.append(buffer) }
    }

    func appendAudio(_ newBuffers: [CMSampleBuffer]) {
        guard !newBuffers.isEmpty else { return }
        lock.withLock { audio.append(contentsOf: newBuffers) }
    }

    func nextVideo() -> CMSampleBuffer? {
        lock.withLock { video.isEmpty ? nil : video.removeFirst() }
    }

    func nextAudio() -> CMSampleBuffer? {
        lock.withLock { audio.isEmpty ? nil : audio.removeFirst() }
    }

    func reset() {
        lock.withLock {
            video.removeAll()
            audio.removeAll()
            endReached = false
        }
    }
}
#endif
