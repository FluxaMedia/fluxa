import Foundation
import FluxaCore

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


    func loadSubtitleUrls(
        transportUrl: String,
        contentType: String,
        id: String
    ) async throws -> [String] {
        guard let url = resolver.resourceUrl(
            transportUrl: transportUrl,
            resource: "subtitles",
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
        guard let root = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              let subtitles = root["subtitles"] as? [[String: Any]] else {
            return []
        }
        return subtitles.compactMap { subtitle in
            if let direct = subtitle["url"] as? String, !direct.isEmpty {
                return direct
            }
            if let attributes = subtitle["attributes"] as? [String: Any],
               let nested = attributes["url"] as? String, !nested.isEmpty {
                return nested
            }
            return nil
        }
    }

    func loadDirectStreams(
        transportUrl: String,
        contentType: String,
        id: String
    ) async throws -> [AppleDetailStreamSnapshot] {
        guard let url = resolver.resourceUrl(
            transportUrl: transportUrl,
            resource: "stream",
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
        let addonName = URL(string: transportUrl)?.host ?? transportUrl
        guard let streams = FluxaCoreStremio.parseDirectStreams(
            body: String(decoding: data, as: UTF8.self)
        ) else {
            throw URLError(.cannotParseResponse)
        }
        return streams.compactMap { stream in
            guard let data = try? JSONSerialization.data(withJSONObject: stream.requestHeaders),
                  let requestHeadersJson = String(data: data, encoding: .utf8) else {
                return nil
            }
            return AppleDetailStreamSnapshot(
                addonName: addonName,
                title: stream.title ?? addonName,
                playableUrl: stream.playableUrl,
                requestHeadersJson: requestHeadersJson
            )
        }
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
