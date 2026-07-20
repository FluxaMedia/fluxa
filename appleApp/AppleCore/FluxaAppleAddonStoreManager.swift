import FluxaShared
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
        var results = [FluxaAppleInstalledAddon]()
        for (index, url) in urls.enumerated() {
            let manifest = await fetchManifest(rawUrl: url)
            results.append(
                FluxaAppleInstalledAddon(
                    name: manifest?.name ?? fallbackName(for: url),
                    description: manifest?.description ?? "",
                    url: url,
                    logoUrl: manifest?.logo,
                    version: manifest?.version,
                    configurable: manifest?.configurable ?? false,
                    isEnabled: !disabled.contains(url),
                    canMoveUp: index > 0,
                    canMoveDown: index < urls.count - 1
                )
            )
        }
        return results
    }

    func submitManifestUrl(_ raw: String) async -> (addons: [FluxaAppleInstalledAddon], addedName: String?, failed: Bool) {
        let normalized = normalizeManifestUrl(raw)
        guard let manifest = await fetchManifest(rawUrl: normalized) else {
            return (await currentAddons(), nil, true)
        }
        var urls = configurationStore.localAddonUrls()
        if !urls.contains(where: { normalizeManifestUrl($0) == normalized }) {
            urls.append(normalized)
            configurationStore.save(localAddonUrls: urls)
        }
        return (await currentAddons(), manifest.name, false)
    }

    func toggleAddon(url: String, enabled: Bool) async -> [FluxaAppleInstalledAddon] {
        var disabled = configurationStore.disabledAddonUrls()
        if enabled {
            disabled.remove(url)
        } else {
            disabled.insert(url)
        }
        configurationStore.save(disabledAddonUrls: disabled)
        return await currentAddons()
    }

    func removeAddon(url: String) async -> [FluxaAppleInstalledAddon] {
        let normalized = normalizeManifestUrl(url)
        var urls = configurationStore.localAddonUrls()
        urls.removeAll { normalizeManifestUrl($0) == normalized }
        configurationStore.save(localAddonUrls: urls)
        var disabled = configurationStore.disabledAddonUrls()
        disabled.remove(url)
        configurationStore.save(disabledAddonUrls: disabled)
        return await currentAddons()
    }

    func moveAddon(url: String, direction: Int) async -> [FluxaAppleInstalledAddon] {
        let normalized = normalizeManifestUrl(url)
        var urls = configurationStore.localAddonUrls()
        guard let index = urls.firstIndex(where: { normalizeManifestUrl($0) == normalized }) else {
            return await currentAddons()
        }
        let newIndex = index + direction
        guard urls.indices.contains(newIndex) else {
            return await currentAddons()
        }
        urls.swapAt(index, newIndex)
        configurationStore.save(localAddonUrls: urls)
        return await currentAddons()
    }

    func refreshAddon(url: String) async -> [FluxaAppleInstalledAddon] {
        await currentAddons()
    }

    private func fetchManifest(rawUrl: String) async -> AppleAddonManifestSnapshot? {
        guard let manifestUrl = URL(string: normalizeManifestUrl(rawUrl)) else {
            return nil
        }
        do {
            let (data, response) = try await session.data(from: manifestUrl)
            guard let httpResponse = response as? HTTPURLResponse,
                  (200..<300).contains(httpResponse.statusCode) else {
                return nil
            }
            return FluxaApple.shared.parseAddonManifest(body: String(decoding: data, as: UTF8.self))
        } catch {
            return nil
        }
    }

    private func normalizeManifestUrl(_ raw: String) -> String {
        FluxaApple.shared.normalizeAddonManifestUrl(rawUrl: raw)
    }

    private func fallbackName(for url: String) -> String {
        URL(string: url)?.host ?? url
    }
}
