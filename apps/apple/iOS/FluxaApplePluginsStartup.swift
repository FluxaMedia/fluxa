import FluxaShared
import Foundation

@MainActor
final class FluxaApplePluginsStartup {
    private let manager: FluxaApplePluginRepositoryManager

    init(manager: FluxaApplePluginRepositoryManager) {
        self.manager = manager
    }

    func start() async {
        let state = await manager.restore()
        push(state: state)
    }

    func handle(_ action: ApplePluginsActionSnapshot) async {
        switch action.type {
        case "refresh":
            push(state: manager.currentState())
        case "addRepository":
            guard let manifestUrl = action.manifestUrl else { return }
            push(state: await manager.addRepository(manifestUrl: manifestUrl))
        case "removeRepository":
            guard let manifestUrl = action.manifestUrl else { return }
            push(state: await manager.removeRepository(manifestUrl: manifestUrl))
        case "refreshRepository":
            guard let manifestUrl = action.manifestUrl else { return }
            push(state: await manager.refreshRepository(manifestUrl: manifestUrl))
        case "toggleScraper":
            guard let scraperId = action.scraperId, let enabled = action.enabled?.boolValue else { return }
            push(state: await manager.toggleScraper(scraperId: scraperId, enabled: enabled))
        case "requestScraperSettings":
            await handleRequestScraperSettings(scraperId: action.scraperId)
        case "saveScraperSettings":
            guard let scraperId = action.scraperId else { return }
            let settingsJson = action.settingsJson ?? "{}"
            push(state: await manager.updateScraperSettings(scraperId: scraperId, settingsJson: settingsJson))
        default:
            break
        }
    }

    private func handleRequestScraperSettings(scraperId: String?) async {
        guard let scraperId,
              let scraper = manager.currentState().scrapers.first(where: { $0.id == scraperId }) else {
            return
        }
        let fields = await manager.getSettingsLayout(scraper: scraper)
        push(state: manager.currentState(), scraperSettingsSheet: (scraper, fields))
    }

    private func push(
        state: FluxaApplePluginsState,
        scraperSettingsSheet: (FluxaApplePluginScraperEntry, [FluxaApplePluginSettingsField])? = nil
    ) {
        let sheetSnapshot = scraperSettingsSheet.map { scraper, fields in
            ApplePluginScraperSettingsSnapshot(
                scraper: scraperSnapshot(scraper),
                loading: false,
                fields: fields.map(fieldSnapshot)
            )
        }
        let snapshot = ApplePluginsSnapshot(
            repositories: state.repositories.map(repositorySnapshot),
            scrapers: state.scrapers.map(scraperSnapshot),
            addingRepositoryUrl: state.addingRepositoryUrl,
            repositoryError: state.error,
            scraperSettingsSheet: sheetSnapshot
        )
        FluxaApple.shared.updatePlugins(snapshot: snapshot)
    }

    private func repositorySnapshot(_ entry: FluxaApplePluginRepositoryEntry) -> ApplePluginRepositorySnapshot {
        ApplePluginRepositorySnapshot(
            manifestUrl: entry.manifestUrl,
            name: entry.name,
            description: entry.description,
            scraperCount: Int32(entry.scraperCount)
        )
    }

    private func scraperSnapshot(_ entry: FluxaApplePluginScraperEntry) -> ApplePluginScraperSnapshot {
        ApplePluginScraperSnapshot(
            id: entry.id,
            name: entry.name,
            repositoryUrl: entry.repositoryUrl,
            enabled: entry.enabled,
            supportedTypes: entry.supportedTypes,
            hasSettings: entry.hasSettings,
            settingsJson: entry.settingsJson
        )
    }

    private func fieldSnapshot(_ field: FluxaApplePluginSettingsField) -> ApplePluginSettingsFieldSnapshot {
        ApplePluginSettingsFieldSnapshot(
            key: field.key,
            type: field.type,
            label: field.label,
            description: field.description,
            placeholder: field.placeholder,
            isPassword: field.isPassword,
            defaultValue: field.defaultValue,
            defaultBoolean: field.defaultBoolean,
            options: field.options.map { ApplePluginSettingsOptionSnapshot(label: $0.label, value: $0.value) }
        )
    }
}
