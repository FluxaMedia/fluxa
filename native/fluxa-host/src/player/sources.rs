use super::*;

pub(super) type SourceRowsKey = (usize, usize, u64, u64);

impl PlayerSession {
    pub(super) fn rows_for(
        &self,
        streams: &[Value],
    ) -> std::sync::Arc<Vec<fluxa_ui::PlayerSource>> {
        let key = (
            streams.as_ptr() as usize,
            streams.len(),
            self.feed_version,
            crate::stream_badges::generation(),
        );
        let mut cached = self
            .source_rows
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some((cached_key, rows)) = cached.as_ref()
            && *cached_key == key
        {
            return rows.clone();
        }
        let rows: std::sync::Arc<Vec<_>> =
            streams.iter().map(source_row).collect::<Vec<_>>().into();
        *cached = Some((key, rows.clone()));
        rows
    }
}

pub(super) fn source_row(stream: &Value) -> fluxa_ui::PlayerSource {
    let lines = |key: &str| {
        stream
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let text = |key: &str| {
        stream
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut detail = text("description");
    if detail.is_empty() {
        detail = text("title");
    }
    let video_size = stream
        .pointer("/behaviorHints/videoSize")
        .and_then(Value::as_i64);
    let filename = stream
        .pointer("/behaviorHints/filename")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let raw = format!(
        "{}\n{}\n{}\n{filename}",
        lines("name"),
        text("description"),
        text("title")
    );
    let (badges, badges_top) = crate::stream_badges::badges_for(&raw, video_size);
    let mut name = text("name");
    if let Some((laid_name, laid_detail)) = crate::stream_badges::lay_out(&raw, video_size) {
        if !laid_name.is_empty() {
            name = laid_name;
        }
        if !laid_detail.is_empty() {
            detail = laid_detail;
        }
    }
    fluxa_ui::PlayerSource {
        addon: lines("addonName"),
        name: crate::stream_badges::format_text(&name),
        detail: crate::stream_badges::format_text(&detail),
        badges,
        badges_top,
        actions: !crate::stream_menu::entries(stream).is_empty(),
    }
}

pub(crate) fn cast_to(
    state: &mut RendererState,
    device: crate::cast::Device,
    url: &str,
    title: &str,
) {
    casting::cast_to(state, device, url, title);
}

pub(crate) fn download_stream(state: &mut RendererState, index: usize) {
    input::download_source(state, index);
}
