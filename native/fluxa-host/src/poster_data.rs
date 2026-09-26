use std::collections::HashSet;
use std::sync::{Arc, mpsc::Receiver};
use std::time::Duration;

use serde_json::Value;
use web_time::Instant;

use crate::RendererState;

const TRENDING_REFRESH: Duration = Duration::from_secs(6 * 60 * 60);
const MDBLIST_BATCH: usize = 200;

#[derive(Default)]
pub(crate) struct PosterData {
    enrichment: Arc<fluxa_ui::Enrichment>,
    trending_rx: Option<Receiver<Vec<Value>>>,
    trending_at: Option<(Instant, String)>,
    graded_rx: Vec<Receiver<Vec<Value>>>,
    requested: HashSet<String>,
    mdblist_key: String,
}

pub(crate) fn tick(state: &mut RendererState) {
    let data = &mut state.poster_data;
    let mut changed = false;
    if let Some(results) = data.trending_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
        data.trending_rx = None;
        if !results.is_empty() {
            Arc::make_mut(&mut data.enrichment).set_trending(&results);
            changed = true;
        }
    }
    data.graded_rx.retain(|rx| match rx.try_recv() {
        Ok(items) => {
            Arc::make_mut(&mut data.enrichment).add_mdblist(&items);
            changed = true;
            false
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => true,
        Err(_) => false,
    });
    if changed && let Some(gpu) = state.gpu.as_ref() {
        fluxa_ui::set_poster_enrichment(&gpu.egui_context, data.enrichment.clone());
        gpu.egui_context.request_repaint();
    }

    let Some(session) = state.session.as_ref() else {
        return;
    };
    let tmdb_key = state
        .settings
        .str_value("tmdbApiKey")
        .unwrap_or_default()
        .trim();
    let stale = data
        .trending_at
        .as_ref()
        .is_none_or(|(at, key)| key != tmdb_key || at.elapsed() > TRENDING_REFRESH);
    if !tmdb_key.is_empty() && stale && data.trending_rx.is_none() {
        data.trending_at = Some((Instant::now(), tmdb_key.to_owned()));
        data.trending_rx = Some(session.executor().fetch_trending(tmdb_key.to_owned()));
    }

    let mdblist_key = state
        .settings
        .str_value("mdblistApiKey")
        .unwrap_or_default()
        .trim();
    if mdblist_key != data.mdblist_key {
        data.mdblist_key = mdblist_key.to_owned();
        data.requested.clear();
    }
    if mdblist_key.is_empty() || data.graded_rx.len() >= 2 {
        return;
    }
    let mut movies = Vec::new();
    let mut shows = Vec::new();
    for card in state.home.rows.iter().flat_map(|row| &row.cards) {
        let Some(imdb) = card.overlay.imdb.as_deref() else {
            continue;
        };
        if data.enrichment.has_graded(imdb) || !data.requested.insert(imdb.to_owned()) {
            continue;
        }
        let series = matches!(card.item_type.as_deref(), Some("series" | "tv" | "show"));
        let batch = if series { &mut shows } else { &mut movies };
        if batch.len() < MDBLIST_BATCH {
            batch.push(imdb.to_owned());
        } else {
            data.requested.remove(imdb);
        }
    }
    for (media_type, ids) in [("movie", movies), ("show", shows)] {
        if !ids.is_empty() {
            data.graded_rx.push(session.executor().fetch_mdblist_media(
                mdblist_key.to_owned(),
                media_type,
                ids,
            ));
        }
    }
}
