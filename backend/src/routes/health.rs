use axum::{extract::State, Json};
use serde::Serialize;

use crate::AppState;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) struct HealthResponse {
    status: &'static str,
    build_sha: String,
}

pub(crate) fn payload(build_sha: &str) -> HealthResponse {
    HealthResponse {
        status: "ok",
        build_sha: build_sha.to_owned(),
    }
}

pub(crate) async fn handler(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(payload(&state.build_sha))
}

#[cfg(test)]
mod tests {
    use super::payload;

    #[test]
    fn reports_the_build_identifier() {
        let response = payload("abc123");

        assert_eq!(response.status, "ok");
        assert_eq!(response.build_sha, "abc123");
    }
}
