import Foundation

struct FluxaCoreAddonCatalogExtra: Decodable {
    let name: String?
    let isRequired: Bool?
    let options: [String]?
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
    static func normalizeManifestUrl(_ url: String) -> String {
        stringValue(method: "normalizeManifestUrl", arguments: ["url": url]) ?? url
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

    static func parseCatalogItems(body: String, fallbackType: String) -> [FluxaCoreCatalogItem]? {
        decodeValue(
            method: "parseCatalogItems",
            arguments: ["body": body, "fallbackType": fallbackType],
            as: [FluxaCoreCatalogItem].self
        )
    }

    static func parseDirectStreams(body: String) -> [FluxaCoreDirectStream]? {
        decodeValue(
            method: "parseDirectStreams",
            arguments: ["body": body],
            as: [FluxaCoreDirectStream].self
        )
    }

    private static func stringValue(method: String, arguments: [String: Any]) -> String? {
        value(method: method, arguments: arguments) as? String
    }

    private static func decodeValue<T: Decodable>(method: String, arguments: [String: Any], as type: T.Type) -> T? {
        guard let value = value(method: method, arguments: arguments),
              JSONSerialization.isValidJSONObject(value),
              let data = try? JSONSerialization.data(withJSONObject: value) else {
            return nil
        }
        return try? JSONDecoder().decode(T.self, from: data)
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
