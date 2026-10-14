import AVKit
import CoreMedia
import UIKit

@MainActor
final class FluxaPictureInPicture: NSObject {
    var status: (paused: Bool, position: Double, duration: Double) = (true, 0, 0)

    private let layer: AVSampleBufferDisplayLayer
    private let send: (String, Double) -> Void
    private var controller: AVPictureInPictureController?

    init(layer: AVSampleBufferDisplayLayer, send: @escaping (String, Double) -> Void) {
        self.layer = layer
        self.send = send
    }

    func update(state: Int32) {
        #if os(iOS)
        guard AVPictureInPictureController.isPictureInPictureSupported() else { return }
        if state == 0 {
            controller?.canStartPictureInPictureAutomaticallyFromInline = false
            return
        }
        if controller == nil {
            if layer.controlTimebase == nil {
                var timebase: CMTimebase?
                CMTimebaseCreateWithSourceClock(allocator: kCFAllocatorDefault, sourceClock: CMClockGetHostTimeClock(), timebaseOut: &timebase)
                layer.controlTimebase = timebase
            }
            let source = AVPictureInPictureController.ContentSource(
                sampleBufferDisplayLayer: layer,
                playbackDelegate: self
            )
            controller = AVPictureInPictureController(contentSource: source)
        }
        controller?.canStartPictureInPictureAutomaticallyFromInline = true
        controller?.invalidatePlaybackState()
        #endif
    }
}

#if os(iOS)
extension FluxaPictureInPicture: AVPictureInPictureSampleBufferPlaybackDelegate {
    nonisolated func pictureInPictureController(_ controller: AVPictureInPictureController, setPlaying playing: Bool) {
        MainActor.assumeIsolated {
            send(playing ? "play" : "pause", 0)
        }
    }

    nonisolated func pictureInPictureControllerTimeRangeForPlayback(_ controller: AVPictureInPictureController) -> CMTimeRange {
        MainActor.assumeIsolated {
            CMTimeRange(start: .zero, duration: CMTime(seconds: max(status.duration, 1), preferredTimescale: 600))
        }
    }

    nonisolated func pictureInPictureControllerIsPlaybackPaused(_ controller: AVPictureInPictureController) -> Bool {
        MainActor.assumeIsolated { status.paused }
    }

    nonisolated func pictureInPictureController(_ controller: AVPictureInPictureController, didTransitionToRenderSize newRenderSize: CMVideoDimensions) {}

    nonisolated func pictureInPictureController(
        _ controller: AVPictureInPictureController,
        skipByInterval skipInterval: CMTime,
        completion completionHandler: @escaping @Sendable () -> Void
    ) {
        MainActor.assumeIsolated { send(skipInterval.seconds < 0 ? "seekBackward" : "seekForward", abs(skipInterval.seconds)) }
        completionHandler()
    }
}
#endif
