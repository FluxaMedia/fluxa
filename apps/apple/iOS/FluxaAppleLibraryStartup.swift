import FluxaShared
import Foundation

@MainActor
final class FluxaAppleLibraryStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator

    init(coordinator: FluxaAppleHeadlessCoordinator) {
        self.coordinator = coordinator
    }

    func refresh() async {
        do {
            let result = try await coordinator.dispatch(
                actionJson: "{\"type\":\"libraryHydrateRequested\",\"profileId\":\"apple-default\"}"
            )
            guard case .object(let library)? = result.state["library"],
                  case .array(let watchlist)? = library["watchlist"] else {
                FluxaApple.shared.updateLibrary(snapshot: AppleLibrarySnapshot(planned: [], completed: [], favorites: [], isLoading: false))
                return
            }
            FluxaApple.shared.updateLibrary(snapshot: AppleLibrarySnapshot(planned: watchlist.compactMap(item), completed: [], favorites: [], isLoading: false))
        } catch {
            FluxaApple.shared.updateLibrary(snapshot: AppleLibrarySnapshot(planned: [], completed: [], favorites: [], isLoading: false))
        }
    }

    private func item(_ value: FluxaAppleJsonValue) -> AppleLibraryItemSnapshot? {
        guard case .object(let item) = value,
              let id = text(item["id"]),
              let type = text(item["type"]),
              let title = text(item["name"]) else { return nil }
        return AppleLibraryItemSnapshot(id: id, type: type, title: title, subtitle: text(item["releaseInfo"]) ?? "", posterUrl: text(item["poster"]), logoUrl: text(item["logo"]), addonTransportUrl: text(item["addonTransportUrl"]), catalogType: text(item["catalogType"]))
    }

    private func text(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else { return nil }
        return text
    }
}
