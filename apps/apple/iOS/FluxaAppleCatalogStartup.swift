import FluxaShared
import Foundation

@MainActor
final class FluxaAppleCatalogStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator
    private let encoder = JSONEncoder()

    init(coordinator: FluxaAppleHeadlessCoordinator) {
        self.coordinator = coordinator
    }

    func refresh() async {
        do {
            let action = FluxaAppleCatalogHomeAction(
                type: FluxaHeadlessActionType.homeLoadRequested,
                profile: FluxaAppleCatalogProfile(id: "apple-default"),
                language: "en",
                force: true
            )
            let actionJson = String(decoding: try encoder.encode(action), as: UTF8.self)
            let result = try await coordinator.dispatch(
                actionJson: actionJson
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

private struct FluxaAppleCatalogHomeAction: Encodable {
    let type: String
    let profile: FluxaAppleCatalogProfile
    let language: String
    let force: Bool
}

private struct FluxaAppleCatalogProfile: Encodable {
    let id: String
}
