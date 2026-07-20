import Foundation

struct FluxaApplePluginRepositoryEntry {
    let manifestUrl: String
    let name: String
    let description: String?
    let scraperCount: Int
}

struct FluxaApplePluginSettingsOption {
    let label: String
    let value: String
}

struct FluxaApplePluginSettingsField {
    let key: String
    let type: String
    let label: String
    let description: String?
    let placeholder: String?
    let isPassword: Bool
    let defaultValue: String?
    let defaultBoolean: Bool
    let options: [FluxaApplePluginSettingsOption]
}

struct FluxaApplePluginScraperEntry {
    let id: String
    let name: String
    let repositoryUrl: String
    let filename: String
    let enabled: Bool
    let supportedTypes: [String]
    let hasSettings: Bool
    let settingsJson: String
}

struct FluxaApplePluginsState {
    let repositories: [FluxaApplePluginRepositoryEntry]
    let scrapers: [FluxaApplePluginScraperEntry]
    let addingRepositoryUrl: String?
    let error: String?

    static let empty = FluxaApplePluginsState(repositories: [], scrapers: [], addingRepositoryUrl: nil, error: nil)
}

private struct FluxaApplePluginPersistedOverride: Codable {
    var enabled: Bool = true
    var settingsJson: String = "{}"
}

private struct FluxaApplePluginPersistedState: Codable {
    var repositoryUrls: [String] = []
    var scraperOverrides: [String: FluxaApplePluginPersistedOverride] = [:]
}

final class FluxaApplePluginRepositoryManager {
    private let runtime: FluxaAppleHeadlessRuntime
    private let coordinator: FluxaAppleHeadlessCoordinator
    private let httpClient = FluxaApplePluginHttpClient()
    private let session: URLSession
    private let defaults: UserDefaults
    private let codeCacheLock = NSLock()
    private var codeCache: [String: String] = [:]

    init(session: URLSession = .shared, defaults: UserDefaults = .standard) {
        let runtime = requireFluxaAppleHeadlessRuntime()
        self.runtime = runtime
        self.coordinator = FluxaAppleHeadlessCoordinator(
            runtime: runtime,
            executor: FluxaApplePlatformEffectExecutor(handler: FluxaApplePluginsEffectHandler(session: session))
        )
        self.session = session
        self.defaults = defaults
    }

    func restore() async -> FluxaApplePluginsState {
        let persisted = loadPersistedState()
        for manifestUrl in persisted.repositoryUrls {
            _ = try? await dispatch(["type": "pluginRepositoryAddRequested", "manifestUrl": manifestUrl])
        }
        for (scraperId, override) in persisted.scraperOverrides {
            if !override.enabled {
                _ = try? await dispatch(["type": "pluginScraperToggled", "scraperId": scraperId, "enabled": false])
            }
            if let settings = jsonObject(from: override.settingsJson), !settings.isEmpty {
                _ = try? await dispatch(["type": "pluginScraperSettingsUpdated", "scraperId": scraperId, "settings": settings])
            }
        }
        return currentState()
    }

    func addRepository(manifestUrl: String) async -> FluxaApplePluginsState {
        let state = (try? await dispatch(["type": "pluginRepositoryAddRequested", "manifestUrl": manifestUrl])).map(pluginsState) ?? currentState()
        persist(state)
        return state
    }

    func removeRepository(manifestUrl: String) async -> FluxaApplePluginsState {
        let state = (try? await dispatch(["type": "pluginRepositoryRemoveRequested", "manifestUrl": manifestUrl])).map(pluginsState) ?? currentState()
        persist(state)
        return state
    }

    func refreshRepository(manifestUrl: String) async -> FluxaApplePluginsState {
        await addRepository(manifestUrl: manifestUrl)
    }

    func toggleScraper(scraperId: String, enabled: Bool) async -> FluxaApplePluginsState {
        let state = (try? await dispatch(["type": "pluginScraperToggled", "scraperId": scraperId, "enabled": enabled])).map(pluginsState) ?? currentState()
        persist(state)
        return state
    }

    func updateScraperSettings(scraperId: String, settingsJson: String) async -> FluxaApplePluginsState {
        let settings = jsonObject(from: settingsJson) ?? [:]
        let state = (try? await dispatch(["type": "pluginScraperSettingsUpdated", "scraperId": scraperId, "settings": settings])).map(pluginsState) ?? currentState()
        persist(state)
        return state
    }

    func getSettingsLayout(scraper: FluxaApplePluginScraperEntry) async -> [FluxaApplePluginSettingsField] {
        guard let code = await fetchScraperCode(scraper) else { return [] }
        let layoutJson = getPluginScraperSettingsLayout(code: code, scraperId: scraper.id)
        return parseSettingsLayout(layoutJson)
    }

    func executeScraper(
        scraper: FluxaApplePluginScraperEntry,
        tmdbId: String,
        mediaType: String,
        season: Int32?,
        episode: Int32?
    ) async -> String {
        guard let code = await fetchScraperCode(scraper) else { return "[]" }
        let rawJson = executePluginScraper(
            client: httpClient,
            code: code,
            scraperId: scraper.id,
            scraperSettingsJson: scraper.settingsJson,
            tmdbId: tmdbId,
            mediaType: mediaType,
            season: season,
            episode: episode
        )
        return coreInvoke(method: "pluginStreamResultsToStreams", argsJson: rawJson)
    }

    func currentState() -> FluxaApplePluginsState {
        pluginsState(fromStateJson: runtime.snapshot())
    }

    private func dispatch(_ fields: [String: Any]) async throws -> FluxaAppleHeadlessResult {
        let data = try JSONSerialization.data(withJSONObject: fields)
        let json = String(decoding: data, as: UTF8.self)
        return try await coordinator.dispatch(actionJson: json)
    }

    private func pluginsState(_ result: FluxaAppleHeadlessResult) -> FluxaApplePluginsState {
        pluginsState(from: result.state)
    }

    private func pluginsState(fromStateJson json: String) -> FluxaApplePluginsState {
        guard let result = try? JSONDecoder().decode(FluxaAppleHeadlessResult.self, from: Data(json.utf8)) else {
            return .empty
        }
        return pluginsState(from: result.state)
    }

    private func pluginsState(from state: [String: FluxaAppleJsonValue]) -> FluxaApplePluginsState {
        guard case .object(let plugins)? = state["plugins"] else { return .empty }
        let repositories = array(plugins["repositories"]).compactMap { repositoryEntry($0) }
        let scrapers = array(plugins["scrapers"]).compactMap { scraperEntry($0) }
        let addingRepositoryUrl = text(plugins["addingRepositoryUrl"])
        let error: String?
        if let errorValue = plugins["error"], case .object(let errorObject) = errorValue {
            error = text(errorObject["code"])
        } else {
            error = nil
        }
        return FluxaApplePluginsState(repositories: repositories, scrapers: scrapers, addingRepositoryUrl: addingRepositoryUrl, error: error)
    }

    private func repositoryEntry(_ value: FluxaAppleJsonValue) -> FluxaApplePluginRepositoryEntry? {
        guard case .object(let object) = value, let manifestUrl = text(object["manifestUrl"]) else { return nil }
        return FluxaApplePluginRepositoryEntry(
            manifestUrl: manifestUrl,
            name: text(object["name"]) ?? "",
            description: text(object["description"]),
            scraperCount: number(object["scraperCount"]).map(Int.init) ?? 0
        )
    }

    private func scraperEntry(_ value: FluxaAppleJsonValue) -> FluxaApplePluginScraperEntry? {
        guard case .object(let object) = value, let id = text(object["id"]) else { return nil }
        let settingsJson: String
        if let settingsValue = object["settings"], case .object = settingsValue {
            settingsJson = jsonString(settingsValue)
        } else {
            settingsJson = "{}"
        }
        return FluxaApplePluginScraperEntry(
            id: id,
            name: text(object["name"]) ?? "",
            repositoryUrl: text(object["repositoryUrl"]) ?? "",
            filename: text(object["filename"]) ?? "",
            enabled: boolean(object["enabled"]) ?? true,
            supportedTypes: array(object["supportedTypes"]).compactMap { text($0) },
            hasSettings: boolean(object["hasSettings"]) ?? false,
            settingsJson: settingsJson
        )
    }

    private func fetchScraperCode(_ scraper: FluxaApplePluginScraperEntry) async -> String? {
        codeCacheLock.lock()
        let cached = codeCache[scraper.id]
        codeCacheLock.unlock()
        if let cached { return cached }

        let base = scraper.repositoryUrl
            .split(separator: "/", omittingEmptySubsequences: false)
            .dropLast()
            .joined(separator: "/")
        guard let url = URL(string: "\(base)/\(scraper.filename)") else { return nil }
        do {
            let (data, response) = try await session.data(from: url)
            guard let httpResponse = response as? HTTPURLResponse, (200..<300).contains(httpResponse.statusCode) else {
                return nil
            }
            let code = String(decoding: data, as: UTF8.self)
            guard !code.isEmpty else { return nil }
            codeCacheLock.lock()
            codeCache[scraper.id] = code
            codeCacheLock.unlock()
            return code
        } catch {
            return nil
        }
    }

    private func parseSettingsLayout(_ layoutJson: String) -> [FluxaApplePluginSettingsField] {
        guard let data = layoutJson.data(using: .utf8),
              let array = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else {
            return []
        }
        return array.compactMap { field in
            guard let type = field["type"] as? String else { return nil }
            let options = (field["options"] as? [[String: Any]] ?? []).map {
                FluxaApplePluginSettingsOption(label: $0["label"] as? String ?? "", value: $0["value"] as? String ?? "")
            }
            let defaultValueRaw = field["defaultValue"]
            let defaultIsBoolean = defaultValueRaw is Bool
            return FluxaApplePluginSettingsField(
                key: field["key"] as? String ?? "",
                type: type,
                label: field["label"] as? String ?? "",
                description: field["description"] as? String,
                placeholder: field["placeholder"] as? String,
                isPassword: field["isPassword"] as? Bool ?? false,
                defaultValue: defaultIsBoolean ? nil : defaultValueRaw as? String,
                defaultBoolean: defaultIsBoolean ? (defaultValueRaw as? Bool ?? false) : false,
                options: options
            )
        }
    }

    private func persist(_ state: FluxaApplePluginsState) {
        var persisted = FluxaApplePluginPersistedState()
        persisted.repositoryUrls = state.repositories.map(\.manifestUrl)
        for scraper in state.scrapers {
            persisted.scraperOverrides[scraper.id] = FluxaApplePluginPersistedOverride(
                enabled: scraper.enabled,
                settingsJson: scraper.settingsJson
            )
        }
        guard let data = try? JSONEncoder().encode(persisted) else { return }
        defaults.set(String(decoding: data, as: UTF8.self), forKey: Self.persistedStateKey)
    }

    private func loadPersistedState() -> FluxaApplePluginPersistedState {
        guard let json = defaults.string(forKey: Self.persistedStateKey),
              let data = json.data(using: .utf8),
              let state = try? JSONDecoder().decode(FluxaApplePluginPersistedState.self, from: data) else {
            return FluxaApplePluginPersistedState()
        }
        return state
    }

    private func jsonObject(from json: String) -> [String: Any]? {
        guard let data = json.data(using: .utf8) else { return nil }
        return try? JSONSerialization.jsonObject(with: data) as? [String: Any]
    }

    private func jsonString(_ value: FluxaAppleJsonValue) -> String {
        guard let data = try? JSONEncoder().encode(value) else { return "{}" }
        return String(decoding: data, as: UTF8.self)
    }

    private func array(_ value: FluxaAppleJsonValue?) -> [FluxaAppleJsonValue] {
        guard case .array(let values)? = value else { return [] }
        return values
    }

    private func text(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else { return nil }
        return text
    }

    private func boolean(_ value: FluxaAppleJsonValue?) -> Bool? {
        guard case .boolean(let flag)? = value else { return nil }
        return flag
    }

    private func number(_ value: FluxaAppleJsonValue?) -> Double? {
        guard case .number(let number)? = value else { return nil }
        return number
    }

    private static let persistedStateKey = "fluxa.apple.pluginRepositoryState"
}
