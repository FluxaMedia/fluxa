import FluxaCore
import FluxaShared
import Foundation

@MainActor
final class FluxaAppleDetailStartup {
    private let coordinator: FluxaAppleHeadlessCoordinator
    private let configurationStore: FluxaAppleAddonConfigurationStore
    private let addonResourceLoader: FluxaAppleAddonResourceLoader
    private let encoder = JSONEncoder()
    private var cache: [String: FluxaAppleDetailCacheEntry] = [:]

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

    func selectSeason(request: FluxaShared.AppleDetailSeasonRequestSnapshot) {
        guard let entry = cache[request.id] else { return }
        entry.selectedSeason = request.season
        entry.selectedEpisodeId = nil
        pushSnapshot(entry: entry, streams: [], isLoadingStreams: false, loadingAddonNames: [], selectedAddon: nil)
    }

    func loadSources(request: FluxaShared.AppleDetailStreamsRequestSnapshot) async {
        guard let entry = cache[request.id] else { return }
        entry.selectedEpisodeId = request.episodeId
        pushSnapshot(
            entry: entry,
            streams: [],
            isLoadingStreams: true,
            loadingAddonNames: entry.addons.map(addonDisplayName),
            selectedAddon: nil
        )
        let targetId = request.episodeId ?? request.id
        let streams = await loadCoreStreams(entry: entry, requestIds: [targetId])
        pushSnapshot(
            entry: entry,
            streams: streams,
            isLoadingStreams: false,
            loadingAddonNames: [],
            selectedAddon: nil,
            errorKey: streams.isEmpty ? "auto.no_results_found" : nil
        )
    }

    func downloadEpisode(request: FluxaShared.AppleDetailStreamsRequestSnapshot) async {
        guard let entry = cache[request.id] else { return }
        let episode = request.episodeId.flatMap { episodeId in entry.videos.first { $0.id == episodeId } }
        await enqueueDownload(entry: entry, episode: episode)
    }

    func downloadSeason(request: FluxaShared.AppleDetailSeasonRequestSnapshot) async {
        guard let entry = cache[request.id] else { return }
        let episodes = entry.videos.filter { $0.season == request.season }
        for episode in episodes {
            await enqueueDownload(entry: entry, episode: episode)
        }
    }

    private func enqueueDownload(entry: FluxaAppleDetailCacheEntry, episode: FluxaAppleVideo?) async {
        let targetId = episode?.id ?? entry.id
        guard let stream = await loadDirectStreams(addons: entry.addons, contentType: entry.type, id: targetId).first else {
            return
        }
        let headers = decodeHeaders(stream.requestHeadersJson)
        FluxaAppleDownloadManager.shared.enqueue(
            metaId: entry.id,
            metaType: entry.type,
            title: entry.title,
            episodeTitle: episode?.name,
            videoId: episode?.id,
            posterUrl: entry.posterUrl,
            backgroundUrl: entry.backgroundUrl,
            streamUrl: stream.playableUrl,
            requestHeaders: headers
        )
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
        let videos = parseVideos(meta["videos"])
        let recommendations = parseCatalogItems(detail["similarItems"])
        let seasonsCount = int32(meta["seasonsCount"])
        let seasons = FluxaCoreStremio.detailAvailableSeasons(
            seasons: videos.map { Int($0.season) },
            seasonsCount: seasonsCount.map(Int.init)
        ).map(String.init)
        let initialSeason = seasons.first.flatMap { Int32($0) } ?? 1

        let entry = FluxaAppleDetailCacheEntry(
            id: id,
            type: type,
            title: text(meta["name"]) ?? request.id,
            description: text(meta["description"]) ?? "",
            posterUrl: text(meta["poster"]),
            backgroundUrl: text(meta["background"]),
            logoUrl: text(meta["logo"]),
            releaseLabel: text(meta["releaseInfo"]) ?? "",
            ratingLabel: text(meta["imdbRating"]) ?? "",
            isInWatchlist: bool(detail["isInWatchlist"]),
            videos: videos,
            availableSeasons: seasons,
            addons: addons,
            recommendationItems: recommendations,
            meta: meta,
            selectedSeason: initialSeason,
            selectedEpisodeId: nil
        )
        cache[id] = entry

        if FluxaCoreStremio.isSeriesContentType(type) && !videos.isEmpty {
            pushSnapshot(entry: entry, streams: [], isLoadingStreams: false, loadingAddonNames: [], selectedAddon: nil)
        } else {
            let streams = await loadCoreStreams(entry: entry, requestIds: [id])
            pushSnapshot(
                entry: entry,
                streams: streams,
                isLoadingStreams: false,
                loadingAddonNames: [],
                selectedAddon: nil
            )
        }
    }

    private func pushSnapshot(
        entry: FluxaAppleDetailCacheEntry,
        streams: [FluxaShared.AppleDetailStreamSnapshot],
        isLoadingStreams: Bool,
        loadingAddonNames: [String],
        selectedAddon: String?,
        errorKey: String? = nil
    ) {
        let episodes = entry.videos
            .filter { $0.season == entry.selectedSeason }
            .map(toEpisodeSnapshot)
        FluxaApple.shared.updateDetail(snapshot: FluxaShared.AppleDetailSnapshot(
            id: entry.id,
            type: entry.type,
            title: entry.title,
            description: entry.description,
            posterUrl: entry.posterUrl,
            backgroundUrl: entry.backgroundUrl,
            logoUrl: entry.logoUrl,
            releaseLabel: entry.releaseLabel,
            ratingLabel: entry.ratingLabel,
            isInWatchlist: entry.isInWatchlist,
            isLoading: false,
            errorKey: errorKey,
            streams: streams,
            hasStreamProviders: !entry.addons.isEmpty,
            availableSeasons: entry.availableSeasons,
            selectedSeason: entry.selectedSeason,
            seasonEpisodes: episodes,
            selectedEpisodeId: entry.selectedEpisodeId,
            isLoadingStreams: isLoadingStreams,
            availableAddons: entry.addons.map(addonDisplayName),
            loadingAddonNames: loadingAddonNames,
            selectedAddon: selectedAddon,
            recommendationItems: entry.recommendationItems
        ))
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

    private func loadCoreStreams(
        entry: FluxaAppleDetailCacheEntry,
        requestIds: [String]
    ) async -> [FluxaShared.AppleDetailStreamSnapshot] {
        do {
            let episodes = entry.videos.map { video in
                FluxaAppleJsonValue.object([
                    "id": .string(video.id),
                    "season": .number(Double(video.season)),
                    // Match fluxa_core::types::Video's serde contract. `title`
                    // and `episode` are not aliases for the Stremio response
                    // names used by the local Swift cache model.
                    "episode": .number(Double(video.number)),
                    "title": video.name.map(FluxaAppleJsonValue.string) ?? .null,
                    "released": video.released.map(FluxaAppleJsonValue.string) ?? .null,
                    "thumbnail": video.thumbnail.map(FluxaAppleJsonValue.string) ?? .null,
                    "overview": video.overview.map(FluxaAppleJsonValue.string) ?? .null
                ])
            }
            let action = FluxaAppleDetailStreamsAction(
                type: "detailStreamsRequested",
                contentType: entry.type,
                requestIds: requestIds,
                detail: entry.meta,
                seasonEpisodes: episodes,
                language: "en",
                profile: .object([
                    "id": .string("apple-default"),
                    "localAddons": .array(entry.addons.map(FluxaAppleJsonValue.string))
                ])
            )
            let actionJson = String(decoding: try encoder.encode(action), as: UTF8.self)
            let result = try await coordinator.dispatch(actionJson: actionJson)
            guard case .object(let detail)? = result.state["detail"],
                  case .array(let streams)? = detail["streams"] else {
                return []
            }
            return streams.compactMap(sharedStream)
        } catch {
            return []
        }
    }

    private func sharedStream(_ value: FluxaAppleJsonValue) -> FluxaShared.AppleDetailStreamSnapshot? {
        guard case .object(let fields) = value,
              let playableUrl = text(fields["playableUrl"]) ?? text(fields["url"]) else {
            return nil
        }
        let headersJson: String = {
            if let existing = text(fields["requestHeadersJson"]) { return existing }
            guard case .object(let headers)? = fields["headers"] else { return "{}" }
            let values = headers.compactMapValues { value -> String? in
                guard case .string(let string) = value else { return nil }
                return string
            }
            guard let data = try? JSONSerialization.data(withJSONObject: values),
                  let json = String(data: data, encoding: .utf8) else { return "{}" }
            return json
        }()
        let subtitleUrls: [String] = {
            guard case .array(let values)? = fields["subtitleUrls"] else { return [] }
            return values.compactMap(text)
        }()
        return FluxaShared.AppleDetailStreamSnapshot(
            addonName: text(fields["addonName"]) ?? "",
            title: text(fields["title"]) ?? text(fields["name"]) ?? "",
            playableUrl: playableUrl,
            requestHeadersJson: headersJson,
            subtitleUrls: subtitleUrls
        )
    }

    private func toEpisodeSnapshot(_ video: FluxaAppleVideo) -> FluxaShared.AppleDetailEpisodeSnapshot {
        FluxaShared.AppleDetailEpisodeSnapshot(
            id: video.id,
            season: video.season,
            number: video.number,
            title: video.name ?? video.id,
            description: video.overview,
            thumbnailUrl: video.thumbnail,
            releaseLabel: video.released,
            runtimeLabel: video.episodeRuntime.flatMap { FluxaCoreStremio.formatRuntimeLabel("\($0)m") },
            isUpcoming: isUpcoming(video.released),
            isWatched: false
        )
    }

    private func parseVideos(_ value: FluxaAppleJsonValue?) -> [FluxaAppleVideo] {
        guard case .array(let items)? = value else { return [] }
        return items.compactMap { item -> FluxaAppleVideo? in
            guard case .object(let fields) = item, let id = text(fields["id"]) else { return nil }
            return FluxaAppleVideo(
                id: id,
                name: text(fields["name"]),
                season: int32(fields["season"]) ?? 0,
                number: int32(fields["number"]) ?? 0,
                released: text(fields["released"]),
                thumbnail: text(fields["thumbnail"]),
                overview: text(fields["overview"]),
                episodeRuntime: int(fields["episodeRuntime"])
            )
        }
    }

    private func addonDisplayName(_ transportUrl: String) -> String {
        URL(string: transportUrl)?.host ?? transportUrl
    }

    private func decodeHeaders(_ json: String) -> [String: String] {
        guard let data = json.data(using: .utf8),
              let decoded = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return [:]
        }
        return decoded.compactMapValues { $0 as? String }
    }

    private func isUpcoming(_ released: String?) -> Bool {
        guard let released, !released.isEmpty else { return false }
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        formatter.timeZone = TimeZone.current
        return FluxaCoreStremio.releaseDateUpcoming(
            released,
            todayIso: formatter.string(from: Date())
        )
    }

    private func updateEmptyDetail(request: FluxaShared.AppleDetailRequestSnapshot) {
        cache[request.id] = nil
        FluxaApple.shared.updateDetail(snapshot: FluxaShared.AppleDetailSnapshot(
            id: request.id,
            type: request.type,
            title: request.title ?? request.id,
            description: "",
            posterUrl: nil,
            backgroundUrl: nil,
            logoUrl: nil,
            releaseLabel: "",
            ratingLabel: "",
            isInWatchlist: false,
            isLoading: false,
            errorKey: "auto.no_results_found",
            streams: [],
            hasStreamProviders: false,
            availableSeasons: [],
            selectedSeason: 1,
            seasonEpisodes: [],
            selectedEpisodeId: nil,
            isLoadingStreams: false,
            availableAddons: [],
            loadingAddonNames: [],
            selectedAddon: nil,
            recommendationItems: []
        ))
    }

    private func parseCatalogItems(_ value: FluxaAppleJsonValue?) -> [FluxaShared.AppleCatalogItemSnapshot] {
        FluxaAppleSnapshotMapper.catalogItems(value)
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

    private func int32(_ value: FluxaAppleJsonValue?) -> Int32? {
        if case .number(let number)? = value { return Int32(number) }
        if case .string(let text)? = value { return Int32(text) }
        return nil
    }

    private func int(_ value: FluxaAppleJsonValue?) -> Int? {
        if case .number(let number)? = value { return Int(number) }
        if case .string(let text)? = value { return Int(text) }
        return nil
    }
}

private final class FluxaAppleDetailCacheEntry {
    let id: String
    let type: String
    let title: String
    let description: String
    let posterUrl: String?
    let backgroundUrl: String?
    let logoUrl: String?
    let releaseLabel: String
    let ratingLabel: String
    let isInWatchlist: Bool
    let videos: [FluxaAppleVideo]
    let availableSeasons: [String]
    let addons: [String]
    let recommendationItems: [FluxaShared.AppleCatalogItemSnapshot]
    let meta: FluxaAppleJsonValue
    var selectedSeason: Int32
    var selectedEpisodeId: String?

    init(
        id: String,
        type: String,
        title: String,
        description: String,
        posterUrl: String?,
        backgroundUrl: String?,
        logoUrl: String?,
        releaseLabel: String,
        ratingLabel: String,
        isInWatchlist: Bool,
        videos: [FluxaAppleVideo],
        availableSeasons: [String],
        addons: [String],
        recommendationItems: [FluxaShared.AppleCatalogItemSnapshot],
        meta: FluxaAppleJsonValue,
        selectedSeason: Int32,
        selectedEpisodeId: String?
    ) {
        self.id = id
        self.type = type
        self.title = title
        self.description = description
        self.posterUrl = posterUrl
        self.backgroundUrl = backgroundUrl
        self.logoUrl = logoUrl
        self.releaseLabel = releaseLabel
        self.ratingLabel = ratingLabel
        self.isInWatchlist = isInWatchlist
        self.videos = videos
        self.availableSeasons = availableSeasons
        self.addons = addons
        self.recommendationItems = recommendationItems
        self.meta = meta
        self.selectedSeason = selectedSeason
        self.selectedEpisodeId = selectedEpisodeId
    }
}

private struct FluxaAppleVideo {
    let id: String
    let name: String?
    let season: Int32
    let number: Int32
    let released: String?
    let thumbnail: String?
    let overview: String?
    let episodeRuntime: Int?
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

private struct FluxaAppleDetailStreamsAction: Encodable {
    let type: String
    let contentType: String
    let requestIds: [String]
    let detail: FluxaAppleJsonValue
    let seasonEpisodes: [FluxaAppleJsonValue]
    let language: String
    let profile: FluxaAppleJsonValue
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
