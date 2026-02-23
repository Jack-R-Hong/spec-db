use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../../web-ui/dist/"]
struct StaticAssets;

pub async fn serve_static(axum::extract::Path(path): axum::extract::Path<String>) -> Response {
    serve_asset(&path)
}

pub async fn serve_index() -> Response {
    serve_asset("index.html")
}

fn serve_asset(path: &str) -> Response {
    #[cfg(debug_assertions)]
    {
        let fs_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web-ui/dist").join(path);
        if let Ok(data) = std::fs::read(&fs_path) {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            return (StatusCode::OK, [(header::CONTENT_TYPE, mime.as_ref().to_owned())], data)
                .into_response();
        }
    }

    #[cfg(not(debug_assertions))]
    {
        if let Some(file) = StaticAssets::get(path) {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime.as_ref().to_owned())],
                file.data.to_vec(),
            )
                .into_response();
        }
    }

    #[cfg(debug_assertions)]
    if let Some(file) = StaticAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, mime.as_ref().to_owned())],
            file.data.to_vec(),
        )
            .into_response();
    }

    if path != "index.html" {
        return serve_asset("index.html");
    }

    (StatusCode::NOT_FOUND, "not found").into_response()
}
