import FluxaCore
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

    func load(request: FluxaShared.AppleDetailRequestSnapshot) async {
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
            await updateSharedDetail(result: result, request: request)
        } catch {
            updateEmptyDetail(request: request)
        }
    }

    func toggleWatchlist(request: FluxaShared.AppleDetailRequestSnapshot) async {
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
            await updateSharedDetail(result: result, request: request)
        } catch {
            return
        }
    }

    private func updateSharedDetail(
        result: FluxaAppleHeadlessResult,
        request: FluxaShared.AppleDetailRequestSnapshot
    ) async {
        guard case .object(let detail)? = result.state["detail"],
              case .object(let meta)? = detail["meta"] else {
            updateEmptyDetail(request: request)
            return
        }
        let id = text(meta["id"]) ?? request.id
        let type = text(meta["type"]) ?? request.type
        let addons = addonUrls(for: request)
        let streams = await loadDirectStreams(addons: addons, contentType: type, id: id)
        let subtitleUrls = await loadSubtitleUrls(addons: addons, contentType: type, id: id)
        FluxaApple.shared.updateDetail(snapshot: FluxaShared.AppleDetailSnapshot(id: id, type: type, title: text(meta["name"]) ?? request.id, description: text(meta["description"]) ?? "", posterUrl: text(meta["poster"]), backgroundUrl: text(meta["background"]), logoUrl: text(meta["logo"]), releaseLabel: text(meta["releaseInfo"]) ?? "", ratingLabel: text(meta["imdbRating"]) ?? "", isInWatchlist: bool(detail["isInWatchlist"]), isLoading: false, errorKey: nil, streams: streams.map { toSharedStream($0, subtitleUrls: subtitleUrls) }, hasStreamProviders: !streams.isEmpty))
    }

    private func addonUrls(for request: FluxaShared.AppleDetailRequestSnapshot) -> [String] {
        ([request.addonTransportUrl].compactMap { $0 } + configurationStore.enabledAddonUrls())
            .reduce(into: [String]()) { result, addon in
                if !result.contains(addon) {
                    result.append(addon)
                }
            }
    }

    private func loadDirectStreams(
        addons: [String],
        contentType: String,
        id: String
    ) async -> [FluxaCore.AppleDetailStreamSnapshot] {
        var results = [FluxaCore.AppleDetailStreamSnapshot]()
        for addon in addons {
            if let streams = try? await addonResourceLoader.loadDirectStreams(
                transportUrl: addon,
                contentType: contentType,
                id: id
            ) {
                results.append(contentsOf: streams)
            }
        }
        return results
    }

    private func loadSubtitleUrls(
        addons: [String],
        contentType: String,
        id: String
    ) async -> [String] {
        var results = [String]()
        for addon in addons {
            if let subtitles = try? await addonResourceLoader.loadSubtitleUrls(
                transportUrl: addon,
                contentType: contentType,
                id: id
            ) {
                results.append(contentsOf: subtitles)
            }
        }
        var seen = Set<String>()
        return results.filter { seen.insert($0).inserted }
    }

    private func toSharedStream(
        _ stream: FluxaCore.AppleDetailStreamSnapshot,
        subtitleUrls: [String]
    ) -> FluxaShared.AppleDetailStreamSnapshot {
        FluxaShared.AppleDetailStreamSnapshot(
            addonName: stream.addonName,
            title: stream.title,
            playableUrl: stream.playableUrl,
            requestHeadersJson: stream.requestHeadersJson,
            subtitleUrls: subtitleUrls
        )
    }

    private func updateEmptyDetail(request: FluxaShared.AppleDetailRequestSnapshot) {
        FluxaApple.shared.updateDetail(snapshot: FluxaShared.AppleDetailSnapshot(id: request.id, type: request.type, title: request.title ?? request.id, description: "", posterUrl: nil, backgroundUrl: nil, logoUrl: nil, releaseLabel: "", ratingLabel: "", isInWatchlist: false, isLoading: false, errorKey: "auto.no_results_found", streams: [], hasStreamProviders: false))
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
