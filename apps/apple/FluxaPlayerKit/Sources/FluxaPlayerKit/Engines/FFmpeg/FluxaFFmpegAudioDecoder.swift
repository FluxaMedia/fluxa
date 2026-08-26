#if FLUXA_FFMPEG
import CFFmpeg
import CoreMedia
import Foundation

final class FluxaFFmpegAudioDecoder {
    private var codecContext: UnsafeMutablePointer<AVCodecContext>?
    private var resampler: OpaquePointer?
    private var frame: UnsafeMutablePointer<AVFrame>?
    private var packet: UnsafeMutablePointer<AVPacket>?
    private var formatDescription: CMAudioFormatDescription?
    private var outputSampleRate: Int32 = 48000
    private var outputChannels: Int32 = 2
    private var inputTimeBase: AVRational?

    func open(_ stream: FluxaFFmpegStream) -> Bool {
        close()
        guard let codec = avcodec_find_decoder(stream.codecID),
              let context = avcodec_alloc_context3(codec) else { return false }

        var mutableContext: UnsafeMutablePointer<AVCodecContext>? = context
        guard avcodec_parameters_to_context(context, stream.parameters) >= 0,
              avcodec_open2(context, codec, nil) >= 0 else {
            avcodec_free_context(&mutableContext)
            return false
        }

        codecContext = context
        inputTimeBase = stream.timeBase
        frame = av_frame_alloc()
        packet = av_packet_alloc()
        outputSampleRate = context.pointee.sample_rate > 0 ? context.pointee.sample_rate : 48000
        outputChannels = min(max(context.pointee.ch_layout.nb_channels, 1), 8)

        guard configureResampler(context) else {
            close()
            return false
        }
        formatDescription = makeFormatDescription()
        return formatDescription != nil
    }

    func close() {
        if var resampler { swr_free(&resampler) }
        resampler = nil
        if frame != nil { av_frame_free(&frame) }
        if packet != nil { av_packet_free(&packet) }
        if codecContext != nil { avcodec_free_context(&codecContext) }
        formatDescription = nil
        inputTimeBase = nil
    }

    func flush() {
        guard let codecContext else { return }
        avcodec_flush_buffers(codecContext)
    }

    func decode(_ input: FluxaFFmpegPacket) -> [CMSampleBuffer] {
        guard let codecContext, let frame, let packet, let formatDescription, let resampler else { return [] }

        var payload = input.data
        let sent = payload.withUnsafeMutableBytes { raw -> Int32 in
            guard let base = raw.baseAddress else { return -1 }
            packet.pointee.data = base.assumingMemoryBound(to: UInt8.self)
            packet.pointee.size = Int32(raw.count)
            packet.pointee.pts = input.pts.isValid ? input.pts.value : Int64.min
            return avcodec_send_packet(codecContext, packet)
        }
        packet.pointee.data = nil
        packet.pointee.size = 0
        guard sent >= 0 else { return [] }

        var buffers: [CMSampleBuffer] = []
        while avcodec_receive_frame(codecContext, frame) >= 0 {
            defer { av_frame_unref(frame) }
            guard let buffer = resample(frame, resampler: resampler, formatDescription: formatDescription, fallbackTime: input.pts) else { continue }
            buffers.append(buffer)
        }
        return buffers
    }

    private func configureResampler(_ context: UnsafeMutablePointer<AVCodecContext>) -> Bool {
        var swr: OpaquePointer? = nil
        var outLayout = AVChannelLayout()
        av_channel_layout_default(&outLayout, outputChannels)
        var inLayout = context.pointee.ch_layout

        let status = swr_alloc_set_opts2(
            &swr,
            &outLayout,
            AV_SAMPLE_FMT_FLT,
            outputSampleRate,
            &inLayout,
            context.pointee.sample_fmt,
            context.pointee.sample_rate,
            0,
            nil
        )
        guard status >= 0, let swr, swr_init(swr) >= 0 else {
            if var swr { swr_free(&swr) }
            return false
        }
        resampler = swr
        return true
    }

    private func makeFormatDescription() -> CMAudioFormatDescription? {
        var asbd = AudioStreamBasicDescription(
            mSampleRate: Float64(outputSampleRate),
            mFormatID: kAudioFormatLinearPCM,
            mFormatFlags: kAudioFormatFlagIsFloat | kAudioFormatFlagIsPacked,
            mBytesPerPacket: UInt32(4 * outputChannels),
            mFramesPerPacket: 1,
            mBytesPerFrame: UInt32(4 * outputChannels),
            mChannelsPerFrame: UInt32(outputChannels),
            mBitsPerChannel: 32,
            mReserved: 0
        )
        var description: CMAudioFormatDescription?
        let status = CMAudioFormatDescriptionCreate(
            allocator: kCFAllocatorDefault,
            asbd: &asbd,
            layoutSize: 0,
            layout: nil,
            magicCookieSize: 0,
            magicCookie: nil,
            extensions: nil,
            formatDescriptionOut: &description
        )
        return status == noErr ? description : nil
    }

    private func resample(
        _ frame: UnsafeMutablePointer<AVFrame>,
        resampler: OpaquePointer,
        formatDescription: CMAudioFormatDescription,
        fallbackTime: CMTime
    ) -> CMSampleBuffer? {
        let maxSamples = swr_get_out_samples(resampler, frame.pointee.nb_samples)
        guard maxSamples > 0 else { return nil }

        let bytesPerFrame = Int(4 * outputChannels)
        var storage = Data(count: Int(maxSamples) * bytesPerFrame)

        let converted = storage.withUnsafeMutableBytes { raw -> Int32 in
            guard let base = raw.baseAddress else { return -1 }
            var output: UnsafeMutablePointer<UInt8>? = base.assumingMemoryBound(to: UInt8.self)
            return withUnsafePointer(to: &output) { outputPointer in
                swr_convert(
                    resampler,
                    UnsafeMutablePointer(mutating: outputPointer),
                    maxSamples,
                    UnsafePointer(frame.pointee.extended_data),
                    frame.pointee.nb_samples
                )
            }
        }
        guard converted > 0 else { return nil }
        let usedBytes = Int(converted) * bytesPerFrame
        if usedBytes < storage.count {
            storage.removeSubrange(usedBytes..<storage.count)
        }

        let presentationTime: CMTime
        if frame.pointee.pts != Int64.min,
           let inputTimeBase,
           inputTimeBase.den != 0 {
            presentationTime = CMTime(
                value: frame.pointee.pts * Int64(inputTimeBase.num),
                timescale: CMTimeScale(inputTimeBase.den)
            )
        } else {
            presentationTime = fallbackTime
        }

        var blockBuffer: CMBlockBuffer?
        let created = storage.withUnsafeMutableBytes { raw -> OSStatus in
            guard let base = raw.baseAddress else { return -1 }
            return CMBlockBufferCreateWithMemoryBlock(
                allocator: kCFAllocatorDefault,
                memoryBlock: base,
                blockLength: raw.count,
                blockAllocator: kCFAllocatorNull,
                customBlockSource: nil,
                offsetToData: 0,
                dataLength: raw.count,
                flags: 0,
                blockBufferOut: &blockBuffer
            )
        }
        guard created == noErr, let blockBuffer else { return nil }

        var contiguous: CMBlockBuffer?
        guard CMBlockBufferCreateContiguous(
            allocator: kCFAllocatorDefault,
            sourceBuffer: blockBuffer,
            blockAllocator: kCFAllocatorDefault,
            customBlockSource: nil,
            offsetToData: 0,
            dataLength: 0,
            flags: kCMBlockBufferAlwaysCopyDataFlag,
            blockBufferOut: &contiguous
        ) == noErr, let contiguous else { return nil }

        var timing = CMSampleTimingInfo(
            duration: CMTime(
                value: CMTimeValue(converted),
                timescale: CMTimeScale(outputSampleRate)
            ),
            presentationTimeStamp: presentationTime,
            decodeTimeStamp: .invalid
        )
        var sampleBuffer: CMSampleBuffer?
        let status = CMSampleBufferCreateReady(
            allocator: kCFAllocatorDefault,
            dataBuffer: contiguous,
            formatDescription: formatDescription,
            sampleCount: CMItemCount(converted),
            sampleTimingEntryCount: 1,
            sampleTimingArray: &timing,
            sampleSizeEntryCount: 1,
            sampleSizeArray: [bytesPerFrame],
            sampleBufferOut: &sampleBuffer
        )
        return status == noErr ? sampleBuffer : nil
    }

    deinit { close() }
}
#endif
