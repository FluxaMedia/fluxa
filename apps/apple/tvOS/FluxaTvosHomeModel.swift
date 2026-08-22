import Foundation
import SwiftUI

@MainActor
final class FluxaTvosHomeModel: ObservableObject {
    struct Row: Identifiable {
        let id: String
        let title: String
        let items: [Item]
    }

    struct Item: Identifiable {
        let id: String
        let title: String
        let subtitle: String
        let artworkUrl: URL?
    }

    @Published private(set) var rows: [Row] = []
    @Published private(set) var isLoading = false

    private let coordinator: FluxaAppleHeadlessCoordinator

    init(runtime: FluxaAppleHeadlessRuntime) {
        let configurationStore = FluxaAppleAddonConfigurationStore()
        let handler = FluxaTvosEffectHandler(configurationStore: configurationStore)
        coordinator = FluxaAppleHeadlessCoordinator(
            runtime: runtime,
            executor: FluxaApplePlatformEffectExecutor(handler: handler)
        )
    }

    func load() async {
        guard !isLoading else {
            return
        }
        isLoading = true
        defer { isLoading = false }
        do {
            let result = try await coordinator.dispatch(
                actionJson: "{\"type\":\"homeLoadRequested\",\"profile\":{\"id\":\"apple-default\"},\"language\":\"en\",\"force\":true}"
            )
            rows = rows(from: result)
        } catch {
            rows = []
        }
    }

    private func rows(from result: FluxaAppleHeadlessResult) -> [Row] {
        guard case .object(let home)? = result.state["home"],
              case .array(let categories)? = home["categories"] else {
            return []
        }
        return categories.compactMap(row)
    }

    private func row(_ value: FluxaAppleJsonValue) -> Row? {
        guard case .object(let category) = value,
              let id = text(category["id"]),
              let title = text(category["name"]),
              case .array(let values)? = category["items"] else {
            return nil
        }
        return Row(id: id, title: title, items: values.compactMap(item))
    }

    private func item(_ value: FluxaAppleJsonValue) -> Item? {
        guard case .object(let meta) = value,
              let id = text(meta["id"]),
              let title = text(meta["name"]) else {
            return nil
        }
        return Item(
            id: id,
            title: title,
            subtitle: text(meta["releaseInfo"]) ?? "",
            artworkUrl: text(meta["poster"]).flatMap { URL(string: $0) }
        )
    }

    private func text(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else {
            return nil
        }
        return text
    }
}
