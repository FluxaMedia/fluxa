#if canImport(MediaPlayer)
import MediaPlayer

@MainActor
final class FluxaNowPlaying {
    func begin(title: String) {
        var info = MPNowPlayingInfoCenter.default().nowPlayingInfo ?? [:]
        info[MPMediaItemPropertyTitle] = title
        MPNowPlayingInfoCenter.default().nowPlayingInfo = info
    }

    func update(_ state: FluxaPlaybackState) {
        var info = MPNowPlayingInfoCenter.default().nowPlayingInfo ?? [:]
        if state.duration > 0 {
            info[MPMediaItemPropertyPlaybackDuration] = state.duration
        }
        info[MPNowPlayingInfoPropertyElapsedPlaybackTime] = state.position
        info[MPNowPlayingInfoPropertyPlaybackRate] = state.isPlaying ? state.rate : 0
        MPNowPlayingInfoCenter.default().nowPlayingInfo = info

        switch state.phase {
        case .playing:
            MPNowPlayingInfoCenter.default().playbackState = .playing
        case .paused, .ended, .failed(_):
            MPNowPlayingInfoCenter.default().playbackState = .paused
        case .idle, .loading:
            break
        }
    }

    func clear() {
        MPNowPlayingInfoCenter.default().nowPlayingInfo = nil
        MPNowPlayingInfoCenter.default().playbackState = .stopped
    }
}
#endif
