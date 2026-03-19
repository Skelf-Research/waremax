//! Static file serving from embedded assets

use axum::{
    body::Body,
    http::{header, Response, StatusCode, Uri},
    response::IntoResponse,
};

use crate::embed::{Assets, FALLBACK_HTML};

/// Serve static files from embedded assets
pub async fn serve_static(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // Try to serve the exact file
    if let Some(file) = Assets::get_file(path) {
        let mime = Assets::mime_type(path);
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime)
            .header(header::CACHE_CONTROL, "public, max-age=3600")
            .body(Body::from(file.data.into_owned()))
            .unwrap();
    }

    // For SPA routing: serve index.html for non-API routes
    if !path.starts_with("api/") && !path.starts_with("ws/") {
        if let Some(file) = Assets::get_file("index.html") {
            return Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Body::from(file.data.into_owned()))
                .unwrap();
        }

        // Fallback HTML when frontend is not built
        return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(FALLBACK_HTML))
            .unwrap();
    }

    // 404 for other paths
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("Not Found"))
        .unwrap()
}

/// Serve index.html (for root path)
pub async fn serve_index() -> impl IntoResponse {
    if let Some(file) = Assets::get_file("index.html") {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(file.data.into_owned()))
            .unwrap()
    } else {
        // Fallback HTML when frontend is not built
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(FALLBACK_HTML))
            .unwrap()
    }
}
