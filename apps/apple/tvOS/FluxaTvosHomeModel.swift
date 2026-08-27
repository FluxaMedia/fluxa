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
        guard let addon = item.addonTransportUrl else { return [] }
        guard let streams = try? await resourceLoader.loadDirectStreams(
            transportUrl: addon,
            contentType: item.type,
            id: item.id
        ) else { return [] }
        let subtitles = (try? await resourceLoader.loadSubtitleUrls(
            transportUrl: addon,
            contentType: item.type,
            id: item.id
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
}
