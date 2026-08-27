import Foundation
import FluxaCore
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
        let type: String
        let title: String
        let subtitle: String
        let artworkUrl: URL?
        let addonTransportUrl: String?
    }

    struct Playback: Sendable {
        let url: URL
        let title: String
        let streamTitle: String
        let headers: [String: String]
        let subtitleUrls: [URL]
    }

    struct Detail: Sendable {
        let title: String
        let description: String
        let episodes: [Episode]
    }

    struct Episode: Identifiable, Sendable {
        let id: String
        let title: String
        let subtitle: String
    }

    @Published private(set) var rows: [Row] = []
    @Published private(set) var isLoading = false

    private let coordinator: FluxaAppleHeadlessCoordinator
    private let resourceLoader = FluxaAppleAddonResourceLoader()

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

    func playbackOptions(for item: Item) async -> [Playback] {
        playbackOptions(for: item, contentId: item.id)
    }

    func playbackOptions(for item: Item, contentId: String) async -> [Playback] {
        guard let addon = item.addonTransportUrl else { return [] }
        guard let streams = try? await resourceLoader.loadDirectStreams(
            transportUrl: addon,
            contentType: item.type,
            id: contentId
        ) else { return [] }
        let subtitles = (try? await resourceLoader.loadSubtitleUrls(
            transportUrl: addon,
            contentType: item.type,
            id: contentId
        )) ?? []
        let subtitleUrls = subtitles.compactMap(URL.init(string:))
        return streams.compactMap { stream in
            guard let url = URL(string: stream.playableUrl) else { return nil }
            return Playback(
                url: url,
                title: item.title,
                streamTitle: stream.title,
                headers: decodeHeaders(stream.requestHeadersJson),
                subtitleUrls: subtitleUrls
            )
        }
    }

    func detail(for item: Item) async -> Detail? {
        guard let addon = item.addonTransportUrl,
              let meta = try? await resourceLoader.loadMeta(
                  transportUrl: addon,
                  contentType: item.type,
                  id: item.id
              ),
              case .object(let object) = meta else {
            return nil
        }
        let episodes: [Episode]
        if case .array(let videos)? = object["videos"] {
            episodes = videos.compactMap { value in
                guard case .object(let video) = value,
                      let id = text(video["id"]),
                      !id.isEmpty else { return nil }
                let name = text(video["name"]) ?? id
                let season = number(video["season"])
                let episode = number(video["number"])
                let subtitle = if let season, let episode {
                    "S\(season) E\(episode)"
                } else {
                    ""
                }
                return Episode(id: id, title: name, subtitle: subtitle)
            }
        } else {
            episodes = []
        }
        return Detail(
            title: text(object["name"]) ?? item.title,
            description: text(object["description"]) ?? "",
            episodes: episodes
        )
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
            type: text(meta["type"]) ?? "movie",
            title: title,
            subtitle: text(meta["releaseInfo"]) ?? "",
            artworkUrl: text(meta["poster"]).flatMap { URL(string: $0) },
            addonTransportUrl: text(meta["addonTransportUrl"])
        )
    }

    private func decodeHeaders(_ json: String) -> [String: String] {
        guard let data = json.data(using: .utf8),
              let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return [:]
        }
        return object.compactMapValues { $0 as? String }
    }

    private func text(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else {
            return nil
        }
        return text
    }

    private func number(_ value: FluxaAppleJsonValue?) -> Int? {
        guard case .number(let value)? = value else { return nil }
        return Int(value)
    }
}
