import Foundation

final class FluxaAppleHomeEffectHandler: FluxaApplePlatformEffectHandler {
    private let configurationStore: FluxaAppleAddonConfigurationStore
    private let catalogBootstrap: FluxaAppleCatalogBootstrap
    private let addonResourceLoader: FluxaAppleAddonResourceLoader

    init(
        configurationStore: FluxaAppleAddonConfigurationStore,
        catalogBootstrap: FluxaAppleCatalogBootstrap,
        addonResourceLoader: FluxaAppleAddonResourceLoader = FluxaAppleAddonResourceLoader()
    ) {
        self.configurationStore = configurationStore
        self.catalogBootstrap = catalogBootstrap
        self.addonResourceLoader = addonResourceLoader
    }

    func execute(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        switch effect.type {
        case "readHomeBootstrap":
            let rows = try await catalogBootstrap.loadRows(
                localAddonUrls: configurationStore.localAddonUrls()
            )
            return .object([
                "categories": .array(rows.map(homeCategory)),
                "continueWatching": .array([]),
                "watchlist": .array([]),
                "userAddons": .array(configurationStore.localAddonUrls().map { .string($0) }),
                "metadataFeeds": .array([]),
                "billboard": rows.first?.items.first.map(homeMeta) ?? .null
            ])
        case "refreshContinueWatching":
            return .object(["continueWatching": .array([])])
        case "fetchMetaDetail":
            return try await loadMeta(effect: effect)
        case "runSearch":
            return try await runSearch(effect: effect)
        case "readPlaybackProgress":
            return .null
        default:
            throw NSError(domain: "FluxaAppleUnsupportedEffect", code: 1)
        }
    }

    private func homeCategory(_ row: FluxaAppleCatalogRow) -> FluxaAppleJsonValue {
        .object([
            "id": .string(row.id),
            "name": .string(row.title),
            "semanticName": .string(row.title),
            "type": .string(row.items.first?.type ?? ""),
            "catalogId": .string(row.id),
            "items": .array(row.items.map(homeMeta))
        ])
    }

    private func homeMeta(_ item: FluxaAppleCatalogItem) -> FluxaAppleJsonValue {
        .object([
            "id": .string(item.id),
            "type": .string(item.type),
            "name": .string(item.title),
            "poster": optionalString(item.artworkUrl),
            "logo": optionalString(item.logoUrl),
            "releaseInfo": .string(item.subtitle),
            "addonTransportUrl": optionalString(item.addonTransportUrl),
            "catalogType": optionalString(item.catalogType)
        ])
    }

    private func optionalString(_ value: String?) -> FluxaAppleJsonValue {
        value.map(FluxaAppleJsonValue.string) ?? .null
    }

    private func string(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else {
            return nil
        }
        return text
    }

    private func loadMeta(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              let contentType = string(payload["contentType"]),
              let id = string(payload["id"]) else {
            throw URLError(.cannotParseResponse)
        }
        let transportUrl = string(payload["sourceAddonTransportUrl"])
            ?? configurationStore.localAddonUrls().first
        guard let transportUrl else {
            throw URLError(.fileDoesNotExist)
        }
        return try await addonResourceLoader.loadMeta(
            transportUrl: transportUrl,
            contentType: contentType,
            id: id
        )
    }

    private func runSearch(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              let query = string(payload["query"]) else {
            throw URLError(.cannotParseResponse)
        }
        let items = try await catalogBootstrap.loadSearchItems(
            localAddonUrls: configurationStore.localAddonUrls(),
            query: query
        )
        return .object(["results": .array(items.map(homeMeta))])
    }
}
