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

    func loadAddonDescriptors(addonUrls: [String]) async throws -> [[String: Any]] {
        try await resolver.loadAddonDescriptors(localAddonUrls: addonUrls)
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

}
