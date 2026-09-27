import Foundation

struct AppleCatalogItemSnapshot: Hashable, Identifiable {
    let id: String
    let type: String
    let title: String
    let subtitle: String
    let artworkUrl: String?
    let logoUrl: String?
    let addonTransportUrl: String?
    let catalogType: String?
    let progress: Double?
    let topTenRank: Int?
}

struct AppleCatalogRowSnapshot: Hashable, Identifiable {
    let id: String
    let title: String
    let items: [AppleCatalogItemSnapshot]
    let canLoadMore: Bool
}

struct AppleDetailStreamSnapshot: Hashable {
    let addonName: String
    let title: String
    let playableUrl: String
    let requestHeadersJson: String
    let subtitleUrls: [String]
}
