import Foundation

final class FluxaAppleAddonConfigurationStore {
    static let localAddonUrlsKey = "fluxa.apple.localAddonUrls"

    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    func localAddonUrls() -> [String] {
        let stored = defaults.stringArray(forKey: Self.localAddonUrlsKey)
        let values = stored ?? ["https://v3-cinemeta.strem.io/manifest.json"]
        return values.filter { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
    }

    func save(localAddonUrls: [String]) {
        let normalized = localAddonUrls.map {
            $0.trimmingCharacters(in: .whitespacesAndNewlines)
        }.filter { !$0.isEmpty }
        defaults.set(normalized, forKey: Self.localAddonUrlsKey)
    }
}
