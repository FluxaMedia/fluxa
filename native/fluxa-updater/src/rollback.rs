use crate::{CURRENT_VERSION, install};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_STARTS: u32 = 3;

#[derive(Serialize, Deserialize)]
struct Pending {
    to: String,
    starts: u32,
}

fn marker(data_dir: &Path) -> PathBuf {
    data_dir.join("rollback.json")
}

fn read(data_dir: &Path) -> Option<Pending> {
    serde_json::from_slice(&fs::read(marker(data_dir)).ok()?).ok()
}

fn write(data_dir: &Path, pending: &Pending) {
    if let Ok(bytes) = serde_json::to_vec(pending) {
        let _ = fs::write(marker(data_dir), bytes);
    }
}

pub(crate) fn skipped(data_dir: &Path) -> Option<String> {
    fs::read_to_string(data_dir.join("skipped-update"))
        .ok()
        .map(|version| version.trim().to_owned())
}

pub(crate) fn before_apply(data_dir: &Path, version: &str) {
    if install::keeps_backup() {
        write(
            data_dir,
            &Pending {
                to: version.to_owned(),
                starts: 0,
            },
        );
    }
}

pub(crate) fn abandon(data_dir: &Path) {
    let _ = fs::remove_file(marker(data_dir));
}

pub(crate) fn on_start(data_dir: &Path) {
    let Some(pending) = read(data_dir) else {
        install::remove_leftovers();
        return;
    };
    if pending.to != CURRENT_VERSION {
        abandon(data_dir);
        install::remove_leftovers();
        return;
    }
    let starts = pending.starts + 1;
    if starts < MAX_STARTS {
        write(
            data_dir,
            &Pending {
                to: pending.to,
                starts,
            },
        );
        return;
    }
    abandon(data_dir);
    if install::restore().is_ok() {
        let _ = fs::write(data_dir.join("skipped-update"), &pending.to);
        std::process::exit(0);
    }
}

pub(crate) fn confirm(data_dir: &Path) {
    if read(data_dir).is_some_and(|pending| pending.to == CURRENT_VERSION) {
        abandon(data_dir);
        install::remove_leftovers();
    }
}
