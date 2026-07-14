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

    func resolveSearchRequests(
        localAddonUrls: [String],
        query: String
    ) async throws -> [FluxaAppleSearchRequest] {
        let normalizedQuery = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedQuery.isEmpty else {
            return []
        }
        var requests = [FluxaAppleSearchRequest]()
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
                    makeSearchRequest(catalog: $0, transportUrl: transportUrl, query: normalizedQuery)
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

    func resolveDiscoverCatalogs(
        localAddonUrls: [String],
        contentType: String
    ) async throws -> [FluxaAppleDiscoverCatalog] {
        let normalizedType = contentType.lowercased()
        var catalogs = [FluxaAppleDiscoverCatalog]()
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
                catalogs.append(contentsOf: (manifest.catalogs ?? []).compactMap { catalog in
                    makeDiscoverCatalog(
                        catalog: catalog,
                        manifest: manifest,
                        transportUrl: transportUrl,
                        contentType: normalizedType
                    )
                })
            } catch {
                lastError = error
            }
        }
        if catalogs.isEmpty, let lastError {
            throw lastError
        }
        return catalogs
    }

    func discoverUrl(
        transportUrl: String,
        contentType: String,
        catalogId: String,
        genre: String?
    ) -> URL? {
        let lower = transportUrl.lowercased()
        let base: String
        if let range = lower.range(of: "manifest.json", options: .backwards) {
            base = String(transportUrl[..<range.lowerBound])
        } else {
            base = transportUrl
        }
        let normalizedBase = base.hasSuffix("/") ? base : "\(base)/"
        let extra = genre.map { "/genre=\(encodePathSegment($0))" } ?? ""
        return URL(string: "\(normalizedBase)catalog/\(encodePathSegment(contentType))/\(encodePathSegment(catalogId))\(extra).json")
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

    private func makeSearchRequest(
        catalog: FluxaAppleAddonCatalog,
        transportUrl: String,
        query: String
    ) -> FluxaAppleSearchRequest? {
        guard let catalogId = catalog.id?.trimmingCharacters(in: .whitespacesAndNewlines),
              !catalogId.isEmpty,
              let contentType = catalog.type?.trimmingCharacters(in: .whitespacesAndNewlines),
              !contentType.isEmpty,
              catalog.supportsSearch,
              let url = searchUrl(
                  transportUrl: transportUrl,
                  contentType: contentType,
                  catalogId: catalogId,
                  query: query
              ) else {
            return nil
        }
        return FluxaAppleSearchRequest(
            url: url,
            contentType: contentType,
            addonTransportUrl: transportUrl,
            catalogType: contentType
        )
    }

    private func makeDiscoverCatalog(
        catalog: FluxaAppleAddonCatalog,
        manifest: FluxaAppleAddonManifest,
        transportUrl: String,
        contentType: String
    ) -> FluxaAppleDiscoverCatalog? {
        guard let catalogId = catalog.id?.trimmingCharacters(in: .whitespacesAndNewlines),
              !catalogId.isEmpty,
              let catalogType = catalog.type?.trimmingCharacters(in: .whitespacesAndNewlines).lowercased(),
              catalogType == contentType,
              !catalog.hasRequiredExtraExceptGenre else {
            return nil
        }
        let label = catalog.name?.trimmingCharacters(in: .whitespacesAndNewlines)
            .flatMap { $0.isEmpty ? nil : $0 } ?? catalogId
        let genres = (catalog.genres ?? []) + (catalog.extra ?? [])
            .filter { $0.name?.lowercased() == "genre" }
            .flatMap { $0.options ?? [] }
        let uniqueGenres = Array(Set(genres.map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }.filter { !$0.isEmpty })).sorted()
        return FluxaAppleDiscoverCatalog(
            key: "discover:\(manifest.id):\(catalogType):\(catalogId)",
            label: label,
            transportUrl: transportUrl,
            contentType: catalogType,
            catalogId: catalogId,
            genres: uniqueGenres,
            requiresGenre: catalog.extra?.contains { $0.name?.lowercased() == "genre" && $0.isRequired == true } == true
        )
    }

    private func searchUrl(
        transportUrl: String,
        contentType: String,
        catalogId: String,
        query: String
    ) -> URL? {
        let lower = transportUrl.lowercased()
        let base: String
        if let range = lower.range(of: "manifest.json", options: .backwards) {
            base = String(transportUrl[..<range.lowerBound])
        } else {
            base = transportUrl
        }
        let normalizedBase = base.hasSuffix("/") ? base : "\(base)/"
        let encodedQuery = encodePathSegment(query)
        return URL(string: "\(normalizedBase)catalog/\(encodePathSegment(contentType))/\(encodePathSegment(catalogId))/search=\(encodedQuery).json")
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
    let genres: [String]?
    let extra: [FluxaAppleAddonCatalogExtra]?
    let extraSupported: [String]?

    var supportsInitialLoad: Bool {
        !(extra?.contains(where: { $0.isRequired == true }) ?? false)
    }

    var supportsSearch: Bool {
        extraSupported?.contains("search") == true || extra?.contains(where: { $0.name == "search" }) == true
    }

    var hasRequiredExtraExceptGenre: Bool {
        extra?.contains { $0.isRequired == true && $0.name?.lowercased() != "genre" } ?? false
    }
}

private struct FluxaAppleAddonCatalogExtra: Decodable {
    let name: String?
    let isRequired: Bool?
    let options: [String]?
}

struct FluxaAppleDiscoverCatalog: Sendable {
    let key: String
    let label: String
    let transportUrl: String
    let contentType: String
    let catalogId: String
    let genres: [String]
    let requiresGenre: Bool
}

struct FluxaAppleSearchRequest: Sendable {
    let url: URL
    let contentType: String
    let addonTransportUrl: String
    let catalogType: String
}
