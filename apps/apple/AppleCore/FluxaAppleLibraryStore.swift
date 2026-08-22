import Foundation

final class FluxaAppleLibraryStore {
    private let defaults: UserDefaults
    private let decoder = JSONDecoder()

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    func watchlist() -> [FluxaAppleJsonValue] {
        let data = defaults.string(forKey: Self.watchlistKey)?.data(using: .utf8)
            ?? defaults.data(forKey: Self.legacyWatchlistKey)
        guard let data else {
            return []
        }
        return (try? decoder.decode([FluxaAppleJsonValue].self, from: data)) ?? []
    }

    func watchlistState(item: FluxaAppleJsonValue) -> (watchlist: [FluxaAppleJsonValue], isInWatchlist: Bool) {
        guard case .object(let candidate) = item,
              case .string(let id)? = candidate["id"] else {
            return (watchlist(), false)
        }
        let items = watchlist()
        let containsItem = items.contains(where: { value in
            guard case .object(let existing) = value,
                  case .string(let existingId)? = existing["id"] else {
                return false
            }
            return existingId == id
        })
        return (items, containsItem)
    }

    private static let watchlistKey = "fluxa.apple-default.watchlist"
    private static let legacyWatchlistKey = "fluxa.apple.watchlist"
}
