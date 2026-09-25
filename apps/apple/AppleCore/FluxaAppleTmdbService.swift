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

    func enrichMeta(
        _ baseMeta: FluxaAppleJsonValue,
        contentType: String,
        id: String,
        language: String
    ) async -> FluxaAppleJsonValue {
        let defaults = UserDefaults.standard
        let apiKey = defaults.string(forKey: "fluxa.apple.settings.tmdb_api_key")
            ?? defaults.string(forKey: "fluxa.apple.settings.tmdbApiKey")
            ?? ""
        guard !apiKey.isEmpty else { return baseMeta }

        let flags: [String: Bool] = [
            "artwork": setting(defaults, "tmdbEnrichArtworkEnabled", legacy: "tmdb_enrich_artwork_enabled"),
            "description": setting(defaults, "tmdbEnrichDescriptionEnabled", legacy: "tmdb_enrich_description_enabled"),
            "genresKeywords": setting(defaults, "tmdbEnrichGenresKeywordsEnabled", legacy: "tmdb_enrich_genres_keywords_enabled"),
            "castCrew": setting(defaults, "tmdbEnrichCastCrewEnabled", legacy: "tmdb_enrich_cast_crew_enabled"),
            "network": setting(defaults, "tmdbEnrichNetworkEnabled", legacy: "tmdb_enrich_network_enabled"),
            "ratings": setting(defaults, "tmdbRatingsEnabled", legacy: "tmdb_ratings_enabled"),
            "collection": setting(defaults, "tmdbCollectionInfoEnabled", legacy: "tmdb_collection_info_enabled"),
            "statusSchedule": setting(defaults, "tmdbEnrichStatusScheduleEnabled", legacy: "tmdb_enrich_status_schedule_enabled"),
            "originTitles": setting(defaults, "tmdbEnrichOriginTitlesEnabled", legacy: "tmdb_enrich_origin_titles_enabled"),
            "watchProviders": setting(defaults, "tmdbEnrichWatchProvidersEnabled", legacy: "tmdb_enrich_watch_providers_enabled"),
            "episodeStills": setting(defaults, "tmdbEpisodeImagesEnabled", legacy: "tmdb_episode_images_enabled"),
        ]
        guard flags.values.contains(true),
              var plan = FluxaCoreStremio.tmdbBuiltinMetaRequestPlan(
                contentType: contentType,
                contentId: id,
                apiKey: apiKey,
                language: language
              ) else { return baseMeta }

        if let findUrl = plan["findUrl"] as? String,
           let find = await loadJson(findUrl),
           let resolved = FluxaCoreStremio.tmdbBuiltinMetaUrlsFromFind(
            find: find,
            contentType: contentType,
            apiKey: apiKey,
            language: language
           ) {
            plan = resolved
        }

        guard let detailsUrl = plan["detailsUrl"] as? String,
              let details = await loadJson(detailsUrl) else { return baseMeta }
        async let credits = loadJson(plan["creditsUrl"] as? String)
        async let images = loadJson(plan["imagesUrl"] as? String)
        async let externalIds = loadJson(plan["externalIdsUrl"] as? String)
        async let keywords = loadJson(plan["keywordsUrl"] as? String)
        async let alternativeTitles = loadJson(plan["alternativeTitlesUrl"] as? String)
        async let contentRatings = loadJson(plan["contentRatingsUrl"] as? String)
        async let watchProviders = loadJson(plan["watchProvidersUrl"] as? String)
        let (creditsValue, imagesValue, externalIdsValue, keywordsValue, titlesValue, ratingsValue, providersValue) = await (
            credits, images, externalIds, keywords, alternativeTitles, contentRatings, watchProviders
        )

        var extras: [String: Any] = [:]
        extras["keywords"] = keywordsValue ?? NSNull()
        extras["alternativeTitles"] = titlesValue ?? NSNull()
        extras["contentRatings"] = ratingsValue ?? NSNull()
        extras["watchProviders"] = providersValue ?? NSNull()
        guard let extrasData = try? JSONSerialization.data(withJSONObject: extras),
              let extrasJson = String(data: extrasData, encoding: .utf8),
              var tmdbMeta = FluxaCoreStremio.tmdbFullMetaToMeta(
                detailsJson: jsonString(details),
                creditsJson: jsonString(creditsValue),
                imagesJson: jsonString(imagesValue),
                externalIdsJson: jsonString(externalIdsValue),
                extrasJson: extrasJson,
                requestedType: contentType,
                language: language
              ) else { return baseMeta }

        if contentType == "series",
           let tmdbId = plan["tmdbId"] as? String,
           let detailsObject = details as? [String: Any],
           let seasons = detailsObject["seasons"] as? [[String: Any]] {
            let seasonNumbers = seasons.compactMap { $0["season_number"] as? Int }.filter { $0 > 0 }.sorted()
            let seriesId = string(tmdbMeta, "id") ?? id
            var videos = [FluxaAppleJsonValue]()
            for batchStart in stride(from: 0, to: seasonNumbers.count, by: 4) {
                let batch = Array(seasonNumbers[batchStart..<min(batchStart + 4, seasonNumbers.count)])
                let batchVideos = await withTaskGroup(of: [FluxaAppleJsonValue].self) { group in
                    for season in batch {
                        guard let url = FluxaCoreStremio.tmdbSeasonRequestUrl(
                            contentId: tmdbId,
                            season: season,
                            apiKey: apiKey,
                            language: language
                        ) else { continue }
                        group.addTask {
                            guard let seasonData = await self.loadJson(url) else { return [] }
                            return FluxaCoreStremio.tmdbEpisodesToVideos(
                                seasonJson: self.jsonString(seasonData),
                                seriesId: seriesId
                            )
                        }
                    }
                    var values = [[FluxaAppleJsonValue]]()
                    for await value in group { values.append(value) }
                    return values.flatMap { $0 }
                }
                videos.append(contentsOf: batchVideos)
            }
            if !videos.isEmpty, case .object(var fields) = tmdbMeta {
                fields["videos"] = .array(videos)
                tmdbMeta = .object(fields)
            }
        }

        return FluxaCoreStremio.tmdbMergeEnrichment(base: baseMeta, tmdb: tmdbMeta, flags: flags) ?? baseMeta
    }

    private func setting(_ defaults: UserDefaults, _ key: String, legacy: String) -> Bool {
        if let value = defaults.object(forKey: "fluxa.apple.settings.\(key)") as? Bool { return value }
        if let value = defaults.object(forKey: "fluxa.apple.settings.\(legacy)") as? Bool { return value }
        return true
    }

    private func loadJson(_ rawUrl: String?) async -> Any? {
        guard let rawUrl, let url = URL(string: rawUrl),
              let (data, response) = try? await session.data(from: url),
              let httpResponse = response as? HTTPURLResponse,
              (200..<300).contains(httpResponse.statusCode) else { return nil }
        return try? JSONSerialization.jsonObject(with: data)
    }

    private func jsonString(_ value: Any?) -> String {
        guard let value, JSONSerialization.isValidJSONObject(value),
              let data = try? JSONSerialization.data(withJSONObject: value),
              let string = String(data: data, encoding: .utf8) else { return "{}" }
        return string
    }

    private func string(_ value: FluxaAppleJsonValue, _ key: String) -> String? {
        guard case .object(let fields) = value, case .string(let text)? = fields[key] else { return nil }
        return text
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
