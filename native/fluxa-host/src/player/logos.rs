use super::*;

pub(super) fn seek_logo(player: &mut PlayerSession, session: &SessionHandle, snapshot: &Value) {
    if player.snapshot_logo.is_some() {
        return;
    }
    let id = player
        .meta
        .get("id")
        .and_then(Value::as_str)
        .map(|id| id.split(':').next().unwrap_or_default().to_owned())
        .filter(|id| !id.is_empty());
    let Some(id) = id else {
        return;
    };
    if !player.logo_requested {
        if player.model_logo_from_meta().is_some() {
            return;
        }
        player.logo_requested = true;
        let content_type = player
            .meta
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("movie");
        let _ = session.dispatch(json!({
            "type": "metaDetailRequested",
            "contentType": content_type,
            "id": id,
            "language": player.language,
            "profile": snapshot.pointer("/profile/active"),
        }));
        return;
    }
    let found = snapshot.pointer("/lookup/metaDetail").and_then(|detail| {
        let meta = detail.get("meta").unwrap_or(detail);
        let same = meta
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|found| found.split(':').next() == Some(id.as_str()));
        same.then(|| meta.get("logo").and_then(Value::as_str))
            .flatten()
            .filter(|url| !url.is_empty())
            .map(str::to_owned)
    });
    if found.is_some() {
        player.snapshot_logo = found;
    }
}

pub(super) fn detail_logo(snapshot: &Value, meta: &Value) -> Option<String> {
    let detail = snapshot.get("detail")?;
    let id = |value: &Value| {
        ["id", "itemId"]
            .iter()
            .find_map(|key| value.get(*key).and_then(Value::as_str))
            .map(|id| id.split(':').next().unwrap_or_default().to_owned())
    };
    let played = id(meta)?;
    let same = id(detail).is_some_and(|id| id == played)
        || detail
            .get("meta")
            .and_then(id)
            .is_some_and(|id| id == played);
    if !same {
        return None;
    }
    detail
        .pointer("/fanartArtwork/hdLogo")
        .or_else(|| detail.pointer("/meta/logo"))
        .and_then(Value::as_str)
        .filter(|url| !url.is_empty())
        .map(str::to_owned)
}
