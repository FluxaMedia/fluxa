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
                actionJson: "{\"type\":\"(FluxaHeadlessActionType.homeLoadRequested)\",\"profile\":{\"id\":\"apple-default\"},\"language\":\"en\",\"force\":true}"
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
        let rows = categories.compactMap(FluxaAppleSnapshotMapper.catalogRow)
        FluxaApple.shared.updateCatalogHome(
            snapshot: AppleCatalogHomeSnapshot(rows: rows, isLoading: false)
        )
    }

    private func updateEmptyHome() {
        FluxaApple.shared.updateCatalogHome(
            snapshot: AppleCatalogHomeSnapshot(rows: [], isLoading: false)
        )
    }

}
