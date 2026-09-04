import Foundation
import FluxaCore

final class FluxaTvosEffectHandler: FluxaApplePlatformEffectHandler {
    private let configurationStore: FluxaAppleAddonConfigurationStore
    private let catalogService: FluxaAppleCatalogService

    init(
        configurationStore: FluxaAppleAddonConfigurationStore,
        catalogService: FluxaAppleCatalogService = FluxaAppleCatalogService()
    ) {
        self.configurationStore = configurationStore
        self.catalogService = catalogService
    }

    func execute(effect: FluxaAppleHeadlessEffect) async throws -> FluxaAppleJsonValue {
        switch effect.type {
        case FluxaHeadlessEffectType.readHomeBootstrap:
            let rows = try await catalogService.loadHomeRows(
                addonUrls: configurationStore.localAddonUrls()
            )
            let categories = rows.map(category)
            return .object([
                "categories": .array(categories),
                "continueWatching": .array([]),
                "watchlist": .array([]),
                "userAddons": .array(configurationStore.localAddonUrls().map(FluxaAppleJsonValue.string)),
                "metadataFeeds": .array([]),
                "billboard": firstItem(in: categories) ?? .null
            ])
        case FluxaHeadlessEffectType.refreshContinueWatching:
            return .object(["continueWatching": .array([])])
        case FluxaHeadlessEffectType.readLibraryState:
            return .object([
                "watchlist": .array([]),
                "continueWatching": .array([]),
                "liked": .array([]),
                "watched": .object([:])
            ])
        case FluxaHeadlessEffectType.readCalendarMonth:
            return .object(["items": .array([])])
        case FluxaHeadlessEffectType.readPlaybackProgress:
            return .null
        default:
            throw NSError(domain: "FluxaTvosUnsupportedEffect", code: 1)
        }
    }

    private func category(_ row: FluxaCore.AppleCatalogRowSnapshot) -> FluxaAppleJsonValue {
        .object([
            "id": .string(row.id),
            "name": .string(row.title),
            "semanticName": .string(row.title),
            "type": .string(row.items.first?.type ?? ""),
            "catalogId": .string(row.id),
            "items": .array(row.items.map { item in
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
            })
        ])
    }

    private func optionalString(_ value: String?) -> FluxaAppleJsonValue {
        value.map(FluxaAppleJsonValue.string) ?? .null
    }

    private func firstItem(in categories: [FluxaAppleJsonValue]) -> FluxaAppleJsonValue? {
        guard case .object(let category)? = categories.first,
              case .array(let items)? = category["items"] else {
            return nil
        }
        return items.first
    }
}
