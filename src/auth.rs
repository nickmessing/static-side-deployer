use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

use crate::config::AppConfig;

pub struct Authenticated;

impl FromRequestParts<Arc<AppConfig>> for Authenticated {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppConfig>,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if !constant_time_eq(token.as_bytes(), state.secret_token.as_bytes()) {
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(Authenticated)
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}
