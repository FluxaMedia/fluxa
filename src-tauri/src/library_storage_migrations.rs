use super::{decrypt_or_legacy, encrypt, sanitize_key, LEGACY_MIGRATION_KEY};
use fluxa_core::FluxaCore;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MigrationEntry {
    key: String,
    value: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LibraryItemMigrationEntry {
    media_id: String,
    status: String,
    value: Value,
}

pub(super) fn ensure_progress_migrated(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<(), String> {
    let migrated: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_migrations WHERE profile_key = ?1",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.is_some() {
        return Ok(());
    }

    let document: Option<Vec<u8>> = connection
        .query_row(
            "SELECT value FROM kv_store WHERE key = ?1",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let document = document
        .as_deref()
        .and_then(|bytes| decrypt_or_legacy(dir, bytes))
        .unwrap_or_else(|| "{}".to_string());
    let entries: Vec<MigrationEntry> =
        serde_json::from_str(&FluxaCore::library_progress_entries_json(&document))
            .map_err(|e| e.to_string())?;
    let entries = entries
        .into_iter()
        .map(|entry| {
            let value = serde_json::to_vec(&entry.value).map_err(|e| e.to_string())?;
            Ok((entry.key, encrypt(dir, &value)?))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    for (media_id, value) in entries {
        tx.execute(
            "INSERT INTO library_progress (profile_key, media_id, value)
             VALUES (?1, ?2, ?3)",
            params![profile_key, media_id, value],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO library_migrations (profile_key) VALUES (?1)",
        [profile_key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

fn library_document(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<String, String> {
    let document: Option<Vec<u8>> = connection
        .query_row(
            "SELECT value FROM kv_store WHERE key = ?1",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(document
        .as_deref()
        .and_then(|bytes| decrypt_or_legacy(dir, bytes))
        .unwrap_or_else(|| "{}".to_string()))
}

pub(super) fn ensure_items_migrated(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<(), String> {
    let migrated: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_domain_migrations WHERE profile_key = ?1 AND domain = 'items'",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.is_some() {
        return Ok(());
    }
    let document = library_document(connection, dir, profile_key)?;
    let entries: Vec<LibraryItemMigrationEntry> =
        serde_json::from_str(&FluxaCore::library_items_json(&document))
            .map_err(|e| e.to_string())?;
    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    for entry in entries {
        let value = encrypt(
            dir,
            &serde_json::to_vec(&entry.value).map_err(|e| e.to_string())?,
        )?;
        tx.execute("INSERT INTO library_items (profile_key, media_id, status, value) VALUES (?1, ?2, ?3, ?4)", params![profile_key, entry.media_id, entry.status, value]).map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO library_domain_migrations (profile_key, domain) VALUES (?1, 'items')",
        [profile_key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

pub(super) fn ensure_watched_migrated(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<(), String> {
    let migrated: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_domain_migrations WHERE profile_key = ?1 AND domain = 'watched'",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.is_some() {
        return Ok(());
    }
    let document = library_document(connection, dir, profile_key)?;
    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    let video_ids: Vec<String> =
        serde_json::from_str(&FluxaCore::library_watched_video_ids_json(&document))
            .map_err(|e| e.to_string())?;
    for video_id in video_ids {
        tx.execute(
            "INSERT INTO watched_videos (profile_key, video_id) VALUES (?1, ?2)",
            params![profile_key, video_id],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO library_domain_migrations (profile_key, domain) VALUES (?1, 'watched')",
        [profile_key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

pub(super) fn ensure_last_watched_migrated(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<(), String> {
    let migrated: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_domain_migrations WHERE profile_key = ?1 AND domain = 'last_watched'",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.is_some() {
        return Ok(());
    }
    let document = library_document(connection, dir, profile_key)?;
    let entries: Vec<MigrationEntry> =
        serde_json::from_str(&FluxaCore::library_last_watched_entries_json(&document))
            .map_err(|e| e.to_string())?;
    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    for entry in entries {
        let value = encrypt(
            dir,
            &serde_json::to_vec(&entry.value).map_err(|e| e.to_string())?,
        )?;
        tx.execute(
            "INSERT INTO library_last_watched (profile_key, series_id, value) VALUES (?1, ?2, ?3)",
            params![profile_key, entry.key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO library_domain_migrations (profile_key, domain) VALUES (?1, 'last_watched')",
        [profile_key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

pub(super) fn ensure_continue_watching_migrated(
    connection: &Connection,
    dir: &Path,
    profile_key: &str,
) -> Result<(), String> {
    let migrated: Option<i64> = connection
        .query_row(
            "SELECT 1 FROM library_domain_migrations WHERE profile_key = ?1 AND domain = 'continue_watching'",
            [profile_key],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.is_some() {
        return Ok(());
    }
    let document = library_document(connection, dir, profile_key)?;
    let entries: Vec<MigrationEntry> = serde_json::from_str(
        &FluxaCore::library_continue_watching_entries_json(&document),
    )
    .map_err(|e| e.to_string())?;
    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    for entry in entries {
        let value = encrypt(
            dir,
            &serde_json::to_vec(&entry.value).map_err(|e| e.to_string())?,
        )?;
        tx.execute(
            "INSERT INTO library_continue_watching (profile_key, media_id, value) VALUES (?1, ?2, ?3)",
            params![profile_key, entry.key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO library_domain_migrations (profile_key, domain) VALUES (?1, 'continue_watching')",
        [profile_key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

pub(super) fn migrate_legacy_json_files(connection: &Connection, dir: &Path) -> Result<(), String> {
    let migrated: Option<String> = connection
        .query_row(
            "SELECT value FROM storage_meta WHERE key = ?1",
            [LEGACY_MIGRATION_KEY],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if migrated.as_deref() == Some("complete") {
        return Ok(());
    }

    let mut legacy_files = Vec::<(String, Vec<u8>, PathBuf)>::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !entry.file_type().map_err(|e| e.to_string())?.is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("json")
        {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };
        legacy_files.push((
            stem.to_owned(),
            fs::read(&path).map_err(|e| e.to_string())?,
            path,
        ));
    }

    let tx = connection
        .unchecked_transaction()
        .map_err(|e| e.to_string())?;
    for (key, bytes, _) in &legacy_files {
        tx.execute(
            "INSERT INTO kv_store (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO NOTHING",
            params![key, bytes],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.execute(
        "INSERT INTO storage_meta (key, value) VALUES (?1, 'complete')
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [LEGACY_MIGRATION_KEY],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;

    for (_, _, path) in legacy_files {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let backup = path.with_file_name(format!("{file_name}.migrated-backup"));
        if !backup.exists() {
            let _ = fs::rename(&path, backup);
        }
    }
    Ok(())
}

pub(super) fn read_legacy_file(dir: &Path, key: &str) -> Option<String> {
    let path = dir.join(format!("{}.json", sanitize_key(key)));
    let bytes = fs::read(path).ok()?;
    decrypt_or_legacy(dir, &bytes)
}
