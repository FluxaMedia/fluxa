import FluxaShared
import Foundation

@MainActor
final class FluxaAppleDetailStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator
    private let configurationStore: FluxaAppleAddonConfigurationStore
    private let addonResourceLoader: FluxaAppleAddonResourceLoader
    private let encoder = JSONEncoder()

    init(
        coordinator: FluxaAppleHeadlessCoordinator,
        configurationStore: FluxaAppleAddonConfigurationStore = FluxaAppleAddonConfigurationStore(),
        addonResourceLoader: FluxaAppleAddonResourceLoader = FluxaAppleAddonResourceLoader()
    ) {
        self.coordinator = coordinator
        self.configurationStore = configurationStore
        self.addonResourceLoader = addonResourceLoader
    }

    func load(request: AppleDetailRequestSnapshot) async {
        do {
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
            updateEmptyDetail(request: request)
        }
    }

    func toggleWatchlist(request: AppleDetailRequestSnapshot) async {
        do {
            let action = FluxaAppleToggleWatchlistAction(
                type: "toggleWatchlistRequested",
                item: FluxaAppleToggleWatchlistItem(
                    id: request.id,
                    type: request.type,
                    name: request.title ?? request.id
                )
            )
            let actionJson = String(decoding: try encoder.encode(action), as: UTF8.self)
            let result = try await coordinator.dispatch(actionJson: actionJson)
            updateSharedDetail(result: result, request: request)
        } catch {
            return
        }
    }

    private func updateSharedDetail(
        result: FluxaAppleHeadlessResult,
        request: AppleDetailRequestSnapshot
    ) {
        guard case .object(let detail)? = result.state["detail"],
              case .object(let meta)? = detail["meta"] else {
            updateEmptyDetail(request: request)
            return
        }
        let id = text(meta["id"]) ?? request.id
        let type = text(meta["type"]) ?? request.type
        let streams = await loadDirectStreams(request: request, contentType: type, id: id)
        FluxaApple.shared.updateDetail(snapshot: AppleDetailSnapshot(id: id, type: type, title: text(meta["name"]) ?? request.id, description: text(meta["description"]) ?? "", posterUrl: text(meta["poster"]), backgroundUrl: text(meta["background"]), logoUrl: text(meta["logo"]), releaseLabel: text(meta["releaseInfo"]) ?? "", ratingLabel: text(meta["imdbRating"]) ?? "", isInWatchlist: bool(detail["isInWatchlist"]), isLoading: false, errorKey: nil, streams: streams, hasStreamProviders: !streams.isEmpty))
    }

    private func loadDirectStreams(
        request: AppleDetailRequestSnapshot,
        contentType: String,
        id: String
    ) async -> [AppleDetailStreamSnapshot] {
        let addons = ([request.addonTransportUrl].compactMap { $0 } + configurationStore.localAddonUrls())
            .reduce(into: [String]()) { result, addon in
                if !result.contains(addon) {
                    result.append(addon)
                }
            }
        var results = [FluxaAppleDirectStream]()
        for addon in addons {
            if let streams = try? await addonResourceLoader.loadDirectStreams(
                transportUrl: addon,
                contentType: contentType,
                id: id
            ) {
                results.append(contentsOf: streams)
            }
        }
        return results.map {
            AppleDetailStreamSnapshot(addonName: $0.addonName, title: $0.title, playableUrl: $0.url, requestHeadersJson: $0.requestHeadersJson)
        }
    }

    private func updateEmptyDetail(request: AppleDetailRequestSnapshot) {
        FluxaApple.shared.updateDetail(snapshot: AppleDetailSnapshot(id: request.id, type: request.type, title: request.title ?? request.id, description: "", posterUrl: nil, backgroundUrl: nil, logoUrl: nil, releaseLabel: "", ratingLabel: "", isInWatchlist: false, isLoading: false, errorKey: "auto.no_results_found"))
    }

    private func text(_ value: FluxaAppleJsonValue?) -> String? {
        switch value {
        case .string(let text): return text
        case .number(let number): return String(number)
        default: return nil
        }
    }

    private func bool(_ value: FluxaAppleJsonValue?) -> Bool {
        if case .boolean(let value)? = value { return value }
        return false
    }
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

private struct FluxaAppleToggleWatchlistAction: Encodable {
    let type: String
    let item: FluxaAppleToggleWatchlistItem
}

private struct FluxaAppleToggleWatchlistItem: Encodable {
    let id: String
    let type: String
    let name: String
}
