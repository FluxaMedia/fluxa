import SwiftUI

struct FluxaThemeColors: Codable {
    let background: String
    let backgroundElevated: String
    let surface: String
    let surfaceRaised: String
    let navigation: String
    let textPrimary: String
    let textSecondary: String
    let textMuted: String
    let border: String
    let borderStrong: String
    let accent: String
    let accentForeground: String
    let success: String
    let warning: String
    let error: String
    let info: String
    let focus: String
    let scrim: String
}

struct FluxaThemeTypography: Codable {
    let displayFont: String
    let bodyFont: String
    let titleWeight: Int
    let bodyWeight: Int
}

struct FluxaThemeShape: Codable {
    let cardRadius: Double
    let controlRadius: Double
    let dialogRadius: Double
}

struct FluxaThemeSpacing: Codable {
    let screenPadding: Double
    let sectionGap: Double
    let controlGap: Double
}

struct FluxaThemeMotion: Codable {
    let enabled: Bool
    let fastMs: Double
    let normalMs: Double
    let slowMs: Double
}

struct FluxaThemeLayouts: Codable {
    let home: String
    let detail: String
    let library: String
    let navigation: String
}

struct FluxaThemePack: Codable {
    let schemaVersion: Int
    let id: String
    let nameKey: String
    let colors: FluxaThemeColors
    let typography: FluxaThemeTypography
    let shape: FluxaThemeShape
    let spacing: FluxaThemeSpacing
    let motion: FluxaThemeMotion
    let layouts: FluxaThemeLayouts
}

enum FluxaThemePacks {
    static let fluxaDark = FluxaThemePack(
        schemaVersion: 1,
        id: "fluxa-dark",
        nameKey: "theme.fluxa_dark",
        colors: FluxaThemeColors(
            background: "#060606",
            backgroundElevated: "#0A0A0B",
            surface: "#141416",
            surfaceRaised: "#1C1C1F",
            navigation: "#0D0D0D",
            textPrimary: "#FFFFFF",
            textSecondary: "#B8B8BE",
            textMuted: "#8A8A91",
            border: "#FFFFFF14",
            borderStrong: "#FFFFFF29",
            accent: "#E85D3F",
            accentForeground: "#FFFFFF",
            success: "#45D483",
            warning: "#F0C674",
            error: "#FF6B6B",
            info: "#2196F3",
            focus: "#FFFFFF",
            scrim: "#000000C7"
        ),
        typography: FluxaThemeTypography(displayFont: "Archivo", bodyFont: "Montserrat", titleWeight: 700, bodyWeight: 400),
        shape: FluxaThemeShape(cardRadius: 12, controlRadius: 8, dialogRadius: 16),
        spacing: FluxaThemeSpacing(screenPadding: 24, sectionGap: 20, controlGap: 12),
        motion: FluxaThemeMotion(enabled: true, fastMs: 120, normalMs: 180, slowMs: 300),
        layouts: FluxaThemeLayouts(home: "shelves", detail: "hero-with-rail", library: "poster-grid", navigation: "sidebar")
    )

    static func decode(_ data: Data) -> FluxaThemePack {
        (try? JSONDecoder().decode(FluxaThemePack.self, from: data)) ?? fluxaDark
    }
}

extension Color {
    init(themeHex: String, fallback: Color) {
        let value = themeHex.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        guard value.count == 6 || value.count == 8 else {
            self = fallback
            return
        }
        let red = Double(Int(value.prefix(2), radix: 16) ?? 0) / 255
        let green = Double(Int(value.dropFirst(2).prefix(2), radix: 16) ?? 0) / 255
        let blue = Double(Int(value.dropFirst(4).prefix(2), radix: 16) ?? 0) / 255
        let opacity = value.count == 8 ? Double(Int(value.suffix(2), radix: 16) ?? 255) / 255 : 1
        self = Color(red: red, green: green, blue: blue, opacity: opacity)
    }
}

private struct FluxaThemeEnvironmentKey: EnvironmentKey {
    static let defaultValue = FluxaThemePacks.fluxaDark
}

extension EnvironmentValues {
    var fluxaTheme: FluxaThemePack {
        get { self[FluxaThemeEnvironmentKey.self] }
        set { self[FluxaThemeEnvironmentKey.self] = newValue }
    }
}
