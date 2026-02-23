#[derive(thiserror::Error, Debug)]
pub enum SpecDbError {
    #[error("search error: {0}")]
    SearchError(String),

    #[error("graph error: {0}")]
    GraphError(String),

    #[error("sync error: {0}")]
    SyncError(String),

    #[error("ingest error: {0}")]
    IngestError(String),

    #[error("consistency error: {0}")]
    ConsistencyError(String),

    #[error("config error: {0}")]
    ConfigError(String),
}
