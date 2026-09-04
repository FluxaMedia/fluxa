import Foundation
import FluxaCore
import FluxaShared

final class FluxaAppleHomeEffectHandler: FluxaApplePlatformEffectHandler {
    private let configurationStore: FluxaAppleAddonConfigurationStore
    private let catalogService: FluxaAppleCatalogService
    private let addonResourceLoader: FluxaAppleAddonResourceLoader
    private let libraryStore: FluxaAppleLibraryStore
    private let tmdbService: FluxaAppleTmdbService

    init(
        configurationStore: FluxaAppleAddonConfigurationStore,
        catalogService: FluxaAppleCatalogService = FluxaAppleCatalogService(),
        addonResourceLoader: FluxaAppleAddonResourceLoader = FluxaAppleAddonResourceLoader(),
        libraryStore: FluxaAppleLibraryStore = FluxaAppleLibraryStore(),
        tmdbService: FluxaAppleTmdbService = FluxaAppleTmdbService()
    ) {
        self.configurationStore = configurationStore
        self.catalogService = catalogService
        self.addonResourceLoader = addonResourceLoader
        self.libraryStore = libraryStore
        self.tmdbService = tmdbService
    }

    func execute(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        switch effect.type {
        case FluxaHeadlessEffectType.readHomeBootstrap:
            let rows = try await catalogService.loadHomeRows(
                localAddonUrls: configurationStore.enabledAddonUrls()
            )
            return .object([
                "categories": .array(rows.map(homeCategory)),
                "continueWatching": .array([]),
                "watchlist": .array([]),
                "userAddons": .array(configurationStore.localAddonUrls().map { .string($0) }),
                "metadataFeeds": .array([]),
                "billboard": rows.first?.items.first.map(homeMeta) ?? .null
            ])
        case FluxaHeadlessEffectType.refreshContinueWatching:
            return .object(["continueWatching": .array([])])
        case FluxaHeadlessEffectType.fetchMetaDetail:
            return try await loadMeta(effect: effect)
        case FluxaHeadlessEffectType.fetchDetailSecondary:
            return try await loadDetailSecondary(effect: effect)
        case FluxaHeadlessEffectType.runSearch:
            return try await runSearch(effect: effect)
        case FluxaHeadlessEffectType.runDiscover:
            return try await runDiscover(effect: effect)
        case FluxaHeadlessEffectType.readDiscoverCatalogFilters:
            return try await readDiscoverCatalogFilters(effect: effect)
        case FluxaHeadlessEffectType.readLibraryState:
            return .object([
                "watchlist": .array(libraryStore.watchlist()),
                "continueWatching": .array([]),
                "liked": .array([]),
                "watched": .object([:])
            ])
        case FluxaHeadlessEffectType.readCalendarMonth:
            return .object(["items": .array([])])
        case FluxaHeadlessEffectType.writeLibraryCommand:
            return try writeLibraryCommand(effect: effect)
        case FluxaHeadlessEffectType.readPlaybackProgress:
            return .null
        default:
            throw NSError(domain: "FluxaAppleUnsupportedEffect", code: 1)
        }
    }

    private func homeCategory(_ row: FluxaCore.AppleCatalogRowSnapshot) -> FluxaAppleJsonValue {
        .object([
            "id": .string(row.id),
            "name": .string(row.title),
            "semanticName": .string(row.title),
            "type": .string(row.items.first?.type ?? ""),
            "catalogId": .string(row.id),
            "items": .array(row.items.map(homeMeta))
        ])
    }

    private func homeMeta(_ item: FluxaCore.AppleCatalogItemSnapshot) -> FluxaAppleJsonValue {
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
            ?? configurationStore.enabledAddonUrls().first
        guard let transportUrl else {
            throw URLError(.fileDoesNotExist)
        }
        return try await addonResourceLoader.loadMeta(
            transportUrl: transportUrl,
            contentType: contentType,
            id: id
        )
    }

    private func loadDetailSecondary(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              let contentType = string(payload["contentType"]),
              let rawId = string(payload["id"]),
              let apiKey = UserDefaults.standard.string(forKey: "fluxa.apple.settings.tmdb_api_key"),
              !apiKey.isEmpty else {
            return .object(["similarItems": .array([]), "watchedVideoIds": .array([])])
        }
        let items = try await tmdbService.loadRecommendations(
            contentType: contentType,
            id: rawId,
            language: string(payload["language"]) ?? "en-US",
            apiKey: apiKey
        )
        return .object(["similarItems": .array(items), "watchedVideoIds": .array([])])
    }

    private func runSearch(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              let query = string(payload["query"]) else {
            throw URLError(.cannotParseResponse)
        }
        let items = try await catalogService.loadSearchItems(
            addonUrls: configurationStore.enabledAddonUrls(),
            query: query
        )
        return .object(["results": .array(items.map(homeMeta))])
    }

    private func runDiscover(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              let contentType = string(payload["contentType"]) else {
            throw URLError(.cannotParseResponse)
        }
        let filters: [String: FluxaAppleJsonValue] = {
            guard case .object(let value)? = payload["filters"] else {
                return [:]
            }
            return value
        }()
        let selectedCatalogKey = string(filters["catalogKey"])
        let genre = string(filters["genre"])
        let catalogs = try await catalogService.resolveDiscoverCatalogs(
            addonUrls: configurationStore.enabledAddonUrls(),
            contentType: contentType
        )
        let selectedCatalogs = selectedCatalogKey.map { key in catalogs.filter { $0.key == key } } ?? []
        let catalogsToLoad = selectedCatalogs.isEmpty ? catalogs : selectedCatalogs
        let requests = catalogsToLoad.compactMap { catalog -> FluxaAppleCatalogRequest? in
            guard let url = catalogService.discoverUrl(
                transportUrl: catalog.transportUrl,
                contentType: catalog.contentType,
                catalogId: catalog.catalogId,
                genre: genre
            ) else {
                return nil
            }
            return FluxaAppleCatalogRequest(
                id: catalog.key,
                title: catalog.label,
                url: url,
                contentType: catalog.contentType,
                addonTransportUrl: catalog.transportUrl,
                catalogType: catalog.contentType
            )
        }
        let rows = try await catalogService.loadRows(requests: requests)
        let items = rows.flatMap(\.items)
        return .object([
            "results": .array(items.map(homeMeta)),
            "resultSources": .object([:])
        ])
    }

    private func readDiscoverCatalogFilters(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              let contentType = string(payload["contentType"]) else {
            throw URLError(.cannotParseResponse)
        }
        let selectedCatalogKey = string(payload["selectedCatalogKey"])
        let catalogs = try await catalogService.resolveDiscoverCatalogs(
            localAddonUrls: configurationStore.enabledAddonUrls(),
            contentType: contentType
        )
        let selectedCatalog = catalogs.first { $0.key == selectedCatalogKey }
        let genres = selectedCatalog.map { catalog in
            catalog.requiresGenre ? catalog.genres : [""] + catalog.genres
        } ?? []
        return .object([
            "catalogs": .array(catalogs.map(discoverCatalog)),
            "genres": .array(genres.map { genre in
                .object([
                    "id": genre.isEmpty ? .null : .string(genre),
                    "label": .string(genre)
                ])
            })
        ])
    }

    private func discoverCatalog(_ catalog: FluxaAppleDiscoverCatalog) -> FluxaAppleJsonValue {
        .object([
            "key": .string(catalog.key),
            "label": .string(catalog.label),
            "transportUrl": .string(catalog.transportUrl),
            "type": .string(catalog.contentType),
            "id": .string(catalog.catalogId),
            "genres": .array(catalog.genres.map(FluxaAppleJsonValue.string)),
            "requiresGenre": .boolean(catalog.requiresGenre)
        ])
    }

    private func writeLibraryCommand(effect: FluxaAppleHeadlessEffect) throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              case .object(let command)? = payload["command"],
              case .string(let type)? = command["type"] else {
            throw URLError(.cannotParseResponse)
        }
        switch type {
        case "toggleWatchlist":
            guard let item = command["item"] else {
                throw URLError(.cannotParseResponse)
            }
            let result = libraryStore.watchlistState(item: item)
            return .object([
                "watchlist": .array(result.watchlist),
                "isInWatchlist": .boolean(result.isInWatchlist)
            ])
        default:
            throw NSError(domain: "FluxaAppleUnsupportedLibraryCommand", code: 1)
        }
    }
}
