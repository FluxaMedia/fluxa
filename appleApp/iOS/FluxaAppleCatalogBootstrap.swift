import FluxaShared
import Foundation

@MainActor
final class FluxaAppleCatalogBootstrap {
    private let loader: FluxaAppleCatalogLoader

    init(loader: FluxaAppleCatalogLoader = FluxaAppleCatalogLoader()) {
        self.loader = loader
    }

    func refresh(requests: [FluxaAppleCatalogRequest]) async throws {
        let snapshot = try await loader.loadSnapshot(requests: requests)
        FluxaApple.shared.updateCatalogHomeJson(homeJson: snapshot)
    }
}
