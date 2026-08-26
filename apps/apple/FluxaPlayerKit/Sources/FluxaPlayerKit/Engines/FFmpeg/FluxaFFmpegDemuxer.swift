#if FLUXA_FFMPEG
import CFFmpeg
import CoreMedia
import Foundation

struct FluxaFFmpegStream {
    let index: Int32
    let kind: FluxaTrackKind?
    let codecID: AVCodecID
    let timeBase: AVRational
    let extradata: Data
    let language: String?
    let title: String?
    let isDefault: Bool
    let isForced: Bool
    let parameters: UnsafeMutablePointer<AVCodecParameters>
}

struct FluxaFFmpegPacket {
    let streamIndex: Int32
    let data: Data
    let pts: CMTime
    let dts: CMTime
    let duration: CMTime
    let isKeyframe: Bool
}

enum FluxaFFmpegError: Error {
    case openFailed(Int32)
    case noStreams
}

final class FluxaFFmpegDemuxer {
    private(set) var streams: [FluxaFFmpegStream] = []
    private(set) var duration: TimeInterval = 0
    private(set) var videoStreamIndex: Int32 = -1
    private(set) var audioStreamIndex: Int32 = -1

    private var formatContext: UnsafeMutablePointer<AVFormatContext>?
    private let packet: UnsafeMutablePointer<AVPacket>

    init() {
        packet = av_packet_alloc()
    }

    deinit {
        var mutablePacket: UnsafeMutablePointer<AVPacket>? = packet
        av_packet_free(&mutablePacket)
        close()
    }

    func open(url: URL, headers: [String: String]) throws {
        var context: UnsafeMutablePointer<AVFormatContext>? = avformat_alloc_context()
        var options: OpaquePointer? = nil
        defer { av_dict_free(&options) }

        if !headers.isEmpty {
            let joined = headers.map { "\($0.key): \($0.value)\r\n" }.joined()
            av_dict_set(&options, "headers", joined, 0)
        }
        av_dict_set(&options, "reconnect", "1", 0)
        av_dict_set(&options, "reconnect_streamed", "1", 0)

        let path = url.isFileURL ? url.path : url.absoluteString
        let status = avformat_open_input(&context, path, nil, &options)
        guard status >= 0, let context else { throw FluxaFFmpegError.openFailed(status) }

        let infoStatus = avformat_find_stream_info(context, nil)
        guard infoStatus >= 0 else {
            var mutable: UnsafeMutablePointer<AVFormatContext>? = context
            avformat_close_input(&mutable)
            throw FluxaFFmpegError.openFailed(infoStatus)
        }

        formatContext = context
        let rawDuration = context.pointee.duration
        duration = rawDuration > 0 ? TimeInterval(rawDuration) / 1_000_000 : 0
        collectStreams(context)
        guard !streams.isEmpty else { throw FluxaFFmpegError.noStreams }
    }

    func close() {
        guard formatContext != nil else { return }
        avformat_close_input(&formatContext)
        formatContext = nil
        streams = []
        videoStreamIndex = -1
        audioStreamIndex = -1
    }

    func selectAudioStream(index: Int32) {
        guard streams.contains(where: { $0.index == index && $0.kind == .audio }) else { return }
        audioStreamIndex = index
    }

    func seek(to seconds: TimeInterval) {
        guard let formatContext else { return }
        let target = Int64(max(0, seconds) * 1_000_000)
        av_seek_frame(formatContext, -1, target, AVSEEK_FLAG_BACKWARD)
    }

    func readPacket() -> FluxaFFmpegPacket? {
        guard let formatContext else { return nil }
        while av_read_frame(formatContext, packet) >= 0 {
            defer { av_packet_unref(packet) }
            let index = packet.pointee.stream_index
            guard index == videoStreamIndex || index == audioStreamIndex,
                  let stream = streams.first(where: { $0.index == index }),
                  let bytes = packet.pointee.data else { continue }

            let timeBase = stream.timeBase
            return FluxaFFmpegPacket(
                streamIndex: index,
                data: Data(bytes: bytes, count: Int(packet.pointee.size)),
                pts: time(packet.pointee.pts, timeBase),
                dts: time(packet.pointee.dts, timeBase),
                duration: time(packet.pointee.duration, timeBase),
                isKeyframe: packet.pointee.flags & AV_PKT_FLAG_KEY != 0
            )
        }
        return nil
    }

    private func collectStreams(_ context: UnsafeMutablePointer<AVFormatContext>) {
        var collected: [FluxaFFmpegStream] = []
        let count = Int(context.pointee.nb_streams)
        for position in 0..<count {
            guard let stream = context.pointee.streams[position] else { continue }
            guard let parameters = stream.pointee.codecpar else { continue }

            let kind: FluxaTrackKind?
            switch parameters.pointee.codec_type {
            case AVMEDIA_TYPE_AUDIO: kind = .audio
            case AVMEDIA_TYPE_SUBTITLE: kind = .subtitle
            default: kind = nil
            }

            var extradata = Data()
            if let raw = parameters.pointee.extradata, parameters.pointee.extradata_size > 0 {
                extradata = Data(bytes: raw, count: Int(parameters.pointee.extradata_size))
            }

            let disposition = stream.pointee.disposition
            collected.append(
                FluxaFFmpegStream(
                    index: stream.pointee.index,
                    kind: kind,
                    codecID: parameters.pointee.codec_id,
                    timeBase: stream.pointee.time_base,
                    extradata: extradata,
                    language: metadata(stream.pointee.metadata, key: "language"),
                    title: metadata(stream.pointee.metadata, key: "title"),
                    isDefault: disposition & AV_DISPOSITION_DEFAULT != 0,
                    isForced: disposition & AV_DISPOSITION_FORCED != 0,
                    parameters: parameters
                )
            )

            if parameters.pointee.codec_type == AVMEDIA_TYPE_VIDEO, videoStreamIndex < 0 {
                videoStreamIndex = stream.pointee.index
            }
            if parameters.pointee.codec_type == AVMEDIA_TYPE_AUDIO, audioStreamIndex < 0 {
                audioStreamIndex = stream.pointee.index
            }
        }
        streams = collected
    }

    private func metadata(_ dictionary: OpaquePointer?, key: String) -> String? {
        guard let entry = av_dict_get(dictionary, key, nil, 0), let value = entry.pointee.value else { return nil }
        return String(cString: value)
    }

    private func time(_ value: Int64, _ timeBase: AVRational) -> CMTime {
        guard value != Int64.min, timeBase.den != 0 else { return .invalid }
        return CMTime(
            value: CMTimeValue(value * Int64(timeBase.num)),
            timescale: CMTimeScale(timeBase.den)
        )
    }
}
#endif
