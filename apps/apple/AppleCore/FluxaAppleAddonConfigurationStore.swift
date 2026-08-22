import Foundation

final class FluxaAppleAddonConfigurationStore {
    static let localAddonUrlsKey = "fluxa.apple.localAddonUrls"
    static let disabledAddonUrlsKey = "fluxa.apple.disabledAddonUrls"

    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    func localAddonUrls() -> [String] {
        let stored = defaults.stringArray(forKey: Self.localAddonUrlsKey)
        let values = stored ?? ["https://v3-cinemeta.strem.io/manifest.json"]
        return values.filter { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
    }

    func enabledAddonUrls() -> [String] {
        let disabled = disabledAddonUrls()
        return localAddonUrls().filter { !disabled.contains($0) }
    }

    func save(localAddonUrls: [String]) {
        let normalized = localAddonUrls.map {
            $0.trimmingCharacters(in: .whitespacesAndNewlines)
        }.filter { !$0.isEmpty }
        defaults.set(normalized, forKey: Self.localAddonUrlsKey)
    }

    func disabledAddonUrls() -> Set<String> {
        Set(defaults.stringArray(forKey: Self.disabledAddonUrlsKey) ?? [])
    }

    func save(disabledAddonUrls: Set<String>) {
        defaults.set(Array(disabledAddonUrls), forKey: Self.disabledAddonUrlsKey)
    }
}
