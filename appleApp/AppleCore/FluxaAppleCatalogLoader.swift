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
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()

    init(session: URLSession = .shared) {
        self.session = session
    }

    func loadSnapshot(requests: [FluxaAppleCatalogRequest]) async throws -> String {
        var rows = [FluxaAppleCatalogRow]()
        var lastError: Error?
        for request in requests {
            do {
                let (data, response) = try await session.data(from: request.url)
                guard let httpResponse = response as? HTTPURLResponse,
                      (200..<300).contains(httpResponse.statusCode) else {
                    throw URLError(.badServerResponse)
                }
                let catalog = try decoder.decode(FluxaAppleStremioCatalogResponse.self, from: data)
                rows.append(
                    FluxaAppleCatalogRow(
                        id: request.id,
                        title: request.title,
                        canLoadMore: false,
                        items: catalog.metas.map {
                            FluxaAppleCatalogItem(
                                id: $0.id,
                                type: $0.type ?? request.contentType,
                                title: $0.name,
                                subtitle: $0.releaseInfo ?? "",
                                artworkUrl: $0.poster,
                                logoUrl: $0.logo,
                                addonTransportUrl: request.addonTransportUrl,
                                catalogType: request.catalogType
                            )
                        }
                    )
                )
            } catch {
                lastError = error
            }
        }
        if rows.isEmpty, let lastError {
            throw lastError
        }
        return String(decoding: try encoder.encode(FluxaAppleCatalogSnapshot(rows: rows)), as: UTF8.self)
    }
}

private struct FluxaAppleStremioCatalogResponse: Decodable {
    let metas: [FluxaAppleStremioMeta]
}

private struct FluxaAppleStremioMeta: Decodable {
    let id: String
    let type: String?
    let name: String
    let poster: String?
    let logo: String?
    let releaseInfo: String?
}

private struct FluxaAppleCatalogSnapshot: Encodable {
    let rows: [FluxaAppleCatalogRow]
    let isLoading = false
}

private struct FluxaAppleCatalogRow: Encodable {
    let id: String
    let title: String
    let canLoadMore: Bool
    let items: [FluxaAppleCatalogItem]
}

private struct FluxaAppleCatalogItem: Encodable {
    let id: String
    let type: String
    let title: String
    let subtitle: String
    let artworkUrl: String?
    let logoUrl: String?
    let addonTransportUrl: String?
    let catalogType: String?
}
