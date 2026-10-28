use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::Pending;

const FILE_LIMIT: u64 = 4 * 1024 * 1024;
const TAIL: u64 = 1024 * 1024;

struct Sink {
    file: File,
    dir: PathBuf,
    bytes: u64,
}

static SINK: Mutex<Option<Sink>> = Mutex::new(None);

fn create(dir: &Path) -> Option<File> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(dir.join("session.log"))
        .ok()
}

pub(crate) fn open(data_dir: &Path, backlog: &[String]) {
    let dir = data_dir.join("logs");
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let _ = fs::rename(dir.join("session.log"), dir.join("previous.log"));
    let Some(mut file) = create(&dir) else {
        return;
    };
    let mut bytes = 0;
    for line in backlog {
        if writeln!(file, "{line}").is_ok() {
            bytes += line.len() as u64 + 1;
        }
    }
    if let Ok(mut sink) = SINK.lock() {
        *sink = Some(Sink { file, dir, bytes });
    }
}

pub(crate) fn append(line: &str) {
    let Ok(mut guard) = SINK.lock() else {
        return;
    };
    let Some(sink) = guard.as_mut() else {
        return;
    };
    if sink.bytes > FILE_LIMIT {
        let _ = fs::rename(sink.dir.join("session.log"), sink.dir.join("session.1.log"));
        let Some(file) = create(&sink.dir) else {
            *guard = None;
            return;
        };
        sink.file = file;
        sink.bytes = 0;
    }
    if writeln!(sink.file, "{line}").is_ok() {
        sink.bytes += line.len() as u64 + 1;
    }
}

fn tail(path: &Path) -> Option<Vec<u8>> {
    let mut file = File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    file.seek(SeekFrom::Start(length.saturating_sub(TAIL)))
        .ok()?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).ok()?;
    Some(buffer)
}

pub(crate) fn attachments(fallback: &[String]) -> Vec<(String, Vec<u8>)> {
    let dir = SINK
        .lock()
        .ok()
        .and_then(|sink| sink.as_ref().map(|sink| sink.dir.clone()));
    let Some(dir) = dir else {
        return vec![("fluxa.log".to_owned(), fallback.join("\n").into_bytes())];
    };
    let mut files = Vec::new();
    let mut session = tail(&dir.join("session.1.log")).unwrap_or_default();
    session.extend(tail(&dir.join("session.log")).unwrap_or_default());
    files.push(("fluxa.log".to_owned(), session));
    if let Some(previous) = tail(&dir.join("previous.log")) {
        files.push(("fluxa-previous-session.log".to_owned(), previous));
    }
    files
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl Level {
    fn label(self) -> &'static str {
        match self {
            Level::Debug => "DEBUG",
            Level::Info => "INFO ",
            Level::Warn => "WARN ",
            Level::Error => "ERROR",
        }
    }
}

pub(crate) fn format(level: Level, target: &str, message: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() % 86_400;
    format!(
        "{:02}:{:02}:{:02}.{:03} {} {}: {}",
        secs / 3600,
        secs / 60 % 60,
        secs % 60,
        now.subsec_millis(),
        level.label(),
        target,
        message
    )
}

pub(crate) fn pending(
    level: sentry::Level,
    message: &str,
    tags: &[(&str, &str)],
    extras: &[(&str, &str)],
    log: Vec<String>,
) -> Pending {
    let owned = |pairs: &[(&str, &str)]| {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect()
    };
    Pending {
        level,
        message: message.to_owned(),
        tags: owned(tags),
        extras: owned(extras),
        files: attachments(&log),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_log_keeps_backlog_and_rotates_previous_run() {
        let dir = std::env::temp_dir().join(format!("fluxa-log-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        open(&dir, &["early".to_owned()]);
        append("first run");
        open(&dir, &["again".to_owned()]);
        append("second run");
        let files = attachments(&[]);
        let session = String::from_utf8(files[0].1.clone()).unwrap();
        let previous = String::from_utf8(files[1].1.clone()).unwrap();
        assert_eq!(session, "again\nsecond run\n");
        assert_eq!(previous, "early\nfirst run\n");
        let _ = fs::remove_dir_all(&dir);
    }
}
