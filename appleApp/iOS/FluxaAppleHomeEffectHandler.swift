import Foundation

final class FluxaAppleHomeEffectHandler: FluxaApplePlatformEffectHandler {
    private let configurationStore: FluxaAppleAddonConfigurationStore
    private let catalogBootstrap: FluxaAppleCatalogBootstrap

    init(
        configurationStore: FluxaAppleAddonConfigurationStore,
        catalogBootstrap: FluxaAppleCatalogBootstrap
    ) {
        self.configurationStore = configurationStore
        self.catalogBootstrap = catalogBootstrap
    }

    func execute(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        switch effect.type {
        case "readHomeBootstrap":
            let rows = try await catalogBootstrap.loadRows(
                localAddonUrls: configurationStore.localAddonUrls()
            )
            return .object([
                "categories": .array(rows.map(homeCategory)),
                "continueWatching": .array([]),
                "watchlist": .array([]),
                "userAddons": .array(configurationStore.localAddonUrls().map { .string($0) }),
                "metadataFeeds": .array([]),
                "billboard": rows.first?.items.first.map(homeMeta) ?? .null
            ])
        case "refreshContinueWatching":
            return .object(["continueWatching": .array([])])
        default:
            throw NSError(domain: "FluxaAppleUnsupportedEffect", code: 1)
        }
    }

    private func homeCategory(_ row: FluxaAppleCatalogRow) -> FluxaAppleJsonValue {
        .object([
            "id": .string(row.id),
            "name": .string(row.title),
            "semanticName": .string(row.title),
            "type": .string(row.items.first?.type ?? ""),
            "catalogId": .string(row.id),
            "items": .array(row.items.map(homeMeta))
        ])
    }

    private func homeMeta(_ item: FluxaAppleCatalogItem) -> FluxaAppleJsonValue {
        .object([
            "id": .string(item.id),
            "type": .string(item.type),
            "name": .string(item.title),
            "poster": optionalString(item.artworkUrl),
            "logo": optionalString(item.logoUrl),
            "releaseInfo": .string(item.subtitle),
            "addonTransportUrl": optionalString(item.addonTransportUrl),
            "catalogType": optionalString(item.catalogType)
        ])
    }

    private func optionalString(_ value: String?) -> FluxaAppleJsonValue {
        value.map(FluxaAppleJsonValue.string) ?? .null
    }
}
