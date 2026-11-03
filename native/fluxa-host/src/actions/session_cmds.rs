use super::*;

pub(crate) fn session_commands(action: &NativeAction, profile: &Value) -> Option<Vec<Value>> {
    let commands = match action {
        NativeAction::CoreCommand { command } => vec![command.clone()],
        NativeAction::LoadMore { .. } => Vec::new(),
        NativeAction::Folder { .. }
        | NativeAction::DetailModal { .. }
        | NativeAction::OpenDetailPage => {
            return None;
        }
        NativeAction::Back => vec![navigation(Route::Home)],
        NativeAction::Navigate { destination } => match destination {
            Route::Home => vec![
                navigation(Route::Home),
                json!({
                    "type": "refreshContinueWatchingRequested",
                    "profile": profile,
                    "language": profile_language(profile),
                    "source": "navigation",
                }),
            ],
            Route::Library => vec![
                navigation(Route::Library),
                json!({"type": "libraryHydrateRequested", "profileId": profile.get("id")}),
            ],
            Route::Discover => vec![
                navigation(Route::Discover),
                discover_command(profile, "movie", "", "", "", true),
            ],
            Route::Calendar => {
                let (year, month) = current_year_month();
                vec![
                    navigation(Route::Calendar),
                    json!({"type": "calendarMonthRequested", "profile": profile, "year": year, "month": month, "plannedItems": []}),
                ]
            }
            Route::Settings => vec![navigation(Route::Settings)],
            Route::Shorts => vec![navigation(Route::Shorts)],
            _ => return None,
        },
        NativeAction::DiscoverType { content_type } => {
            vec![discover_command(profile, content_type, "", "", "", true)]
        }
        NativeAction::DiscoverCatalog {
            content_type,
            catalog_key,
            extra_name,
            extra_value,
            ..
        }
        | NativeAction::DiscoverFilters {
            content_type,
            catalog_key,
            extra_name,
            extra_value,
            ..
        } => vec![discover_command(
            profile,
            content_type,
            catalog_key,
            extra_name,
            extra_value,
            false,
        )],
        NativeAction::CalendarMonth { year, month } => vec![
            json!({"type": "calendarMonthRequested", "profile": profile, "year": year, "month": month, "plannedItems": []}),
        ],
        NativeAction::Detail { id, item_type, .. } | NativeAction::Play { id, item_type } => vec![
            navigation(Route::Detail),
            json!({
                "type": "detailLoadRequested",
                "id": id,
                "contentType": item_type,
                "language": profile_language(profile),
                "profile": profile,
                "preview": match action {
                    NativeAction::Detail { preview, .. } => preview.clone(),
                    _ => Value::Null,
                },
            }),
            json!({
                "type": "detailSecondaryRequested",
                "id": id,
                "contentType": item_type,
                "language": profile_language(profile),
                "profile": profile,
            }),
        ],
        NativeAction::ToggleWatchlist { item } => {
            vec![json!({"type": "toggleWatchlistRequested", "item": item, "profile": profile})]
        }
        NativeAction::ToggleFavorite { item } => {
            vec![json!({"type": "toggleLibraryStatusRequested", "list": "liked", "item": item})]
        }
        NativeAction::SettingsChange { key, value } => {
            let mut commands = vec![json!({"type": "settingsChanged", "key": key, "value": value})];
            let exclusive = match (key.as_str(), value.as_bool()) {
                ("expandedPosters", Some(true)) => Some("posterHoverPreview"),
                ("posterHoverPreview", Some(true)) => Some("expandedPosters"),
                _ => None,
            };
            if let Some(other) = exclusive {
                commands.push(json!({"type": "settingsChanged", "key": other, "value": false}));
            }
            commands
        }
        NativeAction::StartPlayback { item } => {
            vec![player::direct_playback_command(item, profile)]
        }
        NativeAction::PlayTrailer { .. }
        | NativeAction::SettingsSection { .. }
        | NativeAction::AccountToggle { .. }
        | NativeAction::LogOut
        | NativeAction::MediaServer { .. }
        | NativeAction::MediaCommand { .. }
        | NativeAction::OpenFile { .. }
        | NativeAction::OauthCallback { .. }
        | NativeAction::OpenLink { .. }
        | NativeAction::Search { .. } => return None,
    };
    Some(commands)
}
