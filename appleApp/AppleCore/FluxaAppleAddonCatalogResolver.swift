import FluxaShared
import Foundation

struct FluxaAppleAddonCatalogResolver {
    private let session: URLSession

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
                guard let manifest = FluxaApple.shared.parseAddonManifest(body: String(decoding: data, as: UTF8.self)) else {
                    throw URLError(.cannotParseResponse)
                }
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
                guard let manifest = FluxaApple.shared.parseAddonManifest(body: String(decoding: data, as: UTF8.self)) else {
                    throw URLError(.cannotParseResponse)
                }
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
                guard let manifest = FluxaApple.shared.parseAddonManifest(body: String(decoding: data, as: UTF8.self)) else {
                    throw URLError(.cannotParseResponse)
                }
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
        URL(string: FluxaApple.shared.addonCatalogUrl(
            transportUrl: transportUrl,
            contentType: contentType,
            catalogId: catalogId,
            extraName: genre == nil ? nil : "genre",
            extraValue: genre
        ))
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
        catalog: AppleAddonCatalogSnapshot,
        manifest: AppleAddonManifestSnapshot,
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
        catalog: AppleAddonCatalogSnapshot,
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
        catalog: AppleAddonCatalogSnapshot,
        manifest: AppleAddonManifestSnapshot,
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
        let trimmedName = catalog.name?.trimmingCharacters(in: .whitespacesAndNewlines)
        let label: String
        if let trimmedName, !trimmedName.isEmpty {
            label = trimmedName
        } else {
            label = catalogId
        }
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
        URL(string: FluxaApple.shared.addonCatalogUrl(
            transportUrl: transportUrl,
            contentType: contentType,
            catalogId: catalogId,
            extraName: "search",
            extraValue: query
        ))
    }

    private func normalizeManifestUrl(_ rawUrl: String) -> String {
        FluxaApple.shared.normalizeAddonManifestUrl(rawUrl: rawUrl)
    }

    private func resourceUrlString(
        transportUrl: String,
        resource: String,
        contentType: String,
        id: String
    ) -> String {
        FluxaApple.shared.addonResourceUrl(
            transportUrl: transportUrl,
            resource: resource,
            contentType: contentType,
            id: id
        )
    }
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
