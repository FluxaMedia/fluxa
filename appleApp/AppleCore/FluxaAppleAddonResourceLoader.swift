import Foundation

struct FluxaAppleDirectStream: Sendable {
    let addonName: String
    let title: String
    let url: String
    let requestHeadersJson: String
}

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

    func loadDirectStreams(
        transportUrl: String,
        contentType: String,
        id: String
    ) async throws -> [FluxaAppleDirectStream] {
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
        let root = try JSONSerialization.jsonObject(with: data)
        guard let object = root as? [String: Any],
              let streams = object["streams"] as? [[String: Any]] else {
            throw URLError(.cannotParseResponse)
        }
        let addonName = URL(string: transportUrl)?.host ?? transportUrl
        return streams.compactMap { stream in
            guard let streamUrl = streamUrl(stream) else {
                return nil
            }
            let title = (stream["title"] as? String)?.trimmingCharacters(in: .whitespacesAndNewlines)
            let requestHeaders = ((stream["behaviorHints"] as? [String: Any])?["proxyHeaders"] as? [String: Any])?["request"] as? [String: Any]
            return FluxaAppleDirectStream(
                addonName: addonName,
                title: title?.isEmpty == false ? title! : addonName,
                url: streamUrl,
                requestHeadersJson: requestHeadersJson(requestHeaders)
            )
        }
    }

    private func streamUrl(_ stream: [String: Any]) -> String? {
        if let url = stream["url"] as? String,
           let scheme = URL(string: url)?.scheme?.lowercased(),
           scheme == "http" || scheme == "https" || scheme == "magnet" || scheme == "stremio" {
            return url
        }
        guard let infoHash = stream["infoHash"] as? String,
              !infoHash.isEmpty else {
            return nil
        }
        let fileIndex = (stream["fileIdx"] as? NSNumber)?.intValue
        return fileIndex.map { "stremio://torrent/\(infoHash)/\($0)" } ?? "stremio://torrent/\(infoHash)"
    }

    private func requestHeadersJson(_ headers: [String: Any]?) -> String {
        let normalized = (headers ?? [:]).reduce(into: [String: String]()) { result, entry in
            guard let value = entry.value as? String else {
                return
            }
            result[entry.key] = value
        }
        guard let data = try? JSONSerialization.data(withJSONObject: normalized),
              let json = String(data: data, encoding: .utf8) else {
            return "{}"
        }
        return json
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
