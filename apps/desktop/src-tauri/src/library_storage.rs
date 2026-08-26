#[path = "storage_kv.rs"]
mod kv_storage;
#[path = "library_progress_storage.rs"]
mod progress_storage;
#[path = "library_history_storage.rs"]
mod history_storage;
#[path = "library_collections_storage.rs"]
mod collections_storage;

pub use collections_storage::{
    library_status_list, library_status_set, library_watched_list, library_watched_set,
};
pub use history_storage::{
    library_continue_watching_delete, library_continue_watching_list,
    library_continue_watching_upsert, library_last_watched_delete, library_last_watched_list,
    library_last_watched_upsert,
};
pub use kv_storage::{read_pref_field, read_pref_flag, storage_delete, storage_read, storage_write};
pub use progress_storage::{
    library_progress_delete, library_progress_list, library_progress_read, library_progress_upsert,
    library_progress_upsert_many,
};
#[cfg(target_os = "windows")]
pub(crate) use kv_storage::read_pref_bool;
