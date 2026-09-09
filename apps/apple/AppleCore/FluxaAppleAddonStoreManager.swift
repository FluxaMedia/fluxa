import Foundation

struct FluxaAppleInstalledAddon {
    let name: String
    let description: String
    let url: String
    let logoUrl: String?
    let version: String?
    let configurable: Bool
    let isEnabled: Bool
    let canMoveUp: Bool
    let canMoveDown: Bool
}

final class FluxaAppleAddonStoreManager {
    private let configurationStore: FluxaAppleAddonConfigurationStore
    private let session: URLSession

    init(
        configurationStore: FluxaAppleAddonConfigurationStore = FluxaAppleAddonConfigurationStore(),
        session: URLSession = .shared
    ) {
        self.configurationStore = configurationStore
        self.session = session
    }

    func currentAddons() async -> [FluxaAppleInstalledAddon] {
        let urls = configurationStore.localAddonUrls()
        let disabled = configurationStore.disabledAddonUrls()
        var repositoryAddons = [[String: Any]]()
        for url in urls {
            let manifest = await fetchManifest(rawUrl: url)
            let resolved = manifest ?? FluxaCoreAddonManifest(
                id: "",
                name: fallbackName(for: url),
                description: nil,
                logo: nil,
                version: nil,
                configurable: false,
                supportsCatalog: false,
                catalogs: []
            )
            var manifestFields: [String: Any] = ["name": resolved.name]
            if let description = resolved.description { manifestFields["description"] = description }
            if let logo = resolved.logo { manifestFields["logo"] = logo }
            if let version = resolved.version { manifestFields["version"] = version }
            if let configurable = resolved.configurable { manifestFields["configurable"] = configurable }
            repositoryAddons.append([
                "transportUrl": url,
                "manifest": manifestFields,
                "isManaged": false,
                "isEnabled": true
            ])
        }
        return FluxaCoreStremio.addonStoreEntriesPlan(
            repositoryAddons: repositoryAddons,
            localUrls: urls,
            disabledKeys: disabled,
            localLoaded: true
        ).map {
            FluxaAppleInstalledAddon(
                name: $0.name,
                description: $0.description,
                url: $0.url,
                logoUrl: $0.logoUrl,
                version: $0.version,
                configurable: $0.configurable,
                isEnabled: $0.isEnabled,
                canMoveUp: $0.canMoveUp,
                canMoveDown: $0.canMoveDown
            )
        }
    }

    func submitManifestUrl(_ raw: String) async -> (addons: [FluxaAppleInstalledAddon], addedName: String?, failed: Bool) {
        let normalized = FluxaCoreStremio.normalizeManifestUrl(raw)
        guard let manifest = await fetchManifest(rawUrl: normalized) else {
            return (await currentAddons(), nil, true)
        }
        let profile = mutationProfile()
        let updated = FluxaCoreStremio.addonProfileMutationPlan(
            profile: profile,
            command: "install",
            addonKey: normalized
        ) ?? profile
        saveMutationProfile(updated)
        return (await currentAddons(), manifest.name, false)
    }

    func toggleAddon(url: String, enabled: Bool) async -> [FluxaAppleInstalledAddon] {
        let updated = FluxaCoreStremio.addonProfileMutationPlan(
            profile: mutationProfile(),
            command: enabled ? "enable" : "disable",
            addonKey: url
        ) ?? mutationProfile()
        saveMutationProfile(updated)
        return await currentAddons()
    }

    func removeAddon(url: String) async -> [FluxaAppleInstalledAddon] {
        let updated = FluxaCoreStremio.addonProfileMutationPlan(
            profile: mutationProfile(),
            command: "remove",
            addonKey: url
        ) ?? mutationProfile()
        saveMutationProfile(updated)
        return await currentAddons()
    }

    func moveAddon(url: String, direction: Int) async -> [FluxaAppleInstalledAddon] {
        let updated = FluxaCoreStremio.addonProfileMutationPlan(
            profile: mutationProfile(),
            command: "move",
            addonKey: url,
            direction: direction
        ) ?? mutationProfile()
        saveMutationProfile(updated)
        return await currentAddons()
    }

    func refreshAddon(url: String) async -> [FluxaAppleInstalledAddon] {
        await currentAddons()
    }

    private func fetchManifest(rawUrl: String) async -> FluxaCoreAddonManifest? {
        let transportUrl = FluxaCoreStremio.normalizeManifestUrl(rawUrl)
        guard let manifestUrl = URL(string: transportUrl) else {
            return nil
        }
        do {
            let (data, response) = try await session.data(from: manifestUrl)
            guard let httpResponse = response as? HTTPURLResponse,
                  (200..<300).contains(httpResponse.statusCode) else {
                return nil
            }
            return FluxaCoreStremio.parseManifest(
                body: String(decoding: data, as: UTF8.self),
                transportUrl: transportUrl
            )
        } catch {
            return nil
        }
    }

    private func fallbackName(for url: String) -> String {
        URL(string: url)?.host ?? url
    }

    private func mutationProfile() -> [String: Any] {
        [
            "localAddons": configurationStore.localAddonUrls(),
            "disabledLocalAddons": Array(configurationStore.disabledAddonUrls())
        ]
    }

    private func saveMutationProfile(_ profile: [String: Any]) {
        if let urls = profile["localAddons"] as? [String] {
            configurationStore.save(localAddonUrls: urls)
        }
        if let disabled = profile["disabledLocalAddons"] as? [String] {
            configurationStore.save(disabledAddonUrls: Set(disabled))
        }
    }
}
