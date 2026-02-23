use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::{Value, json};
use spec_db_causal::{CausalEngine, FjallStore};

use crate::state::AppState;

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

    let nodes: Vec<Value> = match store.iter_nodes() {
        Ok(all) => all
            .into_iter()
            .map(|n| json!({"id": n.id.to_string(), "title": n.title, "version": n.version}))
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
    let consistent = doc_count == node_count;

    (
        StatusCode::OK,
        Json(json!({
            "spec_count": node_count,
            "last_sync_sha": last_sha,
            "consistency": if consistent { "consistent" } else { "drifted" },
        })),
    )
        .into_response()
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
