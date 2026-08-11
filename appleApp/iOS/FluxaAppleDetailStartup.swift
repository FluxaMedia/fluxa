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
        let streams = await loadDirectStreams(addons: entry.addons, contentType: request.type, id: targetId)
        let subtitleUrls = await loadSubtitleUrls(addons: entry.addons, contentType: request.type, id: targetId)
        pushSnapshot(
            entry: entry,
            streams: streams.map { toSharedStream($0, subtitleUrls: subtitleUrls) },
            isLoadingStreams: false,
            loadingAddonNames: [],
            selectedAddon: nil,
            errorKey: streams.isEmpty ? "auto.no_results_found" : nil
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
        let seasonsCount = int32(meta["seasonsCount"])
        let seasons = availableSeasons(from: videos, seasonsCount: seasonsCount)
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
            selectedSeason: initialSeason,
            selectedEpisodeId: nil
        )
        cache[id] = entry

        if type == "series" && !videos.isEmpty {
            pushSnapshot(entry: entry, streams: [], isLoadingStreams: false, loadingAddonNames: [], selectedAddon: nil)
        } else {
            let streams = await loadDirectStreams(addons: addons, contentType: type, id: id)
            let subtitleUrls = await loadSubtitleUrls(addons: addons, contentType: type, id: id)
            pushSnapshot(
                entry: entry,
                streams: streams.map { toSharedStream($0, subtitleUrls: subtitleUrls) },
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
            selectedAddon: selectedAddon
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

    private func toEpisodeSnapshot(_ video: FluxaAppleVideo) -> FluxaShared.AppleDetailEpisodeSnapshot {
        FluxaShared.AppleDetailEpisodeSnapshot(
            id: video.id,
            season: video.season,
            number: video.number,
            title: video.name ?? video.id,
            description: video.overview,
            thumbnailUrl: video.thumbnail,
            releaseLabel: video.released,
            runtimeLabel: video.episodeRuntime.map { "\($0)m" },
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

    private func availableSeasons(from videos: [FluxaAppleVideo], seasonsCount: Int32?) -> [String] {
        var seasons = Set<Int32>()
        if let seasonsCount, seasonsCount > 0 {
            for season in 1...seasonsCount {
                seasons.insert(season)
            }
        }
        for video in videos where video.season > 0 {
            seasons.insert(video.season)
        }
        var sorted = seasons.sorted().map(String.init)
        if videos.contains(where: { $0.season == 0 }) {
            sorted.append("0")
        }
        return sorted.isEmpty ? ["1"] : sorted
    }

    private func addonDisplayName(_ transportUrl: String) -> String {
        URL(string: transportUrl)?.host ?? transportUrl
    }

    private func isUpcoming(_ released: String?) -> Bool {
        guard let released, !released.isEmpty else { return false }
        if let date = ISO8601DateFormatter().date(from: released) {
            return date > Date()
        }
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        formatter.timeZone = TimeZone(identifier: "UTC")
        if let date = formatter.date(from: String(released.prefix(10))) {
            return date > Date()
        }
        return false
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
            selectedAddon: nil
        ))
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
