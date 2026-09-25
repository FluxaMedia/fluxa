import Foundation

final class FluxaAppleLibraryStore {
    private let defaults: UserDefaults
    private let decoder = JSONDecoder()

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    func watchlist() -> [FluxaAppleJsonValue] {
        if case .object(let library) = snapshot(),
           case .array(let items)? = library["watchlist"] {
            return items
        }
        return legacyWatchlist()
    }

    func snapshot() -> FluxaAppleJsonValue {
        if let data = defaults.string(forKey: Self.libraryKey)?.data(using: .utf8),
           let library = try? decoder.decode(FluxaAppleJsonValue.self, from: data),
           case .object = library {
            return library
        }
        return .object([
            "watchlist": .array(legacyWatchlist()),
            "continueWatching": .array([]),
            "liked": .array([]),
            "watched": .object([:]),
            "completed": .array([]),
            "dropped": .array([])
        ])
    }

    func applyCommand(_ command: FluxaAppleJsonValue, source: String?) throws -> FluxaAppleJsonValue {
        if let source, !["", "local", "fluxa"].contains(source.lowercased()) {
            throw NSError(domain: "FluxaAppleUnsupportedLibraryProvider", code: 1)
        }
        let nowIso = ISO8601DateFormatter().string(from: Date())
        guard let plan = FluxaCoreStremio.libraryCommandPlan(
            library: snapshot(),
            command: command,
            nowIso: nowIso
        ), case .object(let values) = plan,
           let updatedLibrary = values["library"],
           case .object(let commandValues) = command,
           case .string(let type)? = commandValues["type"] else {
            throw URLError(.cannotParseResponse)
        }
        save(updatedLibrary)
        let fields: [String: FluxaAppleJsonValue] = {
            guard case .object(let library) = updatedLibrary else { return [:] }
            return library
        }()
        switch type {
        case "toggleWatchlist":
            let itemId: String? = {
                guard case .object(let item)? = commandValues["item"],
                      case .string(let id)? = item["id"] else { return nil }
                return id
            }()
            let inWatchlist = fields["watchlist"].flatMap { value -> Bool? in
                guard case .array(let items) = value else { return nil }
                return items.contains { item in
                    guard case .object(let entry) = item,
                          case .string(let id)? = entry["id"] else { return false }
                    return id == itemId
                }
            } ?? false
            return .object([
                "watchlist": fields["watchlist"] ?? .array([]),
                "isInWatchlist": .boolean(inWatchlist)
            ])
        case "toggleLibraryStatus":
            return .object([
                "watchlist": fields["watchlist"] ?? .array([]),
                "completed": fields["completed"] ?? .array([]),
                "dropped": fields["dropped"] ?? .array([])
            ])
        case "markWatched":
            let videoIds: [String] = {
                return commandValues["videoIds"].flatMap { value in
                    guard case .array(let ids) = value else { return nil }
                    return ids.compactMap { id in
                        guard case .string(let value) = id else { return nil }
                        return value
                    }
                } ?? []
            }()
            let watchedValue: Bool = {
                guard case .boolean(let watched)? = commandValues["watched"] else { return true }
                return watched
            }()
            let localWatched: [FluxaAppleJsonValue] = watchedValue ? videoIds.map(FluxaAppleJsonValue.string) : []
            return .object([
                "watchlist": fields["watchlist"] ?? .array([]),
                "localWatchedVideoIds": .array(localWatched)
            ])
        default:
            throw NSError(domain: "FluxaAppleUnsupportedLibraryCommand", code: 1)
        }
    }

    private func legacyWatchlist() -> [FluxaAppleJsonValue] {
        let data = defaults.string(forKey: Self.watchlistKey)?.data(using: .utf8)
            ?? defaults.data(forKey: Self.legacyWatchlistKey)
        guard let data else {
            return []
        }
        return (try? decoder.decode([FluxaAppleJsonValue].self, from: data)) ?? []
    }

    private func save(_ library: FluxaAppleJsonValue) {
        guard let data = try? JSONEncoder().encode(library),
              let value = String(data: data, encoding: .utf8) else { return }
        defaults.set(value, forKey: Self.libraryKey)
        if case .object(let fields) = library,
           let watchlist = fields["watchlist"],
           let watchlistData = try? JSONEncoder().encode(watchlist),
           let watchlistValue = String(data: watchlistData, encoding: .utf8) {
            defaults.set(watchlistValue, forKey: Self.watchlistKey)
        }
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
    private static let libraryKey = "fluxa.apple-default.library"
}
