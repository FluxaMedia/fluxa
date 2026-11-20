use super::*;

pub(super) const WATCH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(400);

pub(super) static WATCH: Mutex<Option<(web_time::Instant, std::time::SystemTime, u64)>> =
    Mutex::new(None);

pub(super) fn dev_folder() -> Option<String> {
    std::env::var("FLUXA_SKIN")
        .ok()
        .filter(|path| !path.is_empty())
}

pub(super) fn folder_stamp(dir: &Path) -> (std::time::SystemTime, u64) {
    let mut newest = std::time::UNIX_EPOCH;
    let mut count = 0;
    let mut stack = vec![dir.to_owned()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() {
                stack.push(entry.path());
                continue;
            }
            count += 1;
            if let Ok(modified) = meta.modified() {
                newest = newest.max(modified);
            }
        }
    }
    (newest, count)
}

pub(crate) fn watching() -> bool {
    dev_folder().is_some_and(|path| Path::new(&path).join("manifest.json").is_file())
}

pub(crate) fn next_watch() -> Option<web_time::Instant> {
    if !watching() {
        return None;
    }
    let last = WATCH.lock().ok()?.map(|(at, ..)| at);
    Some(last.map_or_else(web_time::Instant::now, |at| at + WATCH_INTERVAL))
}

pub(crate) fn poll_dev() {
    let Some(path) = dev_folder().filter(|_| watching()) else {
        return;
    };
    let now = web_time::Instant::now();
    let Ok(mut watch) = WATCH.lock() else {
        return;
    };
    if watch.is_some_and(|(at, ..)| now < at + WATCH_INTERVAL) {
        return;
    }
    let (newest, count) = folder_stamp(Path::new(&path));
    let changed = watch.is_some_and(|(_, seen, seen_count)| (seen, seen_count) != (newest, count));
    *watch = Some((now, newest, count));
    drop(watch);
    if !changed {
        return;
    }
    let Some(data_dir) = MANAGER
        .lock()
        .ok()
        .and_then(|manager| manager.data_dir.clone())
    else {
        return;
    };
    load(&data_dir);
    if let Ok(mut cache) = CACHE.lock() {
        cache.clear();
    }
    if let Ok(mut data) = DATA.lock() {
        data.clear();
    }
    touch();
    crate::host_log(format!("skin reloaded from {path}"));
}
