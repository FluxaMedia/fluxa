import FluxaShared
import Foundation

@MainActor
final class FluxaAppleDiscoverStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator
    private let encoder = JSONEncoder()

    init(coordinator: FluxaAppleHeadlessCoordinator) {
        self.coordinator = coordinator
    }

    func discover(request: AppleDiscoverRequestSnapshot) async {
        do {
            let action = FluxaAppleDiscoverAction(
                type: FluxaHeadlessActionType.discoverRequested,
                contentType: request.contentType,
                loadCatalogFilters: true,
                filters: FluxaAppleDiscoverFilters(
                    catalogKey: request.catalogKey,
                    extra: request.genre.map { ["genre": $0] } ?? [:]
                ),
                language: "en",
                profile: FluxaAppleDiscoverProfile(id: "apple-default")
            )
            let actionJson = String(decoding: try encoder.encode(action), as: UTF8.self)
            let result = try await coordinator.dispatch(actionJson: actionJson)
            updateSharedDiscover(result: result, request: request)
        } catch {
            updateSharedDiscover(items: [], request: AppleDiscoverRequestSnapshot(contentType: "movie", catalogKey: nil, genre: nil))
        }
    }

    private func updateSharedDiscover(
        result: FluxaAppleHeadlessResult,
        request: AppleDiscoverRequestSnapshot
    ) {
        let items: [AppleCatalogItemSnapshot]
        if case .object(let discover)? = result.state["discover"],
           case .array(let values)? = discover["results"] {
            items = values.compactMap(FluxaAppleSnapshotMapper.catalogItem)
        } else {
            items = []
        }
        let catalogOptions = filterOptions(
            state: result.state,
            key: "catalogs",
            idKey: "key"
        )
        let genreOptions = filterOptions(
            state: result.state,
            key: "genres",
            idKey: "id"
        )
        updateSharedDiscover(
            items: items,
            catalogOptions: catalogOptions,
            genreOptions: genreOptions,
            request: request
        )
    }

    private func updateSharedDiscover(
        items: [AppleCatalogItemSnapshot],
        catalogOptions: [AppleDiscoverFilterOptionSnapshot] = [],
        genreOptions: [AppleDiscoverFilterOptionSnapshot] = [],
        request: AppleDiscoverRequestSnapshot
    ) {
        FluxaApple.shared.updateDiscover(snapshot: AppleDiscoverSnapshot(request: request, catalogOptions: catalogOptions, genreOptions: genreOptions, results: items, isLoading: false))
    }

    private func filterOptions(
        state: [String: FluxaAppleJsonValue],
        key: String,
        idKey: String
    ) -> [AppleDiscoverFilterOptionSnapshot] {
        guard case .object(let discover)? = state["discover"],
              case .array(let values)? = discover[key] else {
            return []
        }
        return values.compactMap { value in
            guard case .object(let option) = value,
                  let label = text(option["label"]) else {
                return nil
            }
            return AppleDiscoverFilterOptionSnapshot(id: text(option[idKey]), label: label)
        }
    }

}

private struct FluxaAppleDiscoverAction: Encodable {
    let type: String
    let contentType: String
    let loadCatalogFilters: Bool
    let filters: FluxaAppleDiscoverFilters
    let language: String
    let profile: FluxaAppleDiscoverProfile
}

private struct FluxaAppleDiscoverFilters: Encodable {
    let catalogKey: String?
    let extra: [String: String]
}

private struct FluxaAppleDiscoverProfile: Encodable {
    let id: String
}
