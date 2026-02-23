use std::path::PathBuf;
use std::sync::Arc;

use spec_db_core::SpecDbConfig;

pub struct AppState {
    pub tantivy_dir: PathBuf,
    pub fjall_dir: PathBuf,
    pub config: SpecDbConfig,
}

impl AppState {
    pub fn new(tantivy_dir: PathBuf, fjall_dir: PathBuf, config: SpecDbConfig) -> Arc<Self> {
        Arc::new(Self { tantivy_dir, fjall_dir, config })
    }
}
