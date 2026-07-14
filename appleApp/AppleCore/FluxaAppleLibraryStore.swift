import Foundation

final class FluxaAppleLibraryStore {
    private let defaults: UserDefaults
    private let encoder = JSONEncoder()
    private let decoder = JSONDecoder()

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    func watchlist() -> [FluxaAppleJsonValue] {
        guard let data = defaults.data(forKey: Self.watchlistKey) else {
            return []
        }
        return (try? decoder.decode([FluxaAppleJsonValue].self, from: data)) ?? []
    }

    func toggleWatchlist(item: FluxaAppleJsonValue) -> (watchlist: [FluxaAppleJsonValue], isInWatchlist: Bool) {
        guard case .object(let candidate) = item,
              case .string(let id)? = candidate["id"] else {
            return (watchlist(), false)
        }
        var items = watchlist()
        if let index = items.firstIndex(where: { value in
            guard case .object(let existing) = value,
                  case .string(let existingId)? = existing["id"] else {
                return false
            }
            return existingId == id
        }) {
            items.remove(at: index)
            save(items)
            return (items, false)
        }
        items.insert(item, at: 0)
        save(items)
        return (items, true)
    }

    private func save(_ items: [FluxaAppleJsonValue]) {
        guard let data = try? encoder.encode(items) else {
            return
        }
        defaults.set(data, forKey: Self.watchlistKey)
    }

    private static let watchlistKey = "fluxa.apple.watchlist"
}
