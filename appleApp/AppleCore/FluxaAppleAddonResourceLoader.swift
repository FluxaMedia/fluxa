import Foundation

final class FluxaAppleAddonResourceLoader {
    private let resolver: FluxaAppleAddonCatalogResolver
    private let session: URLSession

    init(
        resolver: FluxaAppleAddonCatalogResolver = FluxaAppleAddonCatalogResolver(),
        session: URLSession = .shared
    ) {
        self.resolver = resolver
        self.session = session
    }

    func loadMeta(
        transportUrl: String,
        contentType: String,
        id: String
    ) async throws -> FluxaAppleJsonValue {
        guard let url = resolver.resourceUrl(
            transportUrl: transportUrl,
            resource: "meta",
            contentType: contentType,
            id: id
        ) else {
            throw URLError(.badURL)
        }
        let (data, response) = try await session.data(from: url)
        guard let httpResponse = response as? HTTPURLResponse,
              (200..<300).contains(httpResponse.statusCode) else {
            throw URLError(.badServerResponse)
        }
        let root = try JSONSerialization.jsonObject(with: data)
        guard let object = root as? [String: Any], let meta = object["meta"] else {
            throw URLError(.cannotParseResponse)
        }
        return try FluxaAppleJsonValue(any: meta)
    }
}

private extension FluxaAppleJsonValue {
    init(any value: Any) throws {
        switch value {
        case is NSNull:
            self = .null
        case let value as Bool:
            self = .boolean(value)
        case let value as NSNumber:
            self = .number(value.doubleValue)
        case let value as String:
            self = .string(value)
        case let value as [Any]:
            self = .array(try value.map { try FluxaAppleJsonValue(any: $0) })
        case let value as [String: Any]:
            self = .object(try value.mapValues { try FluxaAppleJsonValue(any: $0) })
        default:
            throw URLError(.cannotParseResponse)
        }
    }
}
