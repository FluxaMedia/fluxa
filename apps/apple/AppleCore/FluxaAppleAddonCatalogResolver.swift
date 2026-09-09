import Foundation

struct FluxaAppleAddonCatalogResolver {
    private let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }

    func resolveRequests(localAddonUrls: [String]) async throws -> [FluxaAppleCatalogRequest] {
        var descriptors = [[String: Any]]()
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
                guard let manifest = FluxaCoreStremio.parseManifest(
                    body: String(decoding: data, as: UTF8.self),
                    transportUrl: transportUrl
                ) else {
                    throw URLError(.cannotParseResponse)
                }
                guard manifest.supportsCatalog else {
                    continue
                }
                if let descriptor = FluxaCoreStremio.addonDescriptorValue(
                    manifest: manifest,
                    transportUrl: transportUrl
                ) {
                    descriptors.append(descriptor)
                }
            } catch {
                lastError = error
            }
        }
        let requests = FluxaCoreStremio.resourceFetchPlan(
            manifests: descriptors,
            kind: "home"
        ).map { plan in
            plan.requests.compactMap { request in
                guard let url = URL(string: request.url) else { return nil }
                return FluxaAppleCatalogRequest(
                    id: request.categoryId ?? request.url,
                    title: request.categoryName ?? request.catalogId ?? request.url,
                    url: url,
                    contentType: request.catalogType ?? "",
                    addonTransportUrl: request.transportUrl,
                    catalogType: request.catalogType
                )
            }
        } ?? []
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
        var descriptors = [[String: Any]]()
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
                guard let manifest = FluxaCoreStremio.parseManifest(
                    body: String(decoding: data, as: UTF8.self),
                    transportUrl: transportUrl
                ) else {
                    throw URLError(.cannotParseResponse)
                }
                guard manifest.supportsCatalog else {
                    continue
                }
                if let descriptor = FluxaCoreStremio.addonDescriptorValue(
                    manifest: manifest,
                    transportUrl: transportUrl
                ) {
                    descriptors.append(descriptor)
                }
            } catch {
                lastError = error
            }
        }
        let requests = FluxaCoreStremio.resourceFetchPlan(
            manifests: descriptors,
            kind: "search",
            query: normalizedQuery
        ).map { plan in
            plan.requests.compactMap { request in
                guard let url = URL(string: request.url) else { return nil }
                return FluxaAppleSearchRequest(
                    url: url,
                    contentType: request.catalogType ?? "",
                    addonTransportUrl: request.transportUrl ?? "",
                    catalogType: request.catalogType ?? ""
                )
            }
        } ?? []
        if requests.isEmpty, let lastError {
            throw lastError
        }
        return requests
    }

    func resolveDiscoverCatalogs(
        localAddonUrls: [String],
        contentType: String
    ) async throws -> [FluxaAppleDiscoverCatalog] {
        let normalizedType = normalizeCatalogType(contentType)
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
                guard let manifest = FluxaCoreStremio.parseManifest(
                    body: String(decoding: data, as: UTF8.self),
                    transportUrl: transportUrl
                ) else {
                    throw URLError(.cannotParseResponse)
                }
                guard manifest.supportsCatalog else {
                    continue
                }
                catalogs.append(contentsOf: FluxaCoreStremio.discoverCatalogOptions(
                    manifest: manifest,
                    transportUrl: transportUrl,
                    selectedType: normalizedType
                ).orEmpty().map {
                    FluxaAppleDiscoverCatalog(
                        key: $0.key,
                        label: $0.label,
                        transportUrl: $0.transportUrl,
                        contentType: $0.type,
                        catalogId: $0.id,
                        genres: $0.genres,
                        requiresGenre: $0.requiresGenre
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
        URL(string: FluxaCoreStremio.resourceUrl(
            transportUrl: transportUrl,
            resource: "catalog",
            contentType: contentType,
            id: catalogId,
            extra: genre.map { ["genre": $0] } ?? [:]
        ) ?? "")
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

    private func normalizeManifestUrl(_ rawUrl: String) -> String {
        FluxaCoreStremio.normalizeManifestUrl(rawUrl)
    }

    private func normalizeCatalogType(_ rawType: String) -> String {
        let trimmed = rawType.trimmingCharacters(in: .whitespacesAndNewlines)
        return FluxaCoreStremio.normalizeCatalogType(trimmed) ?? trimmed.lowercased()
    }

    private func resourceUrlString(
        transportUrl: String,
        resource: String,
        contentType: String,
        id: String
    ) -> String {
        FluxaCoreStremio.resourceUrl(
            transportUrl: transportUrl,
            resource: resource,
            contentType: contentType,
            id: id
        ) ?? ""
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
