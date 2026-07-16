import FluxaShared
import Foundation

@MainActor
final class FluxaAppleCatalogBootstrap {
    private let loader: FluxaAppleCatalogLoader
    private let resolver: FluxaAppleAddonCatalogResolver

    init(
        loader: FluxaAppleCatalogLoader = FluxaAppleCatalogLoader(),
        resolver: FluxaAppleAddonCatalogResolver = FluxaAppleAddonCatalogResolver()
    ) {
        self.loader = loader
        self.resolver = resolver
    }

    func loadRows(requests: [FluxaAppleCatalogRequest]) async throws -> [AppleCatalogRowSnapshot] {
        try await loader.loadRows(requests: requests)
    }

    func loadRows(localAddonUrls: [String]) async throws -> [AppleCatalogRowSnapshot] {
        let requests = try await resolver.resolveRequests(localAddonUrls: localAddonUrls)
        return try await loadRows(requests: requests)
    }

    func loadSearchItems(
        localAddonUrls: [String],
        query: String
    ) async throws -> [AppleCatalogItemSnapshot] {
        let requests = try await resolver.resolveSearchRequests(
            localAddonUrls: localAddonUrls,
            query: query
        )
        return try await loader.loadSearchItems(requests: requests)
    }
}
