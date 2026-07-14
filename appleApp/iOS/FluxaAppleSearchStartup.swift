import FluxaShared
import Foundation

@MainActor
final class FluxaAppleSearchStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator
    private let encoder = JSONEncoder()

    init(coordinator: FluxaAppleHeadlessCoordinator) {
        self.coordinator = coordinator
    }

    func search(query: String) async {
        let normalizedQuery = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedQuery.isEmpty else {
            FluxaApple.shared.updateSearch(snapshot: AppleSearchSnapshot(query: "", results: [], isLoading: false))
            return
        }
        do {
            let action = FluxaAppleSearchAction(
                type: "searchRequested",
                query: normalizedQuery,
                language: "en",
                profile: FluxaAppleSearchProfile(id: "apple-default")
            )
            let actionJson = String(decoding: try encoder.encode(action), as: UTF8.self)
            let result = try await coordinator.dispatch(actionJson: actionJson)
            updateSharedSearch(result: result, query: normalizedQuery)
        } catch {
            FluxaApple.shared.updateSearch(snapshot: AppleSearchSnapshot(query: normalizedQuery, results: [], isLoading: false))
        }
    }

    private func updateSharedSearch(result: FluxaAppleHeadlessResult, query: String) {
        let results: [AppleCatalogItemSnapshot]
        if case .object(let search)? = result.state["search"],
           case .array(let values)? = search["results"] {
            results = values.compactMap(sharedItem)
        } else {
            results = []
        }
        FluxaApple.shared.updateSearch(snapshot: AppleSearchSnapshot(query: query, results: results, isLoading: false))
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

private struct FluxaAppleSearchAction: Encodable {
    let type: String
    let query: String
    let language: String
    let profile: FluxaAppleSearchProfile
}

private struct FluxaAppleSearchProfile: Encodable {
    let id: String
}
