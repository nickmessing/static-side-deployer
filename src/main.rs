use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::post,
    Router,
};

mod archive;
mod auth;
mod caddy;
mod config;
mod deploy;
mod manifest;

use auth::Authenticated;
use config::AppConfig;

const MAX_UPLOAD_SIZE: usize = 200 * 1024 * 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Arc::new(AppConfig::from_env()?);

    let app = Router::new()
        .route("/push", post(push_handler))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_SIZE))
        .with_state(config.clone());

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn push_handler(
    State(config): State<Arc<AppConfig>>,
    _auth: Authenticated,
    body: Bytes,
) -> StatusCode {
    if body.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    tracing::info!("received {} bytes", body.len());

    let contents = match archive::extract_archive(&body) {
        Some(c) => c,
        None => {
            tracing::warn!("invalid archive");
            return StatusCode::BAD_REQUEST;
        }
    };

    let full_domain = contents.manifest.full_domain();
    tracing::info!(
        "deploying {} ({} files) to {full_domain}",
        contents.manifest.domain,
        contents.files.len()
    );

    let caddy_changed = match caddy::upsert_config(&config.caddy_directory, &full_domain).await {
        Ok(changed) => changed,
        Err(e) => {
            tracing::error!("failed to upsert caddy config: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    if caddy_changed {
        if let Err(e) = caddy::reload_caddy().await {
            tracing::error!("failed to reload caddy: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    if let Err(e) = deploy::deploy_files(&config.caddy_directory, &full_domain, contents.files).await {
        tracing::error!("failed to deploy files: {e}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    tracing::info!("deployment complete for {full_domain}");
    StatusCode::OK
}
