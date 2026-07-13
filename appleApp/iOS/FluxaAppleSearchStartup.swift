import FluxaShared
import Foundation

@MainActor
final class FluxaAppleSearchStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator
    private let encoder = JSONEncoder()

    init(runtime: FluxaAppleHeadlessRuntime) {
        let configurationStore = FluxaAppleAddonConfigurationStore()
        let handler = FluxaAppleHomeEffectHandler(
            configurationStore: configurationStore,
            catalogBootstrap: FluxaAppleCatalogBootstrap()
        )
        coordinator = FluxaAppleHeadlessCoordinator(
            runtime: runtime,
            executor: FluxaApplePlatformEffectExecutor(handler: handler)
        )
    }

    func search(query: String) async {
        let normalizedQuery = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !normalizedQuery.isEmpty else {
            FluxaApple.shared.updateSearchJson(searchJson: "{\"query\":\"\",\"results\":[],\"isLoading\":false}")
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
            FluxaApple.shared.updateSearchJson(searchJson: "{\"query\":\"\(normalizedQuery)\",\"results\":[],\"isLoading\":false}")
        }
    }

    private func updateSharedSearch(result: FluxaAppleHeadlessResult, query: String) {
        let results: [FluxaAppleJsonValue]
        if case .object(let search)? = result.state["search"],
           case .array(let values)? = search["results"] {
            results = values.compactMap(sharedItem)
        } else {
            results = []
        }
        let snapshot = FluxaAppleJsonValue.object([
            "query": .string(query),
            "results": .array(results),
            "isLoading": .boolean(false)
        ])
        guard let data = try? encoder.encode(snapshot) else {
            return
        }
        FluxaApple.shared.updateSearchJson(searchJson: String(decoding: data, as: UTF8.self))
    }

    private func sharedItem(_ value: FluxaAppleJsonValue) -> FluxaAppleJsonValue? {
        guard case .object(let item) = value,
              let id = text(item["id"]),
              let type = text(item["type"]),
              let title = text(item["name"]) else {
            return nil
        }
        return .object([
            "id": .string(id),
            "type": .string(type),
            "title": .string(title),
            "subtitle": item["releaseInfo"] ?? .string(""),
            "artworkUrl": item["poster"] ?? .null,
            "logoUrl": item["logo"] ?? .null,
            "addonTransportUrl": item["addonTransportUrl"] ?? .null,
            "catalogType": item["catalogType"] ?? .null
        ])
    }

    private func text(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else {
            return nil
        }
        return text
    }
}

final class FluxaAppleSearchNotificationObserver {
    private let startup: FluxaAppleSearchStartup
    private let token: NSObjectProtocol

    init(startup: FluxaAppleSearchStartup) {
        self.startup = startup
        token = NotificationCenter.default.addObserver(
            forName: Notification.Name("FluxaAppleSearchRequested"),
            object: nil,
            queue: .main
        ) { [weak startup] notification in
            guard let query = notification.object as? String else {
                return
            }
            Task { @MainActor in
                await startup?.search(query: query)
            }
        }
    }

    deinit {
        NotificationCenter.default.removeObserver(token)
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
