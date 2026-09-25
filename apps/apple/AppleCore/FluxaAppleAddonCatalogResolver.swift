import Foundation

struct FluxaAppleAddonCatalogResolver {
    private let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }

    func loadAddonDescriptors(localAddonUrls: [String]) async throws -> [[String: Any]] {
        try await addonDescriptors(localAddonUrls: localAddonUrls)
    }

    func resolveRequests(localAddonUrls: [String]) async throws -> [FluxaAppleCatalogRequest] {
        let descriptors = try await addonDescriptors(localAddonUrls: localAddonUrls)
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
        let descriptors = try await addonDescriptors(localAddonUrls: localAddonUrls)
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
        return requests
    }

    private func addonDescriptors(localAddonUrls: [String]) async throws -> [[String: Any]] {
        var descriptors = [[String: Any]]()
        var lastError: Error?
        for rawUrl in localAddonUrls {
            do {
                let transportUrl = normalizeManifestUrl(rawUrl)
                guard let manifestUrl = URL(string: transportUrl) else { continue }
                let (data, response) = try await session.data(from: manifestUrl)
                guard let httpResponse = response as? HTTPURLResponse,
                      (200..<300).contains(httpResponse.statusCode) else {
                    throw URLError(.badServerResponse)
                }
                guard let descriptor = FluxaCoreStremio.parseManifestDescriptor(
                    body: String(decoding: data, as: UTF8.self),
                    transportUrl: transportUrl
                ) else {
                    throw URLError(.cannotParseResponse)
                }
                descriptors.append(descriptor)
            } catch {
                lastError = error
            }
        }
        if descriptors.isEmpty, let lastError { throw lastError }
        return descriptors
    }

    private func normalizeManifestUrl(_ rawUrl: String) -> String {
        FluxaCoreStremio.normalizeManifestUrl(rawUrl)
    }
}

struct FluxaAppleSearchRequest: Sendable {
    let url: URL
    let contentType: String
    let addonTransportUrl: String
    let catalogType: String
}
