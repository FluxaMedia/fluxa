import Foundation

final class FluxaAppleAddonStreamEffectService {
    private let configurationStore: FluxaAppleAddonConfigurationStore
    private let addonResourceLoader: FluxaAppleAddonResourceLoader

    init(
        configurationStore: FluxaAppleAddonConfigurationStore,
        addonResourceLoader: FluxaAppleAddonResourceLoader = FluxaAppleAddonResourceLoader()
    ) {
        self.configurationStore = configurationStore
        self.addonResourceLoader = addonResourceLoader
    }

    func execute(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        switch effect.type {
        case FluxaHeadlessEffectType.fetchMetaDetail:
            return try await fetchMetaDetail(effect: effect)
        case FluxaHeadlessEffectType.fetchMetaDetailLookup:
            return try await loadMetaDetail(effect: effect)
        case FluxaHeadlessEffectType.fetchDetailStreams:
            return try await fetchDetailStreams(effect: effect)
        case FluxaHeadlessEffectType.prefetchNextEpisodeStreams:
            return try await prefetchNextEpisodeStreams(effect: effect)
        default:
            throw NSError(domain: "FluxaAppleUnsupportedStreamEffect", code: 1)
        }
    }

    private func fetchMetaDetail(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        .object(["meta": try await loadMetaDetail(effect: effect)])
    }

    private func loadMetaDetail(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              let contentType = string(payload["contentType"]),
              let id = string(payload["id"]) else {
            throw URLError(.cannotParseResponse)
        }
        let addons = orderedAddons(profile: payload["profile"])
        let descriptors = try await FluxaAppleAddonCatalogResolver().loadAddonDescriptors(localAddonUrls: addons)
        let policy = FluxaCoreStremio.resourceFetchExecutionPolicy(
            manifests: descriptors,
            kind: "metaDetail",
            transportUrl: string(payload["sourceAddonTransportUrl"]),
            contentType: contentType,
            id: id
        )
        let candidates = await withTaskGroup(of: (Int, FluxaAppleJsonValue?).self) { group in
            for (index, request) in (policy?.requests ?? []).enumerated() {
                group.addTask {
                    (index, try? await addonResourceLoader.loadMeta(url: request.url))
                }
            }
            var results = [(Int, FluxaAppleJsonValue?)]()
            for await result in group { results.append(result) }
            return results
        }
        if let winner = candidates
            .sorted(by: { $0.0 < $1.0 })
            .compactMap { $0.1 }
            .first {
            return await FluxaAppleTmdbService().enrichMeta(
                winner,
                contentType: contentType,
                id: id,
                language: string(payload["language"]) ?? "en"
            )
        }
        throw URLError(.fileDoesNotExist)
    }

    private func fetchDetailStreams(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload else {
            throw URLError(.cannotParseResponse)
        }
        let contentType = string(payload["contentType"]) ?? ""
        let requestIds = strings(payload["requestIds"])
        let addons = orderedAddons(profile: payload["profile"])
        var failedAddons = [String]()
        var resolvedRequestId: String?
        var outputStreams = [FluxaAppleJsonValue]()

        for requestId in requestIds {
            let result = await loadStreams(contentType: contentType, id: requestId, addons: addons)
            let requestStreams = result.streams
            failedAddons.append(contentsOf: result.failedAddons)
            if !requestStreams.isEmpty {
                resolvedRequestId = requestId
                outputStreams = requestStreams
                break
            }
        }

        let availableAddons = outputStreams.reduce(into: [String]()) { names, value in
            guard case .object(let fields) = value,
                  let name = string(fields["addonName"]),
                  !name.isEmpty,
                  !names.contains(name) else { return }
            names.append(name)
        }

        return .object([
            "streams": .array(outputStreams),
            "availableAddons": .array(availableAddons.map(FluxaAppleJsonValue.string)),
            "resolvedRequestId": resolvedRequestId.map(FluxaAppleJsonValue.string) ?? .null,
            "hasStreamProviders": .boolean(!addons.isEmpty),
            "failedAddons": .array(Array(Set(failedAddons)).sorted().map(FluxaAppleJsonValue.string))
        ])
    }

    private func prefetchNextEpisodeStreams(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        guard case .object(let payload) = effect.payload,
              let contentType = string(payload["contentType"]),
              let nextVideoId = string(payload["nextVideoId"]),
              !nextVideoId.isEmpty else {
            throw URLError(.cannotParseResponse)
        }
        let addons = orderedAddons(profile: payload["profile"])
        let result = await loadStreams(contentType: contentType, id: nextVideoId, addons: addons)
        return .object(["streams": .array(result.streams)])
    }

    private func loadStreams(
        contentType: String,
        id: String,
        addons: [String]
    ) async -> (streams: [FluxaAppleJsonValue], failedAddons: [String]) {
        var streams = [FluxaAppleJsonValue]()
        var failedAddons = [String]()
        for addon in addons {
            do {
                let addonStreams = try await addonResourceLoader.loadDirectStreams(
                    transportUrl: addon,
                    contentType: contentType,
                    id: id
                )
                let subtitleUrls = (try? await addonResourceLoader.loadSubtitleUrls(
                    transportUrl: addon,
                    contentType: contentType,
                    id: id
                )) ?? []
                streams.append(contentsOf: addonStreams.map { streamValue($0, subtitleUrls: subtitleUrls) })
            } catch {
                failedAddons.append(addonDisplayName(addon))
            }
        }
        return (streams, failedAddons)
    }

    private func orderedAddons(profile: FluxaAppleJsonValue?) -> [String] {
        let profileAddons: [String] = {
            guard case .object(let profile)? = profile else { return [] }
            return strings(profile["localAddons"])
        }()
        // Keep profile-specific preference first, then add enabled global addons once.
        return (profileAddons + configurationStore.enabledAddonUrls()).reduce(into: [String]()) {
            if !$0.contains($1) { $0.append($1) }
        }
    }

    private func streamValue(
        _ stream: AppleDetailStreamSnapshot,
        subtitleUrls: [String]
    ) -> FluxaAppleJsonValue {
        .object([
            "url": .string(stream.playableUrl),
            "playableUrl": .string(stream.playableUrl),
            "name": .string(stream.title),
            "title": .string(stream.title),
            "addonName": .string(stream.addonName),
            "requestHeadersJson": .string(stream.requestHeadersJson),
            "headers": jsonObject(stream.requestHeadersJson),
            "subtitleUrls": .array(subtitleUrls.map(FluxaAppleJsonValue.string))
        ])
    }

    private func strings(_ value: FluxaAppleJsonValue?) -> [String] {
        guard case .array(let values)? = value else { return [] }
        return values.compactMap { string($0) }.filter { !$0.isEmpty }
    }

    private func string(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else { return nil }
        return text
    }

    private func addonDisplayName(_ transportUrl: String) -> String {
        URL(string: transportUrl)?.host ?? transportUrl
    }

    private func jsonObject(_ json: String) -> FluxaAppleJsonValue {
        guard let data = json.data(using: .utf8),
              let object = try? JSONSerialization.jsonObject(with: data) as? [String: String] else {
            return .object([:])
        }
        return .object(object.mapValues(FluxaAppleJsonValue.string))
    }
}
