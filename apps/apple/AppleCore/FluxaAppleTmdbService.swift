import Foundation

final class FluxaAppleTmdbService {
    private let session: URLSession

    init(session: URLSession = .shared) {
        self.session = session
    }

    func loadRecommendations(
        contentType: String,
        id: String,
        language: String,
        apiKey: String
    ) async throws -> [FluxaAppleJsonValue] {
        guard !apiKey.isEmpty,
              let numericId = Int(id.split(separator: ":").last.map(String.init) ?? id) else {
            return []
        }
        let mediaType = contentType == "series" || contentType == "tv" ? "tv" : "movie"
        guard var components = URLComponents(
            string: "https://api.themoviedb.org/3/\(mediaType)/\(numericId)/recommendations"
        ) else {
            return []
        }
        components.queryItems = [
            URLQueryItem(name: "api_key", value: apiKey),
            URLQueryItem(name: "language", value: language),
            URLQueryItem(name: "page", value: "1")
        ]
        guard let url = components.url else { return [] }
        let (data, response) = try await session.data(from: url)
        guard let httpResponse = response as? HTTPURLResponse,
              (200..<300).contains(httpResponse.statusCode) else {
            return []
        }
        let root = try JSONDecoder().decode([String: FluxaAppleJsonValue].self, from: data)
        guard case .array(let results)? = root["results"] else { return [] }

        return results.prefix(20).compactMap { value in
            guard case .object(let result) = value,
                  let resultId = number(result["id"]),
                  let title = string(result[mediaType == "tv" ? "name" : "title"]),
                  !title.isEmpty else {
                return nil
            }
            return .object([
                "id": .string("tmdb:\(Int(resultId))"),
                "type": .string(contentType == "anime" ? "anime" : (mediaType == "tv" ? "series" : "movie")),
                "name": .string(title),
                "poster": image(result["poster_path"], size: "w500"),
                "background": image(result["backdrop_path"], size: "w1280"),
                "releaseInfo": .string(string(result[mediaType == "tv" ? "first_air_date" : "release_date"]) ?? "")
            ])
        }
    }

    private func string(_ value: FluxaAppleJsonValue?) -> String? {
        guard case .string(let text)? = value else { return nil }
        return text
    }

    private func number(_ value: FluxaAppleJsonValue?) -> Double? {
        guard case .number(let number)? = value else { return nil }
        return number
    }

    private func image(_ value: FluxaAppleJsonValue?, size: String) -> FluxaAppleJsonValue {
        guard let path = string(value) else { return .null }
        return .string("https://image.tmdb.org/t/p/\(size)\(path)")
    }
}
