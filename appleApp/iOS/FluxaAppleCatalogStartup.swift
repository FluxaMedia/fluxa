import FluxaShared

@MainActor
final class FluxaAppleCatalogStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator

    init(coordinator: FluxaAppleHeadlessCoordinator) {
        self.coordinator = coordinator
    }

    func refresh() async {
        do {
            let result = try await coordinator.dispatch(
                actionJson: "{\"type\":\"homeLoadRequested\",\"profile\":{\"id\":\"apple-default\"},\"language\":\"en\",\"force\":true}"
            )
            updateSharedHome(result: result)
        } catch {
            updateEmptyHome()
        }
    }

    private func updateSharedHome(result: FluxaAppleHeadlessResult) {
        guard case .object(let home)? = result.state["home"],
              case .array(let categories)? = home["categories"] else {
            updateEmptyHome()
            return
        }
        let rows = categories.compactMap(sharedRow)
        FluxaApple.shared.updateCatalogHome(
            snapshot: AppleCatalogHomeSnapshot(rows: rows, isLoading: false)
        )
    }

    private func updateEmptyHome() {
        FluxaApple.shared.updateCatalogHome(
            snapshot: AppleCatalogHomeSnapshot(rows: [], isLoading: false)
        )
    }

    private func sharedRow(_ category: FluxaAppleJsonValue) -> AppleCatalogRowSnapshot? {
        guard case .object(let value) = category,
              let id = string(value["id"]),
              let title = string(value["name"]),
              case .array(let items)? = value["items"] else {
            return nil
        }
        return AppleCatalogRowSnapshot(
            id: id,
            title: title,
            items: items.compactMap(sharedItem),
            canLoadMore: false
        )
    }

    private func sharedItem(_ item: FluxaAppleJsonValue) -> AppleCatalogItemSnapshot? {
        guard case .object(let value) = item,
              let id = string(value["id"]),
              let type = string(value["type"]),
              let title = string(value["name"]) else {
            return nil
        }
        return AppleCatalogItemSnapshot(
            id: id,
            type: type,
            title: title,
            subtitle: string(value["releaseInfo"]) ?? "",
            artworkUrl: string(value["poster"]),
            logoUrl: string(value["logo"]),
            addonTransportUrl: string(value["addonTransportUrl"]),
            catalogType: string(value["catalogType"]),
            progress: nil,
            topTenRank: nil
        )
    }

    private func string(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else {
            return nil
        }
        return text
    }
}
