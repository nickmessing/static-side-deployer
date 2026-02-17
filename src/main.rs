use axum::{
    Router,
    body::Bytes,
    extract::DefaultBodyLimit,
    http::{HeaderMap, StatusCode},
    routing::post,
};

const MAX_UPLOAD_SIZE: usize = 200 * 1024 * 1024; // 200 MB

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/push", post(push_handler))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_SIZE));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9092").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;

    Ok(())
}

async fn push_handler(headers: HeaderMap, body: Bytes) -> StatusCode {
    let auth = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    if auth != Some("test1") {
        return StatusCode::UNAUTHORIZED;
    }

    if body.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    println!("received {} bytes", body.len());

    // TODO: handle tgz archive

    StatusCode::OK
}
