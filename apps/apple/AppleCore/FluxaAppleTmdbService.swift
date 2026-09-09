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
              let numericId = FluxaCoreStremio.tmdbNumericId(id) else {
            return []
        }
        let mediaType = FluxaCoreStremio.tmdbContentType(contentType)
        guard let urlString = FluxaCoreStremio.tmdbRecommendationsUrl(
            contentType: contentType,
            tmdbId: numericId,
            apiKey: apiKey,
            language: language
        ), let url = URL(string: urlString) else { return [] }
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
                  let resolvedType = FluxaCoreStremio.tmdbItemContentType(
                      mediaType: string(result["media_type"]),
                      hasFirstAirDate: string(result["first_air_date"])?.isEmpty == false,
                      requestedType: contentType
                  ),
                  let title = string(result[FluxaCoreStremio.isSeriesContentType(resolvedType) ? "name" : "title"]),
                  !title.isEmpty else {
                return nil
            }
            return .object([
                "id": .string("tmdb:\(Int(resultId))"),
                "type": .string(resolvedType),
                "name": .string(title),
                "poster": FluxaCoreStremio.tmdbImageUrl(string(result["poster_path"]), size: "w500").map(FluxaAppleJsonValue.string) ?? .null,
                "background": FluxaCoreStremio.tmdbImageUrl(string(result["backdrop_path"]), size: "w1280").map(FluxaAppleJsonValue.string) ?? .null,
                "releaseInfo": .string(string(result[FluxaCoreStremio.isSeriesContentType(resolvedType) ? "first_air_date" : "release_date"]) ?? "")
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

}
