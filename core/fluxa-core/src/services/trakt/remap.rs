use serde_json::Value;

#[derive(Clone)]
struct MappedEpisode {
    season: i64,
    episode: i64,
    title: String,
}

fn normalized_title(title: &str) -> String {
    let words: Vec<String> = title
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_owned)
        .collect();
    let joined = words.join(" ");
    let generic = matches!(words.as_slice(), [first, number]
        if matches!(first.as_str(), "episode" | "ep" | "e") && number.chars().all(|c| c.is_ascii_digit()));
    if generic { String::new() } else { joined }
}

fn ordered_episodes(entries: impl Iterator<Item = MappedEpisode>) -> Vec<MappedEpisode> {
    let mut episodes: Vec<MappedEpisode> = entries.filter(|entry| entry.season > 0).collect();
    episodes.sort_by_key(|entry| (entry.season, entry.episode));
    episodes
}

fn season_sizes(episodes: &[MappedEpisode]) -> std::collections::BTreeMap<i64, usize> {
    episodes
        .iter()
        .fold(Default::default(), |mut sizes, entry| {
            *sizes.entry(entry.season).or_default() += 1;
            sizes
        })
}

pub(crate) fn trakt_remap_video_ids_json(args_json: &str) -> Option<String> {
    let args: Value = serde_json::from_str(args_json).ok()?;
    let video_ids: Vec<String> = serde_json::from_value(args.get("videoIds")?.clone()).ok()?;
    let episode_of = |entry: &Value| {
        Some(MappedEpisode {
            season: entry.get("season")?.as_i64()?,
            episode: entry
                .get("number")
                .or_else(|| entry.get("episode"))?
                .as_i64()?,
            title: entry
                .get("title")
                .and_then(Value::as_str)
                .map(normalized_title)
                .unwrap_or_default(),
        })
    };
    let addon = ordered_episodes(
        args.get("addonEpisodes")?
            .as_array()?
            .iter()
            .filter_map(episode_of),
    );
    let trakt = ordered_episodes(
        args.get("traktSeasons")?
            .as_array()?
            .iter()
            .flat_map(|season| season.get("episodes")?.as_array().cloned())
            .flatten()
            .filter_map(|entry| episode_of(&entry)),
    );
    if addon.is_empty() || trakt.is_empty() || season_sizes(&addon) == season_sizes(&trakt) {
        return serde_json::to_string(&video_ids).ok();
    }
    let to_addon = args.get("direction").and_then(Value::as_str) == Some("toAddon");
    let (source, target_list) = if to_addon {
        (&trakt, &addon)
    } else {
        (&addon, &trakt)
    };
    let mapped: Vec<String> = video_ids
        .iter()
        .map(|video_id| {
            let Some((prefix, season, number)) = split_episode_id(video_id) else {
                return video_id.clone();
            };
            let Some(index) = source
                .iter()
                .position(|entry| entry.season == season && entry.episode == number)
            else {
                return video_id.clone();
            };
            let title = &source[index].title;
            let by_title: Vec<&MappedEpisode> = target_list
                .iter()
                .filter(|entry| !title.is_empty() && entry.title == *title)
                .collect();
            let target = match by_title.as_slice() {
                [only] => Some((*only).clone()),
                _ if source.len() == target_list.len() => target_list.get(index).cloned(),
                _ => None,
            };
            target.map_or_else(
                || video_id.clone(),
                |target| format!("{prefix}:{}:{}", target.season, target.episode),
            )
        })
        .collect();
    serde_json::to_string(&mapped).ok()
}

fn split_episode_id(video_id: &str) -> Option<(&str, i64, i64)> {
    let (rest, number) = video_id.rsplit_once(':')?;
    let (prefix, season) = rest.rsplit_once(':')?;
    Some((prefix, season.parse().ok()?, number.parse().ok()?))
}
