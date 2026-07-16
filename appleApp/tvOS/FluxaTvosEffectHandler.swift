import FluxaData
import Foundation

final class FluxaTvosEffectHandler: FluxaApplePlatformEffectHandler {
    private let configurationStore: FluxaAppleAddonConfigurationStore
    private let session: URLSession

    init(
        configurationStore: FluxaAppleAddonConfigurationStore,
        session: URLSession = .shared
    ) {
        self.configurationStore = configurationStore
        self.session = session
    }

    func execute(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        switch effect.type {
        case "readHomeBootstrap":
            let categories = try await loadCategories()
            return .object([
                "categories": .array(categories),
                "continueWatching": .array([]),
                "watchlist": .array([]),
                "userAddons": .array(configurationStore.localAddonUrls().map(FluxaAppleJsonValue.string)),
                "metadataFeeds": .array([]),
                "billboard": firstItem(in: categories) ?? .null
            ])
        case "refreshContinueWatching":
            return .object(["continueWatching": .array([])])
        case "readLibraryState":
            return .object([
                "watchlist": .array([]),
                "continueWatching": .array([]),
                "liked": .array([]),
                "watched": .object([:])
            ])
        case "readCalendarMonth":
            return .object(["items": .array([])])
        case "readPlaybackProgress":
            return .null
        default:
            throw NSError(domain: "FluxaTvosUnsupportedEffect", code: 1)
        }
    }

    private func loadCategories() async throws -> [FluxaAppleJsonValue] {
        var categories = [FluxaAppleJsonValue]()
        var lastError: Error?
        for rawUrl in configurationStore.localAddonUrls() {
            do {
                categories.append(contentsOf: try await loadCategories(addonUrl: rawUrl))
            } catch {
                lastError = error
            }
        }
        if categories.isEmpty, let lastError {
            throw lastError
        }
        return categories
    }

    private func loadCategories(addonUrl: String) async throws -> [FluxaAppleJsonValue] {
        let transportUrl = AppleStremioBridge.shared.normalizeManifestUrl(rawUrl: addonUrl)
        let manifestBody = try await body(urlString: transportUrl)
        guard let manifest = AppleStremioBridge.shared.parseManifest(body: manifestBody),
              manifest.supportsCatalog else {
            return []
        }
        var categories = [FluxaAppleJsonValue]()
        for catalog in manifest.catalogs ?? [] where catalog.supportsInitialLoad {
            guard let type = catalog.type, !type.isEmpty,
                  let id = catalog.id, !id.isEmpty else {
                continue
            }
            let catalogUrl = AppleStremioBridge.shared.catalogUrl(
                transportUrl: transportUrl,
                contentType: type,
                catalogId: id,
                extraName: nil,
                extraValue: nil
            )
            let catalogBody = try await body(urlString: catalogUrl)
            let items = AppleStremioBridge.shared.parseCatalogItems(body: catalogBody, fallbackType: type) ?? []
            let title = catalog.name.flatMap { $0.isEmpty ? nil : $0 } ?? id
            categories.append(.object([
                "id": .string("\(manifest.id):\(type):\(id)"),
                "name": .string(title),
                "semanticName": .string(title),
                "type": .string(type),
                "catalogId": .string(id),
                "items": .array(items.map { item in
                    .object([
                        "id": .string(item.id),
                        "type": .string(item.type),
                        "name": .string(item.title),
                        "poster": optionalString(item.artworkUrl),
                        "logo": optionalString(item.logoUrl),
                        "background": optionalString(item.backgroundUrl),
                        "releaseInfo": .string(item.subtitle),
                        "addonTransportUrl": .string(transportUrl),
                        "catalogType": .string(type)
                    ])
                })
            ]))
        }
        return categories
    }

    private func body(urlString: String) async throws -> String {
        guard let url = URL(string: urlString) else {
            throw URLError(.badURL)
        }
        let (data, response) = try await session.data(from: url)
        guard let httpResponse = response as? HTTPURLResponse,
              (200..<300).contains(httpResponse.statusCode) else {
            throw URLError(.badServerResponse)
        }
        return String(decoding: data, as: UTF8.self)
    }

    private func optionalString(_ value: String?) -> FluxaAppleJsonValue {
        value.map(FluxaAppleJsonValue.string) ?? .null
    }

    private func firstItem(in categories: [FluxaAppleJsonValue]) -> FluxaAppleJsonValue? {
        guard case .object(let category)? = categories.first,
              case .array(let items)? = category["items"] else {
            return nil
        }
        return items.first
    }
}
