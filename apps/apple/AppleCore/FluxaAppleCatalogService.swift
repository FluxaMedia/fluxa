import Foundation
import FluxaCore

final class FluxaAppleCatalogService {
    private let resolver: FluxaAppleAddonCatalogResolver
    private let loader: FluxaAppleCatalogLoader

    init(
        resolver: FluxaAppleAddonCatalogResolver = FluxaAppleAddonCatalogResolver(),
        loader: FluxaAppleCatalogLoader = FluxaAppleCatalogLoader()
    ) {
        self.resolver = resolver
        self.loader = loader
    }

    func loadHomeRows(addonUrls: [String]) async throws -> [FluxaCore.AppleCatalogRowSnapshot] {
        let requests = try await resolver.resolveRequests(localAddonUrls: addonUrls)
        return try await loader.loadRows(requests: requests)
    }

    func loadRows(requests: [FluxaAppleCatalogRequest]) async throws -> [FluxaCore.AppleCatalogRowSnapshot] {
        try await loader.loadRows(requests: requests)
    }

    func loadSearchItems(
        addonUrls: [String],
        query: String
    ) async throws -> [FluxaCore.AppleCatalogItemSnapshot] {
        let requests = try await resolver.resolveSearchRequests(localAddonUrls: addonUrls, query: query)
        return try await loader.loadSearchItems(requests: requests)
    }

    func resolveDiscoverCatalogs(
        addonUrls: [String],
        contentType: String
    ) async throws -> [FluxaAppleDiscoverCatalog] {
        try await resolver.resolveDiscoverCatalogs(localAddonUrls: addonUrls, contentType: contentType)
    }

    func discoverUrl(
        transportUrl: String,
        contentType: String,
        catalogId: String,
        genre: String?
    ) -> URL? {
        resolver.discoverUrl(
            transportUrl: transportUrl,
            contentType: contentType,
            catalogId: catalogId,
            genre: genre
        )
    }
}
