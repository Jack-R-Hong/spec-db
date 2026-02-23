use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use spec_db_core::SpecDbConfig;

pub struct UndoState {
    pub commit_sha: String,
    pub created_at: Instant,
}

pub struct AppState {
    pub tantivy_dir: PathBuf,
    pub fjall_dir: PathBuf,
    pub config: SpecDbConfig,
    pub write_lock: Mutex<()>,
    pub undo_state: Mutex<Option<UndoState>>,
}

impl AppState {
    pub fn new(tantivy_dir: PathBuf, fjall_dir: PathBuf, config: SpecDbConfig) -> Arc<Self> {
        Arc::new(Self {
            tantivy_dir,
            fjall_dir,
            config,
            write_lock: Mutex::new(()),
            undo_state: Mutex::new(None),
        })
    }
}
