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

    func refresh(requests: [FluxaAppleCatalogRequest]) async throws {
        let snapshot = try await loader.loadSnapshot(requests: requests)
        FluxaApple.shared.updateCatalogHomeJson(homeJson: snapshot)
    }

    func refresh(localAddonUrls: [String]) async throws {
        let requests = try await resolver.resolveRequests(localAddonUrls: localAddonUrls)
        try await refresh(requests: requests)
    }
}
