import Foundation
import FluxaShared

final class FluxaAppleAddonConfigurationStore {
    static let localAddonUrlsKey = "fluxa.apple.localAddonUrls"

    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    func localAddonUrls() -> [String] {
        let stored = defaults.stringArray(forKey: Self.localAddonUrlsKey)
        let values = stored ?? ["https://v3-cinemeta.strem.io/manifest.json"]
        return values.filter { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }
    }

    func save(localAddonUrls: [String]) {
        let normalized = localAddonUrls.map {
            $0.trimmingCharacters(in: .whitespacesAndNewlines)
        }.filter { !$0.isEmpty }
        defaults.set(normalized, forKey: Self.localAddonUrlsKey)
    }
}

@MainActor
final class FluxaAppleCatalogStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator
    private let encoder = JSONEncoder()

    init(
        runtime: FluxaAppleHeadlessRuntime,
        configurationStore: FluxaAppleAddonConfigurationStore = FluxaAppleAddonConfigurationStore(),
        bootstrap: FluxaAppleCatalogBootstrap = FluxaAppleCatalogBootstrap()
    ) {
        let handler = FluxaAppleHomeEffectHandler(
            configurationStore: configurationStore,
            catalogBootstrap: bootstrap
        )
        coordinator = FluxaAppleHeadlessCoordinator(
            runtime: runtime,
            executor: FluxaApplePlatformEffectExecutor(handler: handler)
        )
    }

    func refresh() async {
        do {
            let result = try await coordinator.dispatch(
                actionJson: "{\"type\":\"homeLoadRequested\",\"profile\":{\"id\":\"apple-default\"},\"language\":\"en\",\"force\":true}"
            )
            updateSharedHome(result: result)
        } catch {
            FluxaApple.shared.updateCatalogHomeJson(homeJson: "{\"rows\":[],\"isLoading\":false}")
        }
    }

    private func updateSharedHome(result: FluxaAppleHeadlessResult) {
        guard case .object(let home)? = result.state["home"],
              case .array(let categories)? = home["categories"] else {
            FluxaApple.shared.updateCatalogHomeJson(homeJson: "{\"rows\":[],\"isLoading\":false}")
            return
        }
        let rows = categories.compactMap(sharedRow)
        let snapshot = FluxaAppleJsonValue.object([
            "rows": .array(rows),
            "isLoading": .boolean(false)
        ])
        guard let data = try? encoder.encode(snapshot) else {
            return
        }
        FluxaApple.shared.updateCatalogHomeJson(homeJson: String(decoding: data, as: UTF8.self))
    }

    private func sharedRow(_ category: FluxaAppleJsonValue) -> FluxaAppleJsonValue? {
        guard case .object(let value) = category,
              let id = string(value["id"]),
              let title = string(value["name"]),
              case .array(let items)? = value["items"] else {
            return nil
        }
        return .object([
            "id": .string(id),
            "title": .string(title),
            "canLoadMore": .boolean(false),
            "items": .array(items.compactMap(sharedItem))
        ])
    }

    private func sharedItem(_ item: FluxaAppleJsonValue) -> FluxaAppleJsonValue? {
        guard case .object(let value) = item,
              let id = string(value["id"]),
              let type = string(value["type"]),
              let title = string(value["name"]) else {
            return nil
        }
        return .object([
            "id": .string(id),
            "type": .string(type),
            "title": .string(title),
            "subtitle": value["releaseInfo"] ?? .string(""),
            "artworkUrl": value["poster"] ?? .null,
            "logoUrl": value["logo"] ?? .null,
            "addonTransportUrl": value["addonTransportUrl"] ?? .null,
            "catalogType": value["catalogType"] ?? .null
        ])
    }

    private func string(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else {
            return nil
        }
        return text
    }
}
