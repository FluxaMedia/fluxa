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
            let filtersAction = FluxaAppleDiscoverFiltersAction(
                type: "discoverCatalogFiltersRequested",
                contentType: request.contentType,
                selectedCatalogKey: request.catalogKey,
                language: "en",
                profile: FluxaAppleDiscoverProfile(id: "apple-default")
            )
            let filtersActionJson = String(decoding: try encoder.encode(filtersAction), as: UTF8.self)
            let filtersResult = try await coordinator.dispatch(actionJson: filtersActionJson)
            let action = FluxaAppleDiscoverAction(
                type: "discoverRequested",
                contentType: request.contentType,
                filters: FluxaAppleDiscoverFilters(
                    catalogKey: request.catalogKey,
                    genre: request.genre
                ),
                language: "en",
                profile: FluxaAppleDiscoverProfile(id: "apple-default")
            )
            let actionJson = String(decoding: try encoder.encode(action), as: UTF8.self)
            let result = try await coordinator.dispatch(actionJson: actionJson)
            updateSharedDiscover(result: result, filtersResult: filtersResult, request: request)
        } catch {
            updateSharedDiscover(items: [], request: AppleDiscoverRequestSnapshot(contentType: "movie", catalogKey: nil, genre: nil))
        }
    }

    private func updateSharedDiscover(
        result: FluxaAppleHeadlessResult,
        filtersResult: FluxaAppleHeadlessResult,
        request: AppleDiscoverRequestSnapshot
    ) {
        let items: [AppleCatalogItemSnapshot]
        if case .object(let discover)? = result.state["discover"],
           case .array(let values)? = discover["results"] {
            items = values.compactMap(sharedItem)
        } else {
            items = []
        }
        let catalogOptions = filterOptions(
            state: filtersResult.state,
            key: "catalogs",
            idKey: "key"
        )
        let genreOptions = filterOptions(
            state: filtersResult.state,
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

    private func sharedItem(_ value: FluxaAppleJsonValue) -> AppleCatalogItemSnapshot? {
        guard case .object(let item) = value,
              let id = text(item["id"]),
              let type = text(item["type"]),
              let title = text(item["name"]) else {
            return nil
        }
        return AppleCatalogItemSnapshot(id: id, type: type, title: title, subtitle: text(item["releaseInfo"]) ?? "", artworkUrl: text(item["poster"]), logoUrl: text(item["logo"]), addonTransportUrl: text(item["addonTransportUrl"]), catalogType: text(item["catalogType"]), progress: nil, topTenRank: nil)
    }

    private func text(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else {
            return nil
        }
        return text
    }
}

private struct FluxaAppleDiscoverAction: Encodable {
    let type: String
    let contentType: String
    let filters: FluxaAppleDiscoverFilters
    let language: String
    let profile: FluxaAppleDiscoverProfile
}

private struct FluxaAppleDiscoverFiltersAction: Encodable {
    let type: String
    let contentType: String
    let selectedCatalogKey: String?
    let language: String
    let profile: FluxaAppleDiscoverProfile
}

private struct FluxaAppleDiscoverFilters: Encodable {
    let catalogKey: String?
    let genre: String?
}

private struct FluxaAppleDiscoverProfile: Encodable {
    let id: String
}
