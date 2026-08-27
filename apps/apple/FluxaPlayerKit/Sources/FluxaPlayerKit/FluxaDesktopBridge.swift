#if os(macOS)
import AppKit
import Darwin
import Foundation
import WebKit

// C ABI used by the Tauri macOS shell. The player remains owned by
// FluxaPlayerKit; Rust only holds an opaque handle and forwards commands.
@MainActor
private final class FluxaDesktopPlayerHandle {
    let player: FluxaPlayer
    let surface: FluxaPlayerSurfaceView

    init(parentPointer: UnsafeMutableRawPointer, width: Double, height: Double) {
        let parent = Unmanaged<NSView>.fromOpaque(parentPointer).takeUnretainedValue()
        let frame = NSRect(x: 0, y: 0, width: max(1, width), height: max(1, height))
        let surface = FluxaPlayerSurfaceView(frame: frame)
        surface.autoresizingMask = [.width, .height]

        let contentView = parent.window?.contentView ?? parent
        var relativeView = parent
        while let superview = relativeView.superview, superview !== contentView {
            relativeView = superview
        }
        contentView.addSubview(surface, positioned: .below, relativeTo: relativeView)
        surface.isHidden = true

        if let webView = parent as? WKWebView {
            webView.setValue(false, forKey: "drawsBackground")
            webView.underPageBackgroundColor = .clear
        }
        parent.window?.isOpaque = false

        self.surface = surface
        player = FluxaPlayer()
    }

    func load(url: String, title: String, start: Double, subtitles: [URL] = []) throws {
        guard let mediaURL = URL(string: url) else {
            throw NSError(domain: "FluxaDesktopPlayer", code: 1, userInfo: [NSLocalizedDescriptionKey: "Invalid media URL"])
        }
        surface.isHidden = false
        player.attach(to: surface)
        player.load(FluxaPlaybackItem(
            url: mediaURL,
            title: title,
            startPosition: max(0, start),
            subtitleUrls: subtitles
        ))
        player.play()
    }
}

private func onMain<T>(_ body: @MainActor () throws -> T) rethrows -> T {
    if Thread.isMainThread {
        return try MainActor.assumeIsolated(body)
    }
    return try DispatchQueue.main.sync {
        try MainActor.assumeIsolated(body)
    }
}

private func handle(_ pointer: UnsafeMutableRawPointer?) -> FluxaDesktopPlayerHandle? {
    guard let pointer else { return nil }
    return Unmanaged<FluxaDesktopPlayerHandle>.fromOpaque(pointer).takeUnretainedValue()
}

private func cString(_ pointer: UnsafePointer<CChar>?) -> String {
    guard let pointer else { return "" }
    return String(cString: pointer)
}

private func splitCommand(_ command: String) -> [Substring] {
    command.split(whereSeparator: { $0 == " " || $0 == "\t" })
}

@_cdecl("fluxa_desktop_avplayer_create")
public func fluxaDesktopAvplayerCreate(
    _ parent: UnsafeMutableRawPointer?,
    _ width: Double,
    _ height: Double
) -> UnsafeMutableRawPointer? {
    guard let parent else { return nil }
    return onMain {
        let handle = FluxaDesktopPlayerHandle(parentPointer: parent, width: width, height: height)
        return Unmanaged.passRetained(handle).toOpaque()
    }
}

@_cdecl("fluxa_desktop_avplayer_destroy")
public func fluxaDesktopAvplayerDestroy(_ pointer: UnsafeMutableRawPointer?) {
    guard let pointer else { return }
    onMain {
        let handle = Unmanaged<FluxaDesktopPlayerHandle>.fromOpaque(pointer).takeRetainedValue()
        handle.player.stop()
        handle.surface.removeFromSuperview()
    }
}

@_cdecl("fluxa_desktop_avplayer_load")
public func fluxaDesktopAvplayerLoad(
    _ pointer: UnsafeMutableRawPointer?,
    _ url: UnsafePointer<CChar>?,
    _ title: UnsafePointer<CChar>?,
    _ start: Double
) -> UnsafeMutablePointer<CChar>? {
    guard let handle = handle(pointer) else { return strdup("AVPlayer handle is unavailable") }
    do {
        try onMain {
            try handle.load(url: cString(url), title: cString(title), start: start)
        }
        return nil
    } catch {
        return strdup(error.localizedDescription)
    }
}

@_cdecl("fluxa_desktop_avplayer_command")
public func fluxaDesktopAvplayerCommand(
    _ pointer: UnsafeMutableRawPointer?,
    _ command: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>? {
    guard let handle = handle(pointer) else { return strdup("AVPlayer handle is unavailable") }
    let parts = splitCommand(cString(command))
    do {
        try onMain {
            guard let verb = parts.first else { return }
            switch verb {
            case "play": handle.player.play()
            case "pause": handle.player.pause()
            case "stop": handle.player.stop()
            case "cycle" where parts.dropFirst().first == "pause": handle.player.togglePlayback()
            case "seek":
                let amount = Double(parts.dropFirst().first ?? "0") ?? 0
                if parts.contains("relative") { handle.player.skip(by: amount) }
                else { handle.player.seek(to: amount) }
            case "set" where parts.dropFirst().first == "time-pos":
                handle.player.seek(to: Double(parts.dropFirst(2).first ?? "0") ?? 0)
            case "set" where parts.dropFirst().first == "speed":
                handle.player.setRate(Float(parts.dropFirst(2).first ?? "1") ?? 1)
            case "set" where parts.dropFirst().first == "volume":
                handle.player.setVolume((Float(parts.dropFirst(2).first ?? "100") ?? 100) / 100)
            case "set" where parts.dropFirst().first == "mute":
                handle.player.setVolume(parts.dropFirst(2).first == "yes" ? 0 : 1)
            case "set" where parts.dropFirst().first == "aid":
                let id = String(parts.dropFirst(2).first ?? "")
                handle.player.selectTrack(handle.player.tracks.first(where: { $0.id == "audio.\(id)" || $0.id == id }), kind: .audio)
            case "set" where parts.dropFirst().first == "sid":
                let value = String(parts.dropFirst(2).first ?? "")
                if value == "no" { handle.player.selectTrack(nil, kind: .subtitle) }
                else { handle.player.selectTrack(handle.player.tracks.first(where: { $0.id == "subtitle.\(value)" || $0.id == value }), kind: .subtitle) }
            default: break
            }
        }
        return nil
    } catch {
        return strdup(error.localizedDescription)
    }
}

@_cdecl("fluxa_desktop_avplayer_add_subtitle")
public func fluxaDesktopAvplayerAddSubtitle(
    _ pointer: UnsafeMutableRawPointer?,
    _ url: UnsafePointer<CChar>?
) -> UnsafeMutablePointer<CChar>? {
    guard let handle = handle(pointer), let subtitleURL = URL(string: cString(url)) else {
        return strdup("Invalid subtitle URL")
    }
    do {
        try onMain {
            handle.player.addExternalSubtitle(subtitleURL)
        }
        return nil
    } catch {
        return strdup(error.localizedDescription)
    }
}

@_cdecl("fluxa_desktop_avplayer_hide")
public func fluxaDesktopAvplayerHide(_ pointer: UnsafeMutableRawPointer?) {
    guard let handle = handle(pointer) else { return }
    onMain {
        handle.player.pause()
        handle.surface.isHidden = true
    }
}

@_cdecl("fluxa_desktop_avplayer_position")
public func fluxaDesktopAvplayerPosition(_ pointer: UnsafeMutableRawPointer?) -> Double {
    guard let handle = handle(pointer) else { return 0 }
    return (try? onMain { handle.player.state.position }) ?? 0
}

@_cdecl("fluxa_desktop_avplayer_duration")
public func fluxaDesktopAvplayerDuration(_ pointer: UnsafeMutableRawPointer?) -> Double {
    guard let handle = handle(pointer) else { return 0 }
    return (try? onMain { handle.player.state.duration }) ?? 0
}

@_cdecl("fluxa_desktop_avplayer_phase")
public func fluxaDesktopAvplayerPhase(_ pointer: UnsafeMutableRawPointer?) -> Int32 {
    guard let handle = handle(pointer) else { return 0 }
    return (try? onMain {
        switch handle.player.state.phase {
        case .idle: return 0
        case .loading: return 1
        case .playing: return 2
        case .paused: return 3
        case .ended: return 4
        case .failed: return 5
        }
    }) ?? 0
}

@_cdecl("fluxa_desktop_avplayer_tracks_json")
public func fluxaDesktopAvplayerTracksJSON(_ pointer: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>? {
    guard let handle = handle(pointer) else { return strdup("[]") }
    let json = (try? onMain {
        let values: [[String: Any]] = handle.player.tracks.map { track in
            [
                "id": track.id,
                "label": track.label,
                "selected": false,
                "lang": track.languageCode ?? "",
                "source": NSNull(),
                "external": track.kind == .subtitle,
                "format": NSNull(),
            ]
        }
        let data = try JSONSerialization.data(withJSONObject: values)
        return String(data: data, encoding: .utf8) ?? "[]"
    }) ?? "[]"
    return strdup(json)
}

@_cdecl("fluxa_desktop_avplayer_free_string")
public func fluxaDesktopAvplayerFreeString(_ pointer: UnsafeMutablePointer<CChar>?) {
    free(pointer)
}
#endif
