import Foundation

struct FluxaCoreAddonCatalogExtra: Decodable {
    let name: String?
    let isRequired: Bool?
    let options: [String]?
}

struct FluxaCoreResourceRequest: Decodable {
    let url: String
    let kind: String
    let addonName: String?
    let stopOnFirstResult: Bool?
    let transportUrl: String?
    let contentType: String?
    let id: String?
    let catalogId: String?
    let catalogType: String?
    let categoryId: String?
    let categoryName: String?
}

struct FluxaCoreResourceFetchPlan: Decodable {
    let requests: [FluxaCoreResourceRequest]
}

struct FluxaCoreResourceExecutionPolicy: Decodable {
    let requests: [FluxaCoreResourceRequest]
    let mode: String
    let concurrency: Int
}

struct FluxaCoreAddonCatalog: Decodable {
    let type: String?
    let id: String?
    let name: String?
    let genres: [String]?
    let extra: [FluxaCoreAddonCatalogExtra]?
    let supportsInitialLoad: Bool
    let supportsSearch: Bool
    let hasRequiredExtraExceptGenre: Bool
}

struct FluxaCoreAddonManifest: Decodable {
    let id: String
    let name: String
    let description: String?
    let logo: String?
    let version: String?
    let configurable: Bool?
    let supportsCatalog: Bool
    let catalogs: [FluxaCoreAddonCatalog]
}

struct FluxaCoreAddonStoreEntry: Decodable {
    let name: String
    let description: String
    let url: String
    let logoUrl: String?
    let version: String?
    let configurable: Bool
    let isEnabled: Bool
    let canRemove: Bool
    let canMoveUp: Bool
    let canMoveDown: Bool
    let isRefreshing: Bool
}

struct FluxaCoreCatalogItem: Decodable {
    let id: String
    let type: String
    let title: String
    let subtitle: String
    let artworkUrl: String?
    let logoUrl: String?
    let backgroundUrl: String?
    let description: String?
}

struct FluxaCoreDirectStream: Decodable {
    let title: String?
    let playableUrl: String
    let requestHeaders: [String: String]
}

private struct FluxaCoreAddonManifestDescriptor: Decodable {
    let manifest: FluxaCoreAddonManifest
}

enum FluxaCoreStremio {
    static func libraryCommandPlan(
        library: FluxaAppleJsonValue,
        command: FluxaAppleJsonValue,
        nowIso: String
    ) -> FluxaAppleJsonValue? {
        let request = FluxaAppleJsonValue.object([
            "library": library,
            "command": command,
            "nowIso": .string(nowIso)
        ])
        guard let requestData = try? JSONEncoder().encode(request),
              let requestJson = String(data: requestData, encoding: .utf8),
              let result = value(method: "libraryCommandPlan", argsJson: requestJson),
              JSONSerialization.isValidJSONObject(result),
              let resultData = try? JSONSerialization.data(withJSONObject: result) else {
            return nil
        }
        return try? JSONDecoder().decode(FluxaAppleJsonValue.self, from: resultData)
    }

    static func formatRuntimeLabel(_ value: String) -> String? {
        stringValue(method: "formatRuntimeLabel", arguments: ["value": value])
    }

    static func tmdbNumericId(_ value: String) -> String? {
        stringValue(method: "tmdbNumericId", arguments: ["id": value])
    }

    static func normalizeContentType(_ value: String) -> String? {
        stringValue(method: "normalizeContentType", arguments: ["value": value])
    }

    static func tmdbContentType(_ contentType: String) -> String {
        stringValue(method: "tmdbContentType", arguments: ["contentType": contentType]) ?? contentType
    }

    static func isSeriesContentType(_ contentType: String) -> Bool {
        (value(method: "isSeriesContentType", arguments: ["value": contentType]) as? Bool) ?? false
    }

    static func tmdbImageUrl(_ path: String?, size: String) -> String? {
        var arguments: [String: Any] = ["size": size]
        if let path {
            arguments["path"] = path
        }
        return value(method: "tmdbImageUrl", arguments: arguments) as? String
    }

    static func tmdbRecommendationsUrl(
        contentType: String,
        tmdbId: String,
        apiKey: String,
        language: String
    ) -> String? {
        value(
            method: "tmdbRecommendationsUrl",
            arguments: [
                "contentType": contentType,
                "tmdbId": tmdbId,
                "apiKey": apiKey,
                "language": language,
            ]
        ) as? String
    }

    static func tmdbBuiltinMetaRequestPlan(
        contentType: String,
        contentId: String,
        apiKey: String,
        language: String
    ) -> [String: Any]? {
        value(
            method: "tmdbBuiltinMetaRequestPlan",
            arguments: ["contentType": contentType, "contentId": contentId, "apiKey": apiKey, "language": language]
        ) as? [String: Any]
    }

    static func tmdbBuiltinMetaUrlsFromFind(
        find: Any,
        contentType: String,
        apiKey: String,
        language: String
    ) -> [String: Any]? {
        value(
            method: "tmdbBuiltinMetaUrlsFromFind",
            arguments: ["find": find, "contentType": contentType, "apiKey": apiKey, "language": language]
        ) as? [String: Any]
    }

    static func tmdbSeasonRequestUrl(contentId: String, season: Int, apiKey: String, language: String) -> String? {
        stringValue(
            method: "tmdbSeasonRequestUrl",
            arguments: ["contentId": contentId, "season": season, "apiKey": apiKey, "language": language]
        )
    }

    static func tmdbFullMetaToMeta(
        detailsJson: String,
        creditsJson: String,
        imagesJson: String,
        externalIdsJson: String,
        extrasJson: String,
        requestedType: String,
        language: String
    ) -> FluxaAppleJsonValue? {
        let result = value(
            method: "tmdbFullMetaToMeta",
            arguments: [
                "detailsJson": detailsJson,
                "creditsJson": creditsJson,
                "imagesJson": imagesJson,
                "externalIdsJson": externalIdsJson,
                "extrasJson": extrasJson,
                "requestedType": requestedType,
                "language": language,
            ]
        )
        return decodeJsonValue(result)
    }

    static func tmdbEpisodesToVideos(seasonJson: String, seriesId: String) -> [FluxaAppleJsonValue] {
        decodeValue(
            method: "tmdbEpisodesToVideos",
            arguments: ["seasonJson": seasonJson, "seriesId": seriesId],
            as: [FluxaAppleJsonValue].self
        ) ?? []
    }

    static func tmdbMergeEnrichment(
        base: FluxaAppleJsonValue,
        tmdb: FluxaAppleJsonValue,
        flags: [String: Bool]
    ) -> FluxaAppleJsonValue? {
        guard let baseData = try? JSONEncoder().encode(base),
              let tmdbData = try? JSONEncoder().encode(tmdb),
              let baseJson = String(data: baseData, encoding: .utf8),
              let tmdbJson = String(data: tmdbData, encoding: .utf8),
              let flagsData = try? JSONSerialization.data(withJSONObject: flags),
              let flagsJson = String(data: flagsData, encoding: .utf8) else {
            return nil
        }
        return decodeJsonValue(value(
            method: "tmdbMergeEnrichment",
            arguments: ["baseJson": baseJson, "tmdbJson": tmdbJson, "flagsJson": flagsJson]
        ))
    }

    static func tmdbItemContentType(
        mediaType: String?,
        hasFirstAirDate: Bool,
        requestedType: String
    ) -> String? {
        value(
            method: "tmdbItemContentType",
            arguments: [
                "mediaType": mediaType as Any,
                "hasFirstAirDate": hasFirstAirDate,
                "requestedType": requestedType,
            ]
        ) as? String
    }

    static func normalizeCatalogType(_ value: String) -> String? {
        stringValue(method: "normalizeCatalogType", arguments: ["value": value])
    }

    static func releaseDateUpcoming(_ released: String, todayIso: String) -> Bool {
        (value(
            method: "releaseDateUpcoming",
            arguments: ["released": released, "todayIso": todayIso]
        ) as? Bool) ?? false
    }

    static func detailAvailableSeasons(seasons: [Int], seasonsCount: Int?) -> [Int] {
        var arguments: [String: Any] = ["seasons": seasons]
        if let seasonsCount {
            arguments["seasonsCount"] = seasonsCount
        }
        return (value(method: "detailAvailableSeasons", arguments: arguments) as? [Int]) ?? [1]
    }

    static func normalizeManifestUrl(_ url: String) -> String {
        stringValue(method: "normalizeManifestUrl", arguments: ["url": url]) ?? url
    }

    static func addonStoreEntriesPlan(
        repositoryAddons: [[String: Any]],
        localUrls: [String],
        disabledKeys: [String],
        localLoaded: Bool,
        refreshingUrl: String? = nil
    ) -> [FluxaCoreAddonStoreEntry] {
        decodeValue(
            method: "addonStoreEntriesPlan",
            arguments: [
                "repositoryAddons": repositoryAddons,
                "localUrls": localUrls,
                "disabledKeys": disabledKeys,
                "isNuvioProfile": false,
                "localLoaded": localLoaded,
                "refreshingUrl": refreshingUrl ?? NSNull()
            ],
            as: [FluxaCoreAddonStoreEntry].self
        ) ?? []
    }

    static func addonProfileMutationPlan(
        profile: [String: Any],
        command: String,
        addonKey: String,
        direction: Int = 0
    ) -> [String: Any]? {
        var arguments: [String: Any] = [
            "profile": profile,
            "command": command,
            "addonKey": addonKey,
            "direction": direction
        ]
        return value(method: "addonProfileMutationPlan", arguments: arguments) as? [String: Any]
    }

    static func pluginUrlAllowed(_ url: String) -> Bool {
        (value(method: "pluginUrlAllowed", arguments: ["url": url]) as? Bool) ?? false
    }

    static func pluginNetworkAddressBytesAllowed(_ bytes: [UInt8]) -> Bool {
        (value(method: "pluginNetworkAddressBytesAllowed", arguments: ["bytes": bytes]) as? Bool) ?? false
    }

    static func resourceUrl(
        transportUrl: String,
        resource: String,
        contentType: String,
        id: String,
        extra: [String: String] = [:]
    ) -> String? {
        var arguments: [String: Any] = [
            "transportUrl": transportUrl,
            "resource": resource,
            "contentType": contentType,
            "id": id
        ]
        if !extra.isEmpty,
           let data = try? JSONSerialization.data(withJSONObject: extra),
           let extraJson = String(data: data, encoding: .utf8) {
            arguments["extraJson"] = extraJson
        }
        return stringValue(method: "buildResourceUrl", arguments: arguments)
    }

    static func parseManifest(body: String, transportUrl: String) -> FluxaCoreAddonManifest? {
        let unknownName = URL(string: transportUrl)?.host ?? "Unknown Addon"
        guard let value = value(
            method: "parseManifest",
            arguments: [
                "body": body,
                "transportUrl": transportUrl,
                "unknownName": unknownName
            ]
        ),
        JSONSerialization.isValidJSONObject(value),
        let data = try? JSONSerialization.data(withJSONObject: value) else {
            return nil
        }
        return try? JSONDecoder().decode(FluxaCoreAddonManifestDescriptor.self, from: data).manifest
    }

    static func parseManifestDescriptor(body: String, transportUrl: String) -> [String: Any]? {
        let unknownName = URL(string: transportUrl)?.host ?? "Unknown Addon"
        return value(
            method: "parseManifest",
            arguments: ["body": body, "transportUrl": transportUrl, "unknownName": unknownName]
        ) as? [String: Any]
    }

    static func parseCatalogItems(body: String, fallbackType: String) -> [FluxaCoreCatalogItem]? {
        decodeValue(
            method: "parseCatalogItems",
            arguments: ["body": body, "fallbackType": fallbackType],
            as: [FluxaCoreCatalogItem].self
        )
    }

    static func resourceFetchPlan(
        manifests: [[String: Any]],
        kind: String,
        query: String? = nil,
        transportUrl: String? = nil,
        contentType: String? = nil,
        id: String? = nil
    ) -> FluxaCoreResourceFetchPlan? {
        var arguments: [String: Any] = ["kind": kind, "addons": manifests]
        if let query { arguments["query"] = query }
        if let transportUrl { arguments["transportUrl"] = transportUrl }
        if let contentType { arguments["contentType"] = contentType }
        if let id { arguments["id"] = id }
        return decodeValue(method: "resourceFetchPlan", arguments: arguments, as: FluxaCoreResourceFetchPlan.self)
    }

    static func resourceFetchExecutionPolicy(
        manifests: [[String: Any]],
        kind: String,
        query: String? = nil,
        transportUrl: String? = nil,
        contentType: String? = nil,
        id: String? = nil
    ) -> FluxaCoreResourceExecutionPolicy? {
        var arguments: [String: Any] = ["kind": kind, "addons": manifests]
        if let query { arguments["query"] = query }
        if let transportUrl { arguments["transportUrl"] = transportUrl }
        if let contentType { arguments["contentType"] = contentType }
        if let id { arguments["id"] = id }
        return decodeValue(
            method: "resourceFetchExecutionPolicy",
            arguments: arguments,
            as: FluxaCoreResourceExecutionPolicy.self
        )
    }

    static func mergeDiscoverSources(_ sources: [[String: Any]]) -> [String: Any]? {
        value(method: "mergeDiscoverSources", arguments: ["sources": sources]) as? [String: Any]
    }

    static func discoverSourceRequests(contentType: String, filters: [String: Any]) -> [[String: Any]]? {
        value(
            method: "discoverSourceRequests",
            arguments: ["contentType": contentType, "filters": filters]
        ) as? [[String: Any]]
    }

    static func addonDescriptorValue(
        manifest: FluxaCoreAddonManifest,
        transportUrl: String
    ) -> [String: Any]? {
        let catalogs: [[String: Any]] = manifest.catalogs.map { catalog in
            let extras: [[String: Any]] = (catalog.extra ?? []).map { extra in
                var value: [String: Any] = [:]
                if let name = extra.name { value["name"] = name }
                if let isRequired = extra.isRequired { value["isRequired"] = isRequired }
                if let options = extra.options { value["options"] = options }
                return value
            }
            var value: [String: Any] = ["extra": extras]
            if let type = catalog.type { value["type"] = type }
            if let id = catalog.id { value["id"] = id }
            if let name = catalog.name { value["name"] = name }
            if let genres = catalog.genres { value["genres"] = genres }
            if catalog.supportsSearch { value["extraSupported"] = ["search"] }
            return value
        }
        return [
            "name": manifest.name,
            "transportUrl": transportUrl,
            "manifest": [
                "id": manifest.id,
                "name": manifest.name,
                "resources": manifest.supportsCatalog ? ["catalog"] : [],
                "catalogs": catalogs
            ]
        ]
    }

    static func parseDirectStreams(body: String) -> [FluxaCoreDirectStream]? {
        decodeValue(
            method: "parseDirectStreams",
            arguments: ["body": body],
            as: [FluxaCoreDirectStream].self
        )
    }

    static func streamSubtitlesResult(stream: FluxaAppleJsonValue) -> FluxaAppleJsonValue {
        guard let streamData = try? JSONEncoder().encode(stream),
              let streamJson = String(data: streamData, encoding: .utf8),
              let result = value(method: "streamSubtitlesResult", argsJson: streamJson),
              JSONSerialization.isValidJSONObject(result),
              let resultData = try? JSONSerialization.data(withJSONObject: result),
              let decoded = try? JSONDecoder().decode(FluxaAppleJsonValue.self, from: resultData) else {
            return .object(["subtitles": .array([])])
        }
        return decoded
    }

    private static func stringValue(method: String, arguments: [String: Any]) -> String? {
        value(method: method, arguments: arguments) as? String
    }

    private static func value(method: String, argsJson: String) -> Any? {
        guard let responseData = coreInvoke(method: method, argsJson: argsJson).data(using: .utf8),
              let response = try? JSONSerialization.jsonObject(with: responseData) as? [String: Any],
              response["ok"] as? Bool == true else {
            return nil
        }
        return response["value"]
    }

    private static func decodeValue<T: Decodable>(method: String, arguments: [String: Any], as type: T.Type) -> T? {
        guard let value = value(method: method, arguments: arguments),
              JSONSerialization.isValidJSONObject(value),
              let data = try? JSONSerialization.data(withJSONObject: value) else {
            return nil
        }
        return try? JSONDecoder().decode(T.self, from: data)
    }

    private static func decodeJsonValue(_ value: Any?) -> FluxaAppleJsonValue? {
        guard let value,
              JSONSerialization.isValidJSONObject(value),
              let data = try? JSONSerialization.data(withJSONObject: value) else {
            return nil
        }
        return try? JSONDecoder().decode(FluxaAppleJsonValue.self, from: data)
    }

    private static func value(method: String, arguments: [String: Any]) -> Any? {
        guard let data = try? JSONSerialization.data(withJSONObject: arguments),
              let argsJson = String(data: data, encoding: .utf8),
              let responseData = coreInvoke(method: method, argsJson: argsJson).data(using: .utf8),
              let response = try? JSONSerialization.jsonObject(with: responseData) as? [String: Any],
              response["ok"] as? Bool == true else {
            return nil
        }
        return response["value"]
    }
}
