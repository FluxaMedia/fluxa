import AVFoundation
import FluxaRendererFFI
import Libmpv
import QuartzCore
import UIKit

@MainActor
final class FluxaNativeVideo {
    let layer = CAMetalLayer()
    var onPlayingChanged: ((Bool) -> Void)?

    private var mpv: OpaquePointer?
    private var audioLanguage = ""
    private var subtitleLanguage = ""
    private var customOptions = ""
    private var audioProcessingMode = ""
    private var loaded = false
    private var error: String?

    init() {
        layer.backgroundColor = UIColor.black.cgColor
        layer.isHidden = true
    }

    func tick() {
        if let value = fluxa_renderer_poll_video() {
            let json = String(cString: value)
            fluxa_renderer_string_free(value)
            let requests = (try? JSONSerialization.jsonObject(with: Data(json.utf8))) as? [[String: Any]] ?? []
            requests.forEach(handle)
        }
        drainEvents()
        report()
    }

    private func handle(_ request: [String: Any]) {
        switch request["type"] as? String {
        case "configure":
            audioLanguage = request["audioLanguage"] as? String ?? ""
            subtitleLanguage = request["subtitleLanguage"] as? String ?? ""
            customOptions = request["mpvOptions"] as? String ?? ""
            audioProcessingMode = request["audioProcessingMode"] as? String ?? ""
        case "load":
            guard let url = request["url"] as? String else { return }
            load(url)
        case "stop":
            stop()
        case "togglePause":
            command("cycle", "pause")
        case "seek":
            guard let seconds = request["seconds"] as? Double else { return }
            command("seek", String(seconds), "relative")
        case "seekTo":
            guard let seconds = request["seconds"] as? Double else { return }
            command("seek", String(max(0, seconds)), "absolute")
        case "toggleMute":
            command("cycle", "mute")
        default:
            break
        }
    }

    private func create() -> OpaquePointer? {
        guard let mpv = mpv_create() else { return nil }
        let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("mpv")
        let cache = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0]
            .appendingPathComponent("mpv")
        try? FileManager.default.createDirectory(at: support, withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(at: cache, withIntermediateDirectories: true)
        var wid = Int64(Int(bitPattern: Unmanaged.passUnretained(layer).toOpaque()))
        mpv_set_option(mpv, "wid", MPV_FORMAT_INT64, &wid)
        let options = [
            ("vo", "gpu-next"),
            ("gpu-api", "vulkan"),
            ("gpu-context", "moltenvk"),
            ("hwdec", "videotoolbox"),
            ("ao", "audiounit"),
            ("config", "yes"),
            ("config-dir", support.path),
            ("gpu-shader-cache-dir", cache.path),
            ("icc-cache-dir", cache.path),
            ("video-sync", "audio"),
            ("cache", "yes"),
            ("cache-secs", "60"),
            ("demuxer-max-bytes", "96MiB"),
            ("demuxer-lavf-o", "allowed_extensions=ALL"),
            ("network-timeout", "15"),
            ("keep-open", "no"),
            ("sub-auto", "fuzzy"),
            ("sub-ass", "yes"),
            ("sub-ass-override", "scale"),
            ("audio-display", "no"),
            ("idle", "once"),
        ]
        options.forEach { mpv_set_option_string(mpv, $0.0, $0.1) }
        for line in customOptions.split(whereSeparator: \.isNewline) {
            let line = line.trimmingCharacters(in: .whitespaces)
            guard !line.isEmpty, !line.hasPrefix("#"), let eq = line.firstIndex(of: "=") else { continue }
            let key = line[..<eq].trimmingCharacters(in: .whitespaces)
            if !["audio-spdif", "audio-channels", "af"].contains(key) {
                mpv_set_option_string(mpv, key, line[line.index(after: eq)...].trimmingCharacters(in: .whitespaces))
            }
        }
        if let filter = filters[audioProcessingMode] {
            mpv_set_option_string(mpv, "af", filter)
        }
        guard mpv_initialize(mpv) >= 0 else {
            mpv_terminate_destroy(mpv)
            return nil
        }
        mpv_request_log_messages(mpv, "error")
        return mpv
    }

    private func load(_ url: String) {
        if mpv == nil {
            mpv = create()
        }
        guard let mpv else {
            error = "libmpv could not be created"
            return
        }
        try? AVAudioSession.sharedInstance().setCategory(.playback, mode: .moviePlayback)
        try? AVAudioSession.sharedInstance().setActive(true)
        mpv_set_property_string(mpv, "alang", audioLanguage)
        mpv_set_property_string(mpv, "slang", subtitleLanguage)
        if url.hasPrefix("http://127.0.0.1:") {
            mpv_set_property_string(mpv, "network-timeout", "90")
        }
        error = nil
        loaded = true
        layer.isHidden = false
        command("loadfile", url, "replace")
        mpv_set_property_string(mpv, "pause", "no")
        onPlayingChanged?(true)
    }

    private func stop() {
        guard let mpv else { return }
        command("stop")
        mpv_terminate_destroy(mpv)
        self.mpv = nil
        loaded = false
        layer.isHidden = true
        onPlayingChanged?(false)
    }

    private func command(_ args: String...) {
        guard let mpv else { return }
        var cargs = args.map { strdup($0) }
        defer { cargs.forEach { free($0) } }
        cargs.append(nil)
        cargs.withUnsafeMutableBufferPointer { buffer in
            buffer.baseAddress!.withMemoryRebound(to: UnsafePointer<CChar>?.self, capacity: buffer.count) {
                _ = mpv_command(mpv, $0)
            }
        }
    }

    private func drainEvents() {
        guard let mpv else { return }
        while let event = mpv_wait_event(mpv, 0), event.pointee.event_id != MPV_EVENT_NONE {
            switch event.pointee.event_id {
            case MPV_EVENT_LOG_MESSAGE:
                let message = event.pointee.data.assumingMemoryBound(to: mpv_event_log_message.self).pointee
                if String(cString: message.prefix) == "cplayer" {
                    error = String(String(cString: message.text).trimmingCharacters(in: .whitespacesAndNewlines).prefix(180))
                }
            case MPV_EVENT_END_FILE:
                let end = event.pointee.data.assumingMemoryBound(to: mpv_event_end_file.self).pointee
                if end.reason == MPV_END_FILE_REASON_ERROR {
                    error = String(cString: mpv_error_string(end.error))
                }
            default:
                break
            }
        }
    }

    private func double(_ name: String) -> Double {
        var value = 0.0
        guard let mpv, mpv_get_property(mpv, name, MPV_FORMAT_DOUBLE, &value) >= 0 else { return 0 }
        return value
    }

    private func flag(_ name: String) -> Bool {
        var value: Int32 = 0
        guard let mpv, mpv_get_property(mpv, name, MPV_FORMAT_FLAG, &value) >= 0 else { return false }
        return value != 0
    }

    private func report() {
        guard mpv != nil, loaded else { return }
        let paused = flag("pause")
        let buffering = flag("paused-for-cache") || (flag("core-idle") && !paused)
        fluxa_renderer_video_status(
            double("time-pos"),
            double("duration"),
            paused,
            double("video-params/w") > 0,
            buffering ? 0 : -1,
            error
        )
    }
}

private let filters = [
    "balanced": "lavfi=[acompressor=threshold=0.78:ratio=1.5:attack=30:release=300:link=maximum,alimiter=limit=0.98]",
    "night": "lavfi=[acompressor=threshold=0.55:ratio=3:attack=20:release=250:link=maximum,alimiter=limit=0.98]",
]
