import Foundation

struct FluxaAppleAddonCatalogResolver {
    private let session: URLSession
    private let decoder = JSONDecoder()

    init(session: URLSession = .shared) {
        self.session = session
    }

    func resolveRequests(localAddonUrls: [String]) async throws -> [FluxaAppleCatalogRequest] {
        var requests = [FluxaAppleCatalogRequest]()
        var lastError: Error?
        for rawUrl in localAddonUrls {
            do {
                let transportUrl = normalizeManifestUrl(rawUrl)
                guard let manifestUrl = URL(string: transportUrl) else {
                    continue
                }
                let (data, response) = try await session.data(from: manifestUrl)
                guard let httpResponse = response as? HTTPURLResponse,
                      (200..<300).contains(httpResponse.statusCode) else {
                    throw URLError(.badServerResponse)
                }
                let manifest = try decoder.decode(FluxaAppleAddonManifest.self, from: data)
                guard manifest.supportsCatalog else {
                    continue
                }
                requests.append(contentsOf: (manifest.catalogs ?? []).compactMap {
                    makeRequest(catalog: $0, manifest: manifest, transportUrl: transportUrl)
                })
            } catch {
                lastError = error
            }
        }
        if requests.isEmpty, let lastError {
            throw lastError
        }
        return requests
    }

    func resourceUrl(
        transportUrl: String,
        resource: String,
        contentType: String,
        id: String
    ) -> URL? {
        URL(string: resourceUrlString(
            transportUrl: transportUrl,
            resource: resource,
            contentType: contentType,
            id: id
        ))
    }

    private func makeRequest(
        catalog: FluxaAppleAddonCatalog,
        manifest: FluxaAppleAddonManifest,
        transportUrl: String
    ) -> FluxaAppleCatalogRequest? {
        guard let catalogId = catalog.id?.trimmingCharacters(in: .whitespacesAndNewlines),
              !catalogId.isEmpty,
              let contentType = catalog.type?.trimmingCharacters(in: .whitespacesAndNewlines),
              !contentType.isEmpty,
              catalog.supportsInitialLoad,
              let url = resourceUrl(
                  transportUrl: transportUrl,
                  resource: "catalog",
                  contentType: contentType,
                  id: catalogId
              ) else {
            return nil
        }
        let title = catalog.name?.trimmingCharacters(in: .whitespacesAndNewlines)
        let resolvedTitle = title.flatMap { $0.isEmpty ? nil : $0 } ?? catalogId
        return FluxaAppleCatalogRequest(
            id: "\(manifest.id):\(contentType):\(catalogId)",
            title: resolvedTitle,
            url: url,
            contentType: contentType,
            addonTransportUrl: transportUrl,
            catalogType: contentType
        )
    }

    private func normalizeManifestUrl(_ rawUrl: String) -> String {
        let trimmed = rawUrl.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            return ""
        }
        let lower = trimmed.lowercased()
        let withScheme: String
        if lower.hasPrefix("stremio://") {
            withScheme = "https://" + String(trimmed.dropFirst("stremio://".count))
        } else if lower.hasPrefix("http://") || lower.hasPrefix("https://") {
            withScheme = trimmed
        } else {
            withScheme = "https://\(trimmed)"
        }
        if withScheme.lowercased().hasSuffix("manifest.json") {
            return withScheme
        }
        return withScheme.hasSuffix("/") ? "\(withScheme)manifest.json" : "\(withScheme)/manifest.json"
    }

    private func resourceUrlString(
        transportUrl: String,
        resource: String,
        contentType: String,
        id: String
    ) -> String {
        let lower = transportUrl.lowercased()
        let base: String
        if let range = lower.range(of: "manifest.json", options: .backwards) {
            base = String(transportUrl[..<range.lowerBound])
        } else {
            base = transportUrl
        }
        let normalizedBase = base.hasSuffix("/") ? base : "\(base)/"
        return "\(normalizedBase)\(resource)/\(encodePathSegment(contentType))/\(encodePathSegment(id)).json"
    }

    private func encodePathSegment(_ value: String) -> String {
        value.addingPercentEncoding(withAllowedCharacters: .alphanumerics.union(CharacterSet(charactersIn: "-_.*"))) ?? value
    }
}

private struct FluxaAppleAddonManifest: Decodable {
    let id: String
    let resources: [FluxaAppleAddonResource]?
    let catalogs: [FluxaAppleAddonCatalog]?

    var supportsCatalog: Bool {
        resources?.contains(where: { $0.name == "catalog" }) ?? false
    }
}

private struct FluxaAppleAddonResource: Decodable {
    let name: String

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if let name = try? container.decode(String.self) {
            self.name = name
        } else {
            self.name = try container.decode(FluxaAppleAddonResourceObject.self).name
        }
    }
}

private struct FluxaAppleAddonResourceObject: Decodable {
    let name: String
}

private struct FluxaAppleAddonCatalog: Decodable {
    let type: String?
    let id: String?
    let name: String?
    let extra: [FluxaAppleAddonCatalogExtra]?

    var supportsInitialLoad: Bool {
        !(extra?.contains(where: { $0.isRequired == true }) ?? false)
    }
}

private struct FluxaAppleAddonCatalogExtra: Decodable {
    let isRequired: Bool?
}
