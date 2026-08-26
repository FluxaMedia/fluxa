#if FLUXA_FFMPEG
import CFFmpeg
import CoreMedia
import Foundation

enum FluxaSampleBufferFactory {
    static func videoFormatDescription(for stream: FluxaFFmpegStream) -> CMVideoFormatDescription? {
        let parameters = stream.parameters.pointee
        let width = Int32(parameters.width)
        let height = Int32(parameters.height)
        guard width > 0, height > 0 else { return nil }

        let atomKey: String
        let codecType: CMVideoCodecType
        switch stream.codecID {
        case AV_CODEC_ID_H264:
            atomKey = "avcC"
            codecType = kCMVideoCodecType_H264
        case AV_CODEC_ID_HEVC:
            atomKey = "hvcC"
            codecType = kCMVideoCodecType_HEVC
        default:
            return nil
        }

        guard !stream.extradata.isEmpty else { return nil }
        guard isLengthPrefixedExtradata(stream.extradata) else {
            return parameterSetDescription(codecID: stream.codecID, annexB: stream.extradata)
        }

        let extensions: [CFString: Any] = [
            kCMFormatDescriptionExtension_SampleDescriptionExtensionAtoms: [atomKey: stream.extradata]
        ]

        var description: CMVideoFormatDescription?
        let status = CMVideoFormatDescriptionCreate(
            allocator: kCFAllocatorDefault,
            codecType: codecType,
            width: width,
            height: height,
            extensions: extensions as CFDictionary,
            formatDescriptionOut: &description
        )
        return status == noErr ? description : nil
    }

    static func parameterSetDescription(codecID: AVCodecID, annexB: Data) -> CMVideoFormatDescription? {
        let units = annexBUnits(annexB)
        guard !units.isEmpty else { return nil }

        var description: CMVideoFormatDescription?
        let pointers = units.map { UnsafePointer<UInt8>(($0 as NSData).bytes.assumingMemoryBound(to: UInt8.self)) }
        let sizes = units.map { $0.count }

        let status: OSStatus
        switch codecID {
        case AV_CODEC_ID_H264:
            status = CMVideoFormatDescriptionCreateFromH264ParameterSets(
                allocator: kCFAllocatorDefault,
                parameterSetCount: pointers.count,
                parameterSetPointers: pointers,
                parameterSetSizes: sizes,
                nalUnitHeaderLength: 4,
                formatDescriptionOut: &description
            )
        case AV_CODEC_ID_HEVC:
            status = CMVideoFormatDescriptionCreateFromHEVCParameterSets(
                allocator: kCFAllocatorDefault,
                parameterSetCount: pointers.count,
                parameterSetPointers: pointers,
                parameterSetSizes: sizes,
                nalUnitHeaderLength: 4,
                extensions: nil,
                formatDescriptionOut: &description
            )
        default:
            return nil
        }
        return status == noErr ? description : nil
    }

    private static func annexBUnits(_ data: Data) -> [Data] {
        var units: [Data] = []
        var cursor = 0
        var unitStart = -1
        let bytes = [UInt8](data)
        while cursor + 2 < bytes.count {
            if bytes[cursor] == 0, bytes[cursor + 1] == 0, bytes[cursor + 2] == 1 {
                if unitStart >= 0 {
                    var end = cursor
                    if end > unitStart, bytes[end - 1] == 0 { end -= 1 }
                    if end > unitStart { units.append(Data(bytes[unitStart..<end])) }
                }
                cursor += 3
                unitStart = cursor
            } else {
                cursor += 1
            }
        }
        if unitStart >= 0, unitStart < bytes.count {
            units.append(Data(bytes[unitStart...]))
        }
        return units
    }

    static func nalLengthSize(for stream: FluxaFFmpegStream) -> Int {
        guard isLengthPrefixedExtradata(stream.extradata) else { return 0 }
        switch stream.codecID {
        case AV_CODEC_ID_H264:
            guard stream.extradata.count > 4 else { return 4 }
            return Int(stream.extradata[4] & 0x03) + 1
        case AV_CODEC_ID_HEVC:
            guard stream.extradata.count > 21 else { return 4 }
            return Int(stream.extradata[21] & 0x03) + 1
        default:
            return 4
        }
    }

    static func sampleBuffer(
        data: Data,
        formatDescription: CMFormatDescription,
        presentationTime: CMTime,
        decodeTime: CMTime,
        duration: CMTime
    ) -> CMSampleBuffer? {
        var blockBuffer: CMBlockBuffer?
        var payload = data

        let created = payload.withUnsafeMutableBytes { raw -> OSStatus in
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

        var copied: CMBlockBuffer?
        guard CMBlockBufferCreateContiguous(
            allocator: kCFAllocatorDefault,
            sourceBuffer: blockBuffer,
            blockAllocator: kCFAllocatorDefault,
            customBlockSource: nil,
            offsetToData: 0,
            dataLength: 0,
            flags: kCMBlockBufferAlwaysCopyDataFlag,
            blockBufferOut: &copied
        ) == noErr, let copied else { return nil }

        var timing = CMSampleTimingInfo(
            duration: duration.isValid ? duration : .invalid,
            presentationTimeStamp: presentationTime,
            decodeTimeStamp: decodeTime
        )
        var sampleSize = data.count
        var sampleBuffer: CMSampleBuffer?
        let status = CMSampleBufferCreateReady(
            allocator: kCFAllocatorDefault,
            dataBuffer: copied,
            formatDescription: formatDescription,
            sampleCount: 1,
            sampleTimingEntryCount: 1,
            sampleTimingArray: &timing,
            sampleSizeEntryCount: 1,
            sampleSizeArray: &sampleSize,
            sampleBufferOut: &sampleBuffer
        )
        return status == noErr ? sampleBuffer : nil
    }

    static func lengthPrefixed(annexB: Data) -> Data {
        var output = Data()
        var ranges: [Range<Int>] = []
        var cursor = 0
        var unitStart = -1

        annexB.withUnsafeBytes { raw in
            let bytes = raw.bindMemory(to: UInt8.self)
            while cursor + 2 < bytes.count {
                if bytes[cursor] == 0, bytes[cursor + 1] == 0, bytes[cursor + 2] == 1 {
                    if unitStart >= 0 {
                        var end = cursor
                        if end > unitStart, bytes[end - 1] == 0 { end -= 1 }
                        ranges.append(unitStart..<end)
                    }
                    cursor += 3
                    unitStart = cursor
                } else {
                    cursor += 1
                }
            }
            if unitStart >= 0, unitStart < bytes.count {
                ranges.append(unitStart..<bytes.count)
            }
        }

        for range in ranges where !range.isEmpty {
            var length = UInt32(range.count).bigEndian
            withUnsafeBytes(of: &length) { output.append(contentsOf: $0) }
            output.append(annexB.subdata(in: range))
        }
        return output.isEmpty ? annexB : output
    }

    static func isLengthPrefixedExtradata(_ extradata: Data) -> Bool {
        guard extradata.count > 3 else { return false }
        if extradata[0] == 0, extradata[1] == 0, extradata[2] == 0 || extradata[2] == 1 { return false }
        return extradata[0] == 1
    }
}
#endif
