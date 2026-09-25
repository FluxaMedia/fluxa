import Foundation
import FluxaCore
import FluxaShared

final class FluxaAppleHomeEffectHandler: FluxaApplePlatformEffectHandler {
    private let configurationStore: FluxaAppleAddonConfigurationStore
    private let catalogService: FluxaAppleCatalogService
    private let streamEffectService: FluxaAppleAddonStreamEffectService
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
        self.streamEffectService = FluxaAppleAddonStreamEffectService(
            configurationStore: configurationStore,
            addonResourceLoader: addonResourceLoader
        )
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
            return try await streamEffectService.execute(effect: effect)
        case FluxaHeadlessEffectType.fetchMetaDetailLookup:
            return try await streamEffectService.execute(effect: effect)
        case FluxaHeadlessEffectType.fetchDetailSecondary:
            return try await loadDetailSecondary(effect: effect)
        case FluxaHeadlessEffectType.fetchDetailStreams:
            return try await streamEffectService.execute(effect: effect)
        case FluxaHeadlessEffectType.prefetchNextEpisodeStreams:
            return try await streamEffectService.execute(effect: effect)
        case FluxaHeadlessEffectType.runSearch:
            return try await runSearch(effect: effect)
        case FluxaHeadlessEffectType.runDiscover:
            return try await runDiscover(effect: effect)
        case FluxaHeadlessEffectType.readDiscoverCatalogFilters:
            return try await readDiscoverCatalogFilters(effect: effect)
        case FluxaHeadlessEffectType.readLibraryState:
            return libraryStore.snapshot()
        case FluxaHeadlessEffectType.readCalendarMonth:
            return .object(["items": .array([])])
        case FluxaHeadlessEffectType.writeLibraryCommand:
            return try writeLibraryCommand(effect: effect)
        case FluxaHeadlessEffectType.readPlaybackProgress:
            return .null
        case FluxaHeadlessEffectType.fetchSubtitles:
            guard case .object(let payload) = effect.payload,
                  let stream = payload["stream"] else {
                return .object(["subtitles": .array([])])
            }
            return FluxaCoreStremio.streamSubtitlesResult(stream: stream)
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
        guard let filterData = try? JSONEncoder().encode(FluxaAppleJsonValue.object(filters)),
              let coreFilters = try? JSONSerialization.jsonObject(with: filterData) as? [String: Any] else {
            throw URLError(.cannotParseResponse)
        }
        guard let sourceRequests = FluxaCoreStremio.discoverSourceRequests(
            contentType: contentType,
            filters: coreFilters
        ) else { throw URLError(.cannotParseResponse) }
        let requestsWithSources = sourceRequests.compactMap { source -> (FluxaAppleCatalogRequest, [String: Any])? in
            guard let transportUrl = source["transportUrl"] as? String,
                  let requestType = source["type"] as? String,
                  let catalogId = source["catalogId"] as? String,
                  let rawUrl = FluxaCoreStremio.resourceUrl(
                    transportUrl: transportUrl,
                    resource: "catalog",
                    contentType: requestType,
                    id: catalogId,
                    extra: (source["extra"] as? [String: Any] ?? [:]).compactMapValues { value in
                        if let value = value as? String { return value }
                        if let value = value as? NSNumber { return value.stringValue }
                        return nil
                    }
                  ),
                  let url = URL(string: rawUrl) else { return nil }
            let request = FluxaAppleCatalogRequest(
                id: "\(transportUrl)|\(catalogId)|\(requestType)",
                title: catalogId,
                url: url,
                contentType: requestType,
                addonTransportUrl: transportUrl,
                catalogType: requestType
            )
            return (request, source)
        }
        let requests = requestsWithSources.map(\.0)
        let rows = (try? await catalogService.loadRows(requests: requests)) ?? []
        let sourcesByRequestId = Dictionary(uniqueKeysWithValues: requestsWithSources.map { ($0.0.id, $0.1) })
        let sources = rows.map { row -> [String: Any] in
            let request = requestsWithSources.first { $0.0.id == row.id }?.0
            let sourceRequest = sourcesByRequestId[row.id] ?? [:]
            let sourceExtra = sourceRequest["extra"] as? [String: Any] ?? [:]
            return [
                "transportUrl": sourceRequest["transportUrl"] as? String ?? request?.addonTransportUrl ?? "",
                "catalogId": sourceRequest["catalogId"] as? String ?? "",
                "type": sourceRequest["type"] as? String ?? request?.catalogType ?? row.items.first?.type ?? contentType,
                "genre": sourceExtra["genre"] as Any? ?? NSNull(),
                "items": row.items.map { item in
                    [
                        "id": item.id,
                        "type": item.type,
                        "name": item.title,
                        "releaseInfo": item.subtitle,
                        "poster": item.artworkUrl as Any? ?? NSNull(),
                        "logo": item.logoUrl as Any? ?? NSNull(),
                        "addonTransportUrl": item.addonTransportUrl as Any? ?? NSNull(),
                        "catalogType": item.catalogType as Any? ?? NSNull()
                    ]
                }
            ]
        }
        guard let merged = FluxaCoreStremio.mergeDiscoverSources(sources),
              JSONSerialization.isValidJSONObject(merged),
              let data = try? JSONSerialization.data(withJSONObject: merged),
              let result = try? JSONDecoder().decode(FluxaAppleJsonValue.self, from: data) else {
            throw URLError(.cannotParseResponse)
        }
        return result
    }

    private func readDiscoverCatalogFilters(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload else {
            throw URLError(.cannotParseResponse)
        }
        let profileAddonUrls: [String] = {
            guard case .object(let profile)? = payload["profile"],
                  case .array(let values)? = profile["localAddons"] else {
                return []
            }
            return values.compactMap { string($0) }.filter { !$0.isEmpty }
        }()
        let addonUrls = (profileAddonUrls + configurationStore.enabledAddonUrls()).reduce(into: [String]()) {
            if !$0.contains($1) { $0.append($1) }
        }
        let descriptors = try await catalogService.loadAddonDescriptors(
            addonUrls: addonUrls
        )
        guard JSONSerialization.isValidJSONObject(descriptors),
              let data = try? JSONSerialization.data(withJSONObject: descriptors),
              let value = try? JSONDecoder().decode(FluxaAppleJsonValue.self, from: data) else {
            throw URLError(.cannotParseResponse)
        }
        return .object(["addons": value])
    }

    private func writeLibraryCommand(effect: FluxaAppleHeadlessEffect) throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              let command = payload["command"] else {
            throw URLError(.cannotParseResponse)
        }
        return try libraryStore.applyCommand(command, source: string(payload["source"]))
    }
}
