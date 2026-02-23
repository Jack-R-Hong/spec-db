pub mod api;
pub mod assets;
pub mod state;
pub mod writeback;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::routing::{get, post};

use state::AppState;

pub use state::AppState as WebAppState;

pub struct WebConfig {
    pub host: String,
    pub port: u16,
    pub auth_token: Option<String>,
}

pub fn build_router(state: Arc<AppState>, web_config: &WebConfig) -> Router {
    let api_routes = Router::new()
        .route("/api/graph", get(api::get_graph))
        .route("/api/status", get(api::get_status))
        .route("/api/spec/{*id}", get(api::get_spec))
        .route("/api/sync", post(api::post_sync))
        .route("/api/writeback", post(api::post_writeback))
        .route("/api/writeback/undo", post(api::post_undo))
        .with_state(state.clone());

    let asset_routes = Router::new()
        .route("/", get(assets::serve_index))
        .route("/{*path}", get(assets::serve_static));

    let mut app = Router::new().merge(api_routes).merge(asset_routes);

    let requires_auth = web_config.host != "127.0.0.1" && web_config.auth_token.is_some();
    if requires_auth {
        let token = web_config.auth_token.clone().unwrap_or_default();
        app = app.layer(middleware::from_fn(move |req: Request, next: Next| {
            let expected = token.clone();
            async move { bearer_auth(req, next, &expected).await }
        }));
    }

    app
}

async fn bearer_auth(req: Request, next: Next, expected_token: &str) -> axum::response::Response {
    let auth_header = req.headers().get("authorization").and_then(|v| v.to_str().ok());

    match auth_header {
        Some(value) if value.strip_prefix("Bearer ").is_some_and(|t| t == expected_token) => {
            next.run(req).await
        }
        _ => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
    }
}

pub async fn start_web_server(
    app: Router,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    tracing::info!("Web UI available at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request as HttpRequest;
    use tower::ServiceExt;

    use super::*;

    fn test_state() -> (tempfile::TempDir, Arc<AppState>) {
        let dir = tempfile::tempdir().unwrap();
        let tantivy_dir = dir.path().join("tantivy");
        let fjall_dir = dir.path().join("fjall");
        std::fs::create_dir_all(&tantivy_dir).unwrap();
        std::fs::create_dir_all(&fjall_dir).unwrap();

        let state = AppState::new(tantivy_dir, fjall_dir, spec_db_core::SpecDbConfig::default());
        (dir, state)
    }

    #[test]
    fn web_config_defaults() {
        let config = WebConfig { host: "127.0.0.1".to_owned(), port: 3000, auth_token: None };
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert!(config.auth_token.is_none());
    }

    #[test]
    fn config_web_defaults_from_core() {
        let cfg = spec_db_core::SpecDbConfig::default();
        assert!(cfg.web.enabled);
        assert_eq!(cfg.web.host, "127.0.0.1");
        assert_eq!(cfg.web.port, 3000);
        assert!(cfg.web.auth_token.is_none());
    }

    #[test]
    fn config_web_from_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        std::fs::write(&path, "web:\n  enabled: true\n  host: \"0.0.0.0\"\n  port: 4000\n  auth_token: \"secret123\"\n").unwrap();
        let cfg = spec_db_core::load_config(&path).unwrap();
        assert!(cfg.web.enabled);
        assert_eq!(cfg.web.host, "0.0.0.0");
        assert_eq!(cfg.web.port, 4000);
        assert_eq!(cfg.web.auth_token.as_deref(), Some("secret123"));
    }

    #[tokio::test]
    async fn localhost_no_auth_required() {
        let (_dir, state) = test_state();
        let config = WebConfig {
            host: "127.0.0.1".to_owned(),
            port: 3000,
            auth_token: Some("secret".to_owned()),
        };
        let app = build_router(state, &config);

        let resp = app
            .oneshot(HttpRequest::builder().uri("/api/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn non_localhost_requires_auth() {
        let (_dir, state) = test_state();
        let config = WebConfig {
            host: "0.0.0.0".to_owned(),
            port: 3000,
            auth_token: Some("secret".to_owned()),
        };
        let app = build_router(state, &config);

        let resp = app
            .oneshot(HttpRequest::builder().uri("/api/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn non_localhost_valid_token_passes() {
        let (_dir, state) = test_state();
        let config = WebConfig {
            host: "0.0.0.0".to_owned(),
            port: 3000,
            auth_token: Some("secret".to_owned()),
        };
        let app = build_router(state, &config);

        let resp = app
            .oneshot(
                HttpRequest::builder()
                    .uri("/api/status")
                    .header("authorization", "Bearer secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_status_returns_valid_json() {
        let (_dir, state) = test_state();
        let config = WebConfig { host: "127.0.0.1".to_owned(), port: 3000, auth_token: None };
        let app = build_router(state, &config);

        let resp = app
            .oneshot(HttpRequest::builder().uri("/api/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("spec_count").is_some());
        assert!(json.get("consistency").is_some());
    }

    #[tokio::test]
    async fn api_graph_returns_nodes_and_edges() {
        let (_dir, state) = test_state();
        let config = WebConfig { host: "127.0.0.1".to_owned(), port: 3000, auth_token: None };
        let app = build_router(state, &config);

        let resp = app
            .oneshot(HttpRequest::builder().uri("/api/graph").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("nodes").and_then(|v| v.as_array()).is_some());
        assert!(json.get("edges").and_then(|v| v.as_array()).is_some());
    }

    #[tokio::test]
    async fn serves_index_html() {
        let (_dir, state) = test_state();
        let config = WebConfig { host: "127.0.0.1".to_owned(), port: 3000, auth_token: None };
        let app = build_router(state, &config);

        let resp = app
            .oneshot(HttpRequest::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("Lattice Web UI"));
    }
}
