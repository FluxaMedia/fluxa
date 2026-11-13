use super::*;

mod playback_items;
mod remember;
mod routing;
mod session_cmds;
mod settings_actions;
mod snapshot;
mod text_edit;
pub(super) use playback_items::*;
pub(super) use remember::*;
pub(super) use routing::*;
pub(super) use session_cmds::*;
pub(super) use settings_actions::*;
pub(super) use snapshot::*;
pub(super) use text_edit::*;

pub(super) fn native_action_for_node(
    node: u64,
    home: &HomeModel,
    library: &LibraryModel,
    library_tab: LibraryTab,
    discover: &DiscoverModel,
    folder: &FolderModel,
    folder_tab: usize,
    calendar: &CalendarModel,
    detail: &DetailModel,
    settings: &SettingsModel,
    route: Route,
    hero: Option<&HomeHero>,
) -> Option<NativeAction> {
    let destination = match node {
        NODE_HOME => Some(Route::Home),
        NODE_LIBRARY => Some(Route::Library),
        NODE_DISCOVER => Some(Route::Discover),
        NODE_CALENDAR => Some(Route::Calendar),
        fluxa_ui::NODE_SHORTS => Some(Route::Shorts),
        fluxa_ui::NODE_LIVE_TV => Some(Route::Live),
        NODE_PROFILE => Some(Route::Settings),
        fluxa_ui::NODE_SIDEBAR_SEARCH => Some(Route::Discover),
        fluxa_ui::NODE_SIDEBAR_DOWNLOADS => Some(Route::Library),
        _ => None,
    };
    if let Some(destination) = destination {
        return Some(NativeAction::Navigate { destination });
    }
    if route == Route::Discover {
        if let Some(entry) =
            fluxa_ui::discover_recent(node).and_then(|index| discover.recent_searches.get(index))
        {
            return Some(NativeAction::Search {
                query: entry.clone(),
            });
        }
        if node == fluxa_ui::NODE_DISCOVER_TYPE_BASE {
            return Some(NativeAction::DiscoverType {
                content_type: if node == fluxa_ui::NODE_DISCOVER_TYPE_BASE {
                    "movie".to_owned()
                } else {
                    "series".to_owned()
                },
            });
        }
        if node == fluxa_ui::NODE_DISCOVER_CATALOG_BASE {
            let index = (node - fluxa_ui::NODE_DISCOVER_CATALOG_BASE) as usize;
            if let Some(catalog) = discover
                .catalogs
                .iter()
                .filter(|catalog| catalog.content_type == discover.content_type)
                .nth(index)
            {
                return Some(NativeAction::DiscoverCatalog {
                    content_type: discover.content_type.clone(),
                    catalog_key: catalog.key.clone(),
                    extra_name: discover.selected_extra_name.clone(),
                    extra_value: discover.selected_extra_value.clone(),
                    query: discover.query.clone(),
                });
            }
        }
    }
    if route == Route::Calendar {
        if (fluxa_ui::NODE_CALENDAR_EVENT_BASE..fluxa_ui::NODE_CALENDAR_EVENT_BASE + 10)
            .contains(&node)
        {
            let day = calendar.selected_day?;
            let entry = calendar
                .entries_for_day(day)
                .nth((node - fluxa_ui::NODE_CALENDAR_EVENT_BASE) as usize)?;
            return Some(NativeAction::Detail {
                id: entry.card.id.clone()?,
                item_type: entry.card.item_type.clone()?,
                preview: entry.card.raw.cloned(),
            });
        }
        if node == fluxa_ui::NODE_CALENDAR_PREV || node == fluxa_ui::NODE_CALENDAR_NEXT {
            let delta = if node == fluxa_ui::NODE_CALENDAR_PREV {
                -1
            } else {
                1
            };
            let mut year = calendar.year;
            let mut month = calendar.month + delta;
            if month < 1 {
                year -= 1;
                month = 12;
            } else if month > 12 {
                year += 1;
                month = 1;
            }
            return Some(NativeAction::CalendarMonth { year, month });
        }
    }
    if route == Route::Settings {
        if node == fluxa_ui::NODE_SETTINGS_BACK {
            return Some(NativeAction::Navigate {
                destination: Route::Home,
            });
        }
        if node == fluxa_ui::NODE_SETTINGS_SWITCH_PROFILE {
            return Some(NativeAction::Navigate {
                destination: Route::Profiles,
            });
        }
        if node == fluxa_ui::NODE_SETTINGS_LOG_OUT {
            return Some(NativeAction::LogOut);
        }
        if let Some(provider) = node
            .checked_sub(fluxa_ui::NODE_SETTINGS_ACCOUNT_BASE)
            .and_then(|slot| fluxa_ui::ACCOUNT_PROVIDERS.get(slot as usize))
        {
            return Some(NativeAction::AccountToggle {
                provider: (*provider).to_owned(),
            });
        }
        if matches!(
            node,
            fluxa_ui::NODE_SETTINGS_SERVER_JELLYFIN
                | fluxa_ui::NODE_SETTINGS_SERVER_EMBY
                | fluxa_ui::NODE_SETTINGS_SERVER_PLEX
                | fluxa_ui::NODE_SETTINGS_SERVER_CONNECT
        ) || fluxa_ui::server_index(node).is_some()
        {
            return Some(NativeAction::MediaServer { node });
        }
        if let Some(index) = fluxa_ui::settings_page_for_node(node) {
            return Some(NativeAction::SettingsSection { index });
        }
        if (fluxa_ui::NODE_SETTINGS_ROW_BASE
            ..fluxa_ui::NODE_SETTINGS_ROW_BASE
                + fluxa_ui::SETTINGS_SECTIONS
                    .iter()
                    .map(|section| section.rows.len())
                    .sum::<usize>() as u64)
            .contains(&node)
        {
            let index = (node - fluxa_ui::NODE_SETTINGS_ROW_BASE) as usize;
            let setting = fluxa_ui::settings_row_by_index(index)?;
            return Some(NativeAction::SettingsChange {
                key: setting.key.to_owned(),
                value: settings.next_value_for(setting, home.form_factor.into()),
            });
        }
    }
    if route == Route::Detail {
        if node == fluxa_ui::NODE_DETAIL_BACK {
            return Some(NativeAction::Back);
        }
        if node == fluxa_ui::NODE_DETAIL_WATCHLIST {
            return Some(NativeAction::ToggleWatchlist {
                item: detail.item.clone(),
            });
        }
        if node == fluxa_ui::NODE_DETAIL_FAVORITE {
            return Some(NativeAction::ToggleFavorite {
                item: detail.item.clone(),
            });
        }
        if let Some(index) = node
            .checked_sub(fluxa_ui::NODE_DETAIL_TRAILER_BASE)
            .map(|offset| offset as usize)
            .filter(|index| *index < detail.trailer_cards.len())
        {
            let mut urls = vec![detail.trailer_cards[index].url.clone()];
            urls.extend(
                detail
                    .trailer_cards
                    .iter()
                    .enumerate()
                    .filter(|(other, _)| *other != index)
                    .map(|(_, card)| card.url.clone()),
            );
            return Some(NativeAction::PlayTrailer {
                item: detail.item.clone(),
                urls,
            });
        }
        if node == fluxa_ui::NODE_DETAIL_PLAY {
            if !detail.id.is_empty() {
                let first = detail
                    .episodes
                    .iter()
                    .find(|episode| episode.season > 0)
                    .or(detail.episodes.first())
                    .map(|episode| episode.id.clone());
                return Some(NativeAction::StartPlayback {
                    item: playback_item(&detail.item, detail.resume.as_ref(), first),
                });
            }
        }
        if node >= fluxa_ui::NODE_DETAIL_EPISODE_BASE {
            let episode = detail
                .episodes
                .get((node - fluxa_ui::NODE_DETAIL_EPISODE_BASE) as usize)?;
            let mut item = detail.item.clone();
            item.as_object_mut()?
                .insert("lastVideoId".to_owned(), episode.id.clone().into());
            return Some(NativeAction::StartPlayback { item });
        }
        if node >= fluxa_ui::NODE_DETAIL_SEASON_BASE {
            return None;
        }
        if (fluxa_ui::NODE_DETAIL_SIMILAR_BASE..fluxa_ui::NODE_DETAIL_CAST_BASE).contains(&node) {
            let card = detail.linked_card(node)?;
            return Some(NativeAction::Detail {
                id: card.id.as_ref()?.clone(),
                item_type: card.item_type.as_ref()?.clone(),
                preview: card.raw.cloned(),
            });
        }
    }
    if matches!(route, Route::Home | Route::Discover) && node >= fluxa_ui::NODE_PREVIEW_BASE {
        let slot = node - fluxa_ui::NODE_PREVIEW_BASE;
        let index = (slot / fluxa_ui::PREVIEW_ACTIONS) as usize;
        let card = if route == Route::Home {
            home.card_at(index)?
        } else {
            discover.results.get(index)?
        };
        let id = card.id.as_ref()?.clone();
        let item_type = card.item_type.as_ref()?.clone();
        return Some(match slot % fluxa_ui::PREVIEW_ACTIONS {
            0 => {
                let series = matches!(item_type.as_str(), "series" | "tv" | "show");
                NativeAction::StartPlayback {
                    item: playback_item(
                        &card.raw,
                        home.resume_for(&id),
                        series.then(|| format!("{id}:1:1")),
                    ),
                }
            }
            1 => NativeAction::ToggleWatchlist {
                item: card.raw.cloned(),
            },
            2 => NativeAction::Detail {
                id,
                item_type,
                preview: card.raw.cloned(),
            },
            3 => {
                let watched = library
                    .personal
                    .get(&id)
                    .is_some_and(|personal| personal.watched);
                NativeAction::CoreCommand {
                    command: json!({
                        "type": "markWatchedRequested",
                        "seriesId": id,
                        "videoIds": [id],
                        "watched": !watched,
                        "meta": card.raw,
                    }),
                }
            }
            _ => NativeAction::CoreCommand {
                command: json!({
                    "type": "clearPlaybackProgressRequested",
                    "meta": card.raw,
                }),
            },
        });
    }
    let (id, item_type, preview) = if route == Route::Library && node >= NODE_CARD_BASE {
        let card = library
            .cards(library_tab)
            .get((node - NODE_CARD_BASE) as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &*card.raw)
    } else if route == Route::Discover && node >= NODE_CARD_BASE {
        let card = discover.results.get((node - NODE_CARD_BASE) as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &*card.raw)
    } else if route == Route::Folder {
        if node == fluxa_ui::NODE_FOLDER_BACK {
            return Some(NativeAction::Back);
        }
        let card = folder.card_at(folder_tab, node.checked_sub(NODE_CARD_BASE)? as usize)?;
        (card.id.as_ref()?, card.item_type.as_ref()?, &*card.raw)
    } else if node == fluxa_ui::NODE_HERO_WATCHLIST {
        return Some(NativeAction::ToggleWatchlist {
            item: hero?.raw.clone(),
        });
    } else if node == NODE_PLAY || node == NODE_MORE_INFO {
        if let Some(hero) = hero
            && let (Some(id), Some(item_type)) = (hero.item_id.as_ref(), hero.item_type.as_ref())
        {
            if node == NODE_MORE_INFO {
                return Some(NativeAction::Detail {
                    id: id.clone(),
                    item_type: item_type.clone(),
                    preview: hero.raw.clone(),
                });
            }
            let series = matches!(item_type.as_str(), "series" | "tv" | "show");
            return Some(NativeAction::StartPlayback {
                item: playback_item(
                    &hero.raw,
                    home.resume_for(id),
                    series.then(|| format!("{id}:1:1")),
                ),
            });
        }
        (
            home.item_id.as_ref()?,
            home.item_type.as_ref()?,
            &Value::Null,
        )
    } else if route == Route::Home && node >= NODE_CARD_BASE {
        let card = home.card_at((node - NODE_CARD_BASE) as usize)?;
        if card.row_kind == fluxa_ui::HomeRowKind::Collection {
            return Some(NativeAction::Folder {
                id: card.id.as_ref()?.clone(),
            });
        }
        if card.row_kind == fluxa_ui::HomeRowKind::Continue && !card.upcoming {
            return Some(NativeAction::StartPlayback {
                item: playback_item(&card.raw, Some(card), None),
            });
        }
        (card.id.as_ref()?, card.item_type.as_ref()?, &*card.raw)
    } else {
        return None;
    };
    if node == NODE_PLAY {
        Some(NativeAction::Play {
            id: id.clone(),
            item_type: item_type.clone(),
        })
    } else {
        Some(NativeAction::Detail {
            id: id.clone(),
            item_type: item_type.clone(),
            preview: preview.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailer_resolution_does_not_reproject() {
        let old = Snapshot::from_value(
            json!({"home": {"categories": [1, 2]}, "trailer": {"resolutions": {}}}),
        );
        let new = Snapshot::from_value(
            json!({"home": {"categories": [1, 2]}, "trailer": {"resolutions": {"a": 1}}}),
        );
        assert!(!projection_inputs_changed(&old, &new));
    }

    #[test]
    fn new_catalog_page_reprojects() {
        let old = Snapshot::from_value(json!({"home": {"categories": [1]}, "trailer": {}}));
        let new = Snapshot::from_value(json!({"home": {"categories": [1, 2]}, "trailer": {}}));
        assert!(projection_inputs_changed(&old, &new));
        assert!(projection_inputs_changed(
            &old,
            &Snapshot::from_value(json!({"trailer": {}}))
        ));
    }
}
