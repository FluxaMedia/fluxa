import FluxaShared
import Foundation

struct FluxaAppleCatalogRequest: Sendable {
    let id: String
    let title: String
    let url: URL
    let contentType: String
    let addonTransportUrl: String?
    let catalogType: String?
}

final class FluxaAppleCatalogLoader {
    private let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }

    func loadRows(requests: [FluxaAppleCatalogRequest]) async throws -> [AppleCatalogRowSnapshot] {
        var rows = [AppleCatalogRowSnapshot]()
        var lastError: Error?
        for request in requests {
            do {
                let (data, response) = try await session.data(from: request.url)
                guard let httpResponse = response as? HTTPURLResponse,
                      (200..<300).contains(httpResponse.statusCode) else {
                    throw URLError(.badServerResponse)
                }
                guard let catalogItems = FluxaApple.shared.parseCatalogItems(
                    body: String(decoding: data, as: UTF8.self),
                    fallbackType: request.contentType,
                    addonTransportUrl: request.addonTransportUrl,
                    catalogType: request.catalogType
                ) else {
                    throw URLError(.cannotParseResponse)
                }
                rows.append(
                    AppleCatalogRowSnapshot(
                        id: request.id,
                        title: request.title,
                        items: catalogItems.map {
                            AppleCatalogItemSnapshot(
                                id: $0.id,
                                type: $0.type,
                                title: $0.title,
                                subtitle: $0.subtitle,
                                artworkUrl: $0.artworkUrl,
                                logoUrl: $0.logoUrl,
                                addonTransportUrl: $0.addonTransportUrl,
                                catalogType: $0.catalogType,
                                progress: nil,
                                topTenRank: nil
                            )
                        },
                        canLoadMore: false
                    )
                )
            } catch {
                lastError = error
            }
        }
        if rows.isEmpty, let lastError {
            throw lastError
        }
        return rows
    }

    func loadSearchItems(requests: [FluxaAppleSearchRequest]) async throws -> [AppleCatalogItemSnapshot] {
        var items = [AppleCatalogItemSnapshot]()
        var lastError: Error?
        for request in requests {
            do {
                let (data, response) = try await session.data(from: request.url)
                guard let httpResponse = response as? HTTPURLResponse,
                      (200..<300).contains(httpResponse.statusCode) else {
                    throw URLError(.badServerResponse)
                }
                guard let catalogItems = FluxaApple.shared.parseCatalogItems(
                    body: String(decoding: data, as: UTF8.self),
                    fallbackType: request.contentType,
                    addonTransportUrl: request.addonTransportUrl,
                    catalogType: request.catalogType
                ) else {
                    throw URLError(.cannotParseResponse)
                }
                items.append(contentsOf: catalogItems.map {
                    AppleCatalogItemSnapshot(
                        id: $0.id,
                        type: $0.type,
                        title: $0.title,
                        subtitle: $0.subtitle,
                        artworkUrl: $0.artworkUrl,
                        logoUrl: $0.logoUrl,
                        addonTransportUrl: $0.addonTransportUrl,
                        catalogType: $0.catalogType,
                        progress: nil,
                        topTenRank: nil
                    )
                })
            } catch {
                lastError = error
            }
        }
        if items.isEmpty, let lastError {
            throw lastError
        }
        return items
    }
}
