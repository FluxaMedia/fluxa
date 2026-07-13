import FluxaShared
import Foundation

@MainActor
final class FluxaAppleDetailStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator
    private let decoder = JSONDecoder()
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

    func load(requestJson: String) async {
        do {
            let request = try decoder.decode(FluxaAppleDetailRequest.self, from: Data(requestJson.utf8))
            let action = FluxaAppleDetailAction(
                type: "detailLoadRequested",
                contentType: request.type,
                id: request.id,
                language: "en",
                sourceAddonTransportUrl: request.addonTransportUrl,
                sourceAddonCatalogType: request.catalogType,
                profile: FluxaAppleDetailProfile(id: "apple-default")
            )
            let actionJson = String(decoding: try encoder.encode(action), as: UTF8.self)
            let result = try await coordinator.dispatch(actionJson: actionJson)
            updateSharedDetail(result: result, request: request)
        } catch {
            FluxaApple.shared.updateDetailJson(detailJson: "{\"isLoading\":false,\"errorKey\":\"auto.no_results_found\"}")
        }
    }

    private func updateSharedDetail(
        result: FluxaAppleHeadlessResult,
        request: FluxaAppleDetailRequest
    ) {
        guard case .object(let detail)? = result.state["detail"],
              case .object(let meta)? = detail["meta"] else {
            FluxaApple.shared.updateDetailJson(detailJson: "{\"isLoading\":false,\"errorKey\":\"auto.no_results_found\"}")
            return
        }
        let snapshot = FluxaAppleJsonValue.object([
            "id": meta["id"] ?? .string(request.id),
            "type": meta["type"] ?? .string(request.type),
            "title": text(meta["name"]) ?? .string(request.id),
            "description": text(meta["description"]) ?? .string(""),
            "posterUrl": meta["poster"] ?? .null,
            "backgroundUrl": meta["background"] ?? .null,
            "logoUrl": meta["logo"] ?? .null,
            "releaseLabel": text(meta["releaseInfo"]) ?? .string(""),
            "ratingLabel": text(meta["imdbRating"]) ?? .string(""),
            "isInWatchlist": detail["isInWatchlist"] ?? .boolean(false),
            "relatedItems": .array([]),
            "isLoading": .boolean(false)
        ])
        guard let data = try? encoder.encode(snapshot) else {
            return
        }
        FluxaApple.shared.updateDetailJson(detailJson: String(decoding: data, as: UTF8.self))
    }

    private func text(_ value: FluxaAppleJsonValue?) -> FluxaAppleJsonValue? {
        switch value {
        case .string:
            value
        case .number(let number):
            .string(String(number))
        default:
            nil
        }
    }
}

final class FluxaAppleDetailNotificationObserver {
    private let startup: FluxaAppleDetailStartup
    private let token: NSObjectProtocol

    init(startup: FluxaAppleDetailStartup) {
        self.startup = startup
        token = NotificationCenter.default.addObserver(
            forName: Notification.Name("FluxaAppleDetailRequested"),
            object: nil,
            queue: .main
        ) { [weak startup] notification in
            guard let requestJson = notification.object as? String else {
                return
            }
            Task { @MainActor in
                await startup?.load(requestJson: requestJson)
            }
        }
    }

    deinit {
        NotificationCenter.default.removeObserver(token)
    }
}

private struct FluxaAppleDetailRequest: Decodable {
    let id: String
    let type: String
    let addonTransportUrl: String?
    let catalogType: String?
}

private struct FluxaAppleDetailAction: Encodable {
    let type: String
    let contentType: String
    let id: String
    let language: String
    let sourceAddonTransportUrl: String?
    let sourceAddonCatalogType: String?
    let profile: FluxaAppleDetailProfile
}

private struct FluxaAppleDetailProfile: Encodable {
    let id: String
}
