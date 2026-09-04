import FluxaShared

enum FluxaAppleSnapshotMapper {
    static func catalogRow(_ value: FluxaAppleJsonValue) -> AppleCatalogRowSnapshot? {
        guard case .object(let row) = value,
              let id = text(row["id"]),
              let title = text(row["name"]),
              case .array(let values)? = row["items"] else {
            return nil
        }
        return AppleCatalogRowSnapshot(
            id: id,
            title: title,
            items: values.compactMap(catalogItem),
            canLoadMore: false
        )
    }

    static func catalogItems(_ value: FluxaAppleJsonValue?) -> [AppleCatalogItemSnapshot] {
        guard case .array(let values)? = value else { return [] }
        return values.compactMap(catalogItem)
    }

    static func catalogItem(_ value: FluxaAppleJsonValue) -> AppleCatalogItemSnapshot? {
        guard case .object(let item) = value,
              let id = text(item["id"]),
              let title = text(item["name"]) ?? text(item["title"]) else {
            return nil
        }
        return AppleCatalogItemSnapshot(
            id: id,
            type: text(item["type"]) ?? "movie",
            title: title,
            subtitle: text(item["releaseInfo"]) ?? "",
            artworkUrl: text(item["poster"]),
            logoUrl: text(item["logo"]),
            addonTransportUrl: text(item["addonTransportUrl"]),
            catalogType: text(item["catalogType"]),
            progress: nil,
            topTenRank: nil
        )
    }

    static func text(_ value: FluxaAppleJsonValue?) -> String? {
        switch value {
        case .string(let value): return value
        case .number(let value): return String(value)
        default: return nil
        }
    }
}
