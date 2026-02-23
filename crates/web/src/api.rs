use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::{Value, json};
use spec_db_causal::{CausalEngine, FjallStore};
use spec_db_core::SpecId;
use spec_db_search::SearchIndex;

use crate::state::{AppState, UndoState};
use crate::writeback::{WriteBackOp, WriteBackPipeline};

pub async fn get_graph(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _span = tracing::info_span!("spec_db.web.api.graph").entered();

    let store = match FjallStore::open(&state.fjall_dir) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "GraphError", &e.to_string());
        }
    };

    let graph = match CausalEngine::from_store(store.clone()) {
        Ok(g) => g,
        Err(e) => {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "GraphError", &e.to_string());
        }
    };

    let search = SearchIndex::open_or_create(&state.tantivy_dir).ok();

    let nodes: Vec<Value> = match store.iter_nodes() {
        Ok(all) => all
            .into_iter()
            .map(|n| {
                let tags: Vec<String> = search
                    .as_ref()
                    .and_then(|s| s.get_spec(&n.id).ok().flatten())
                    .map(|doc| doc.tags)
                    .unwrap_or_default();
                json!({"id": n.id.to_string(), "title": n.title, "version": n.version, "tags": tags})
            })
            .collect(),
        Err(e) => {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "GraphError", &e.to_string());
        }
    };

    let edges: Vec<Value> = match graph.all_edges() {
        Ok(all) => all
            .into_iter()
            .map(|e| {
                json!({
                    "source": e.source.to_string(),
                    "target": e.target.to_string(),
                    "edge_type": e.edge_type.to_string(),
                    "trust": e.trust.value(),
                    "origin": e.origin.to_string(),
                })
            })
            .collect(),
        Err(e) => {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "GraphError", &e.to_string());
        }
    };

    (StatusCode::OK, Json(json!({"nodes": nodes, "edges": edges}))).into_response()
}

pub async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _span = tracing::info_span!("spec_db.web.api.status").entered();

    let store = match FjallStore::open(&state.fjall_dir) {
        Ok(s) => s,
        Err(e) => {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "GraphError", &e.to_string());
        }
    };

    let node_count = store.iter_nodes().map(|n| n.len()).unwrap_or(0);
    let last_sha = store.last_sync_sha().ok().flatten().unwrap_or_default();
    let doc_count = store.doc_count().ok().flatten().unwrap_or(node_count);
    let drift_detected = doc_count != node_count;

    (
        StatusCode::OK,
        Json(json!({
            "spec_count": node_count,
            "last_sync_sha": last_sha,
            "consistency": if !drift_detected { "consistent" } else { "drifted" },
            "drift_detected": drift_detected,
        })),
    )
        .into_response()
}

pub async fn get_spec(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let _span = tracing::info_span!("spec_db.web.api.spec", spec_id = %id).entered();

    let spec_id = match SpecId::try_new(&id) {
        Ok(s) => s,
        Err(e) => {
            return api_error(StatusCode::BAD_REQUEST, "InvalidId", &e.to_string());
        }
    };

    let search = match SearchIndex::open_or_create(&state.tantivy_dir) {
        Ok(s) => s,
        Err(e) => {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "SearchError", &e.to_string());
        }
    };

    let doc = match search.get_spec(&spec_id) {
        Ok(Some(d)) => d,
        Ok(None) => {
            return api_error(StatusCode::NOT_FOUND, "NotFound", &format!("Spec '{id}' not found"));
        }
        Err(e) => {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "SearchError", &e.to_string());
        }
    };

    let store = match FjallStore::open(&state.fjall_dir) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "GraphError", &e.to_string());
        }
    };

    let graph = match CausalEngine::from_store(store) {
        Ok(g) => g,
        Err(e) => {
            return api_error(StatusCode::INTERNAL_SERVER_ERROR, "GraphError", &e.to_string());
        }
    };

    let all_edges = graph.all_edges().unwrap_or_default();
    let inbound: Vec<Value> = all_edges
        .iter()
        .filter(|e| e.target == spec_id)
        .map(|e| {
            json!({
                "source": e.source.to_string(),
                "edge_type": e.edge_type.to_string(),
                "trust": e.trust.value(),
                "origin": e.origin.to_string(),
            })
        })
        .collect();

    let outbound: Vec<Value> = all_edges
        .iter()
        .filter(|e| e.source == spec_id)
        .map(|e| {
            json!({
                "target": e.target.to_string(),
                "edge_type": e.edge_type.to_string(),
                "trust": e.trust.value(),
                "origin": e.origin.to_string(),
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(json!({
            "id": doc.id.to_string(),
            "title": doc.title,
            "version": doc.version,
            "tags": doc.tags,
            "owner": doc.owner,
            "created": doc.created,
            "depends_on": doc.depends_on.iter().map(|d| d.to_string()).collect::<Vec<_>>(),
            "body": doc.body,
            "inbound_edges": inbound,
            "outbound_edges": outbound,
        })),
    )
        .into_response()
}

pub async fn post_sync(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> axum::response::Response {
    let mode = params.get("mode").map(|s| s.as_str()).unwrap_or("incremental");

    let tantivy_dir = state.tantivy_dir.clone();
    let fjall_dir = state.fjall_dir.clone();
    let config = state.config.clone();
    let full = mode == "full";

    let result = tokio::task::spawn_blocking(move || {
        let _span = tracing::info_span!("spec_db.web.api.sync").entered();
        let repo_path = std::path::Path::new(&config.specs_dir)
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();

        let store_paths = spec_db_ingest::StorePaths {
            tantivy_dir: tantivy_dir.clone(),
            fjall_dir: fjall_dir.clone(),
        };

        let sync = spec_db_ingest::GitSync::new(repo_path, config.specs_dir.clone(), store_paths);

        let report = if full {
            sync.full_rebuild().map_err(|e| e.to_string())?
        } else {
            sync.incremental_sync().map_err(|e| e.to_string())?
        };

        Ok::<_, String>(json!({
            "status": "ok",
            "mode": if full { "full" } else { "incremental" },
            "spec_count": report.specs_ingested,
            "last_sync_sha": report.head_sha,
        }))
    })
    .await;

    match result {
        Ok(Ok(val)) => (StatusCode::OK, Json(val)).into_response(),
        Ok(Err(msg)) => api_error(StatusCode::INTERNAL_SERVER_ERROR, "SyncError", &msg),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, "SyncError", &e.to_string()),
    }
}

pub async fn post_writeback(
    State(state): State<Arc<AppState>>,
    Json(op): Json<WriteBackOp>,
) -> axum::response::Response {
    let config = state.config.clone();
    let tantivy_dir = state.tantivy_dir.clone();
    let fjall_dir = state.fjall_dir.clone();

    let result = tokio::task::spawn_blocking({
        let state = state.clone();
        move || {
            let _lock = state.write_lock.lock().map_err(|_| "write lock poisoned".to_string())?;
            let _span = tracing::info_span!("spec_db.web.api.writeback").entered();

            let repo_path = std::path::Path::new(&config.specs_dir)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();

            let pipeline = WriteBackPipeline::new(repo_path.clone(), config.specs_dir.clone());
            let commit_sha = pipeline.apply(&op).map_err(|e| e.to_string())?;

            let store_paths = spec_db_ingest::StorePaths { tantivy_dir, fjall_dir };
            let sync =
                spec_db_ingest::GitSync::new(repo_path, config.specs_dir.clone(), store_paths);
            let _ = sync.incremental_sync();

            if let Ok(mut undo) = state.undo_state.lock() {
                *undo =
                    Some(UndoState { commit_sha: commit_sha.clone(), created_at: Instant::now() });
            }

            Ok::<_, String>(commit_sha)
        }
    })
    .await;

    match result {
        Ok(Ok(sha)) => {
            (StatusCode::OK, Json(json!({"status": "ok", "commit_sha": sha}))).into_response()
        }
        Ok(Err(msg)) => api_error(StatusCode::INTERNAL_SERVER_ERROR, "WriteBackError", &msg),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, "WriteBackError", &e.to_string()),
    }
}

pub async fn post_undo(State(state): State<Arc<AppState>>) -> axum::response::Response {
    let config = state.config.clone();
    let tantivy_dir = state.tantivy_dir.clone();
    let fjall_dir = state.fjall_dir.clone();

    let commit_sha = {
        let undo = state.undo_state.lock().ok();
        match undo.as_ref().and_then(|u| u.as_ref()) {
            Some(u) if u.created_at.elapsed().as_secs() < 5 => u.commit_sha.clone(),
            Some(_) => {
                return api_error(StatusCode::GONE, "Expired", "undo window has expired");
            }
            None => {
                return api_error(StatusCode::NOT_FOUND, "NoUndo", "no undo state available");
            }
        }
    };

    let result = tokio::task::spawn_blocking({
        let state = state.clone();
        move || {
            let _lock = state.write_lock.lock().map_err(|_| "write lock poisoned".to_string())?;
            let _span = tracing::info_span!("spec_db.web.api.undo").entered();

            let repo_path = std::path::Path::new(&config.specs_dir)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();

            let pipeline = WriteBackPipeline::new(repo_path.clone(), config.specs_dir.clone());
            pipeline.undo(&commit_sha).map_err(|e| e.to_string())?;

            let store_paths = spec_db_ingest::StorePaths { tantivy_dir, fjall_dir };
            let sync =
                spec_db_ingest::GitSync::new(repo_path, config.specs_dir.clone(), store_paths);
            let _ = sync.incremental_sync();

            if let Ok(mut undo) = state.undo_state.lock() {
                *undo = None;
            }

            Ok::<_, String>(())
        }
    })
    .await;

    match result {
        Ok(Ok(())) => (StatusCode::OK, Json(json!({"status": "ok"}))).into_response(),
        Ok(Err(msg)) => api_error(StatusCode::INTERNAL_SERVER_ERROR, "UndoError", &msg),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, "UndoError", &e.to_string()),
    }
}

fn api_error(status: StatusCode, error_type: &str, message: &str) -> axum::response::Response {
    (
        status,
        Json(json!({
            "error_type": error_type,
            "message": message,
            "context": Value::Null,
        })),
    )
        .into_response()
}
