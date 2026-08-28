use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

use crate::AppState;

const DEMO_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const TOKEN_BYTES: usize = 32;
const CONSENT_WORDING: &str = "Email me once about this booking if I leave before confirming.";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DemoEnvelope {
    workspace_token: String,
    workspace: DemoWorkspace,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DemoWorkspace {
    id: String,
    expires_at: String,
    practice: Practice,
    service: Service,
    attempts: Vec<Attempt>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Practice {
    name: String,
    timezone: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Service {
    name: String,
    duration_minutes: i64,
    deposit_cents: i64,
    currency: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Attempt {
    id: String,
    client_name: String,
    scheduled_for: String,
    state: String,
    reason: String,
    consent: Consent,
    outcome: Option<String>,
    receipts: Vec<Receipt>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Consent {
    email: bool,
    wording: Option<String>,
    recorded_at: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    channel: String,
    status: String,
    detail: String,
    occurred_at: String,
    simulated: bool,
}

#[derive(Debug, FromRow)]
struct WorkspaceRow {
    id: String,
    practice_name: String,
    practice_timezone: String,
    service_name: String,
    service_duration_minutes: i64,
    deposit_cents: i64,
    currency: String,
    expires_at: i64,
}

#[derive(Debug, FromRow)]
struct AttemptRow {
    id: String,
    client_name: String,
    scheduled_for: i64,
    state: String,
    reason: String,
    email_consent: bool,
    consent_wording: Option<String>,
    consent_recorded_at: Option<i64>,
    outcome: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    error: &'static str,
    message: String,
}

#[derive(Debug)]
pub(crate) struct ApiError {
    status: StatusCode,
    error: &'static str,
    message: String,
}

impl ApiError {
    fn bad_request(error: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error,
            message: message.into(),
        }
    }

    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            error: "demo_not_found",
            message: "This demo expired or is not available. Start a fresh demo.".to_owned(),
        }
    }

    fn conflict(error: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            error,
            message: message.into(),
        }
    }

    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "demo_unavailable",
            message: "The sample workspace could not be loaded. Try again.".to_owned(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiErrorBody {
                error: self.error,
                message: self.message,
            }),
        )
            .into_response()
    }
}

pub(crate) async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<DemoEnvelope>), ApiError> {
    let idempotency_key = idempotency_key(&headers)?;
    purge_expired(&state.pool).await?;
    reject_reused_key(&state.pool, &idempotency_key).await?;
    let envelope = seed_workspace(&state.pool, idempotency_key).await?;
    Ok((StatusCode::CREATED, Json(envelope)))
}

pub(crate) async fn show(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DemoEnvelope>, ApiError> {
    let token = workspace_token(&headers)?;
    Ok(Json(load_workspace(&state.pool, token).await?))
}

pub(crate) async fn reset(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DemoEnvelope>, ApiError> {
    let token = workspace_token(&headers)?.to_owned();
    let idempotency_key = idempotency_key(&headers)?;
    let current = load_workspace_row(&state.pool, &token).await?;
    reject_reused_key(&state.pool, &idempotency_key).await?;

    let envelope = seed_workspace(&state.pool, idempotency_key).await?;
    sqlx::query("DELETE FROM demo_workspaces WHERE id = ? AND is_demo = 1")
        .bind(current.id)
        .execute(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
    Ok(Json(envelope))
}

pub(crate) async fn recover(
    State(state): State<AppState>,
    Path(attempt_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<DemoEnvelope>, ApiError> {
    let token = workspace_token(&headers)?.to_owned();
    let idempotency_key = idempotency_key(&headers)?;
    let workspace = load_workspace_row(&state.pool, &token).await?;

    let attempt = sqlx::query_as::<_, AttemptRow>(
        "SELECT id, client_name, scheduled_for, state, reason, email_consent, \
         consent_wording, consent_recorded_at, outcome \
         FROM booking_attempts WHERE id = ? AND workspace_id = ?",
    )
    .bind(&attempt_id)
    .bind(&workspace.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::not_found)?;

    if !attempt.email_consent {
        return Err(ApiError::conflict(
            "consent_required",
            "No email consent was recorded. This recovery stays stopped.",
        ));
    }
    if attempt.state == "completed" {
        return Err(ApiError::conflict(
            "already_completed",
            "This booking is already complete, so no recovery is needed.",
        ));
    }

    let now = Utc::now().timestamp();
    let mut transaction = state.pool.begin().await.map_err(|_| ApiError::internal())?;
    let existing = sqlx::query_scalar::<_, String>(
        "SELECT id FROM outbound_messages WHERE workspace_id = ? AND idempotency_key = ?",
    )
    .bind(&workspace.id)
    .bind(&idempotency_key)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    if existing.is_none() && attempt.state != "recovered" {
        let message_id = Uuid::now_v7().to_string();
        sqlx::query(
            "INSERT INTO outbound_messages \
             (id, workspace_id, attempt_id, idempotency_key, channel, state, created_at) \
             VALUES (?, ?, ?, ?, 'email', 'delivered', ?)",
        )
        .bind(&message_id)
        .bind(&workspace.id)
        .bind(&attempt_id)
        .bind(&idempotency_key)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
        sqlx::query(
            "INSERT INTO delivery_events \
             (id, message_id, status, detail, occurred_at, simulated) \
             VALUES (?, ?, 'delivered', 'Sample email accepted by the in-process demo mailbox.', ?, 1)",
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&message_id)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
        sqlx::query(
            "UPDATE booking_attempts SET state = 'recovered', \
             outcome = 'Sample follow-up delivered' WHERE id = ? AND workspace_id = ?",
        )
        .bind(&attempt_id)
        .bind(&workspace.id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
    }

    transaction
        .commit()
        .await
        .map_err(|_| ApiError::internal())?;
    Ok(Json(load_workspace(&state.pool, &token).await?))
}

async fn seed_workspace(
    pool: &SqlitePool,
    idempotency_key: String,
) -> Result<DemoEnvelope, ApiError> {
    let now = Utc::now().timestamp();
    let expires_at = now + DEMO_TTL.as_secs() as i64;
    let workspace_id = Uuid::now_v7().to_string();
    let token = new_token()?;
    let token_hash = token_hash(&token);
    let mut transaction = pool.begin().await.map_err(|_| ApiError::internal())?;

    sqlx::query(
        "INSERT INTO demo_workspaces \
         (id, token_hash, idempotency_key, is_demo, practice_name, practice_timezone, \
          service_name, service_duration_minutes, deposit_cents, currency, created_at, expires_at) \
         VALUES (?, ?, ?, 1, 'North Star Coaching', 'Europe/London', \
                 '45-minute focus session', 45, 3500, 'GBP', ?, ?)",
    )
    .bind(&workspace_id)
    .bind(&token_hash)
    .bind(&idempotency_key)
    .bind(now)
    .bind(expires_at)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    let attempts = [
        (
            "maya-unfinished",
            "Maya Patel",
            now + 2 * 24 * 60 * 60,
            "unfinished",
            "Left before the sample deposit step",
            true,
            Some(CONSENT_WORDING),
            Some(now - 18 * 60),
            None,
        ),
        (
            "jordan-no-consent",
            "Jordan Lee",
            now + 3 * 24 * 60 * 60,
            "unfinished",
            "Email consent was not recorded",
            false,
            None,
            None,
            None,
        ),
        (
            "alex-completed",
            "Alex Morgan",
            now + 4 * 24 * 60 * 60,
            "completed",
            "Deposit received and booking confirmed",
            true,
            Some(CONSENT_WORDING),
            Some(now - 2 * 24 * 60 * 60),
            Some("Booking confirmed"),
        ),
    ];

    for (suffix, name, scheduled, status, reason, consent, wording, recorded, outcome) in attempts {
        sqlx::query(
            "INSERT INTO booking_attempts \
             (id, workspace_id, client_name, scheduled_for, state, reason, email_consent, \
              consent_wording, consent_recorded_at, outcome) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(format!("{workspace_id}:{suffix}"))
        .bind(&workspace_id)
        .bind(name)
        .bind(scheduled)
        .bind(status)
        .bind(reason)
        .bind(consent)
        .bind(wording)
        .bind(recorded)
        .bind(outcome)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
    }

    let completed_attempt = format!("{workspace_id}:alex-completed");
    let message_id = Uuid::now_v7().to_string();
    sqlx::query(
        "INSERT INTO outbound_messages \
         (id, workspace_id, attempt_id, idempotency_key, channel, state, created_at) \
         VALUES (?, ?, ?, ?, 'email', 'delivered', ?)",
    )
    .bind(&message_id)
    .bind(&workspace_id)
    .bind(&completed_attempt)
    .bind(format!("seed:{workspace_id}"))
    .bind(now - 24 * 60 * 60)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;
    sqlx::query(
        "INSERT INTO delivery_events \
         (id, message_id, status, detail, occurred_at, simulated) \
         VALUES (?, ?, 'delivered', 'Sample confirmation reached the demo mailbox.', ?, 1)",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(&message_id)
    .bind(now - 24 * 60 * 60)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;

    transaction
        .commit()
        .await
        .map_err(|_| ApiError::internal())?;
    load_workspace(pool, &token).await
}

async fn reject_reused_key(pool: &SqlitePool, key: &str) -> Result<(), ApiError> {
    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM demo_workspaces WHERE idempotency_key = ?",
    )
    .bind(key)
    .fetch_one(pool)
    .await
    .map_err(|_| ApiError::internal())?;
    if exists > 0 {
        return Err(ApiError::conflict(
            "request_already_used",
            "That demo request was already used. Start a fresh demo request.",
        ));
    }
    Ok(())
}

async fn load_workspace(pool: &SqlitePool, token: &str) -> Result<DemoEnvelope, ApiError> {
    let row = load_workspace_row(pool, token).await?;
    let attempts = sqlx::query_as::<_, AttemptRow>(
        "SELECT id, client_name, scheduled_for, state, reason, email_consent, \
         consent_wording, consent_recorded_at, outcome FROM booking_attempts \
         WHERE workspace_id = ? ORDER BY scheduled_for",
    )
    .bind(&row.id)
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::internal())?;

    let mut response_attempts = Vec::with_capacity(attempts.len());
    for attempt in attempts {
        let receipts = sqlx::query_as::<_, Receipt>(
            "SELECT m.channel, e.status, e.detail, \
             strftime('%Y-%m-%dT%H:%M:%SZ', e.occurred_at, 'unixepoch') AS occurred_at, \
             e.simulated FROM delivery_events e \
             JOIN outbound_messages m ON m.id = e.message_id \
             WHERE m.attempt_id = ? ORDER BY e.occurred_at",
        )
        .bind(&attempt.id)
        .fetch_all(pool)
        .await
        .map_err(|_| ApiError::internal())?;

        response_attempts.push(Attempt {
            id: attempt.id,
            client_name: attempt.client_name,
            scheduled_for: timestamp(attempt.scheduled_for),
            state: attempt.state,
            reason: attempt.reason,
            consent: Consent {
                email: attempt.email_consent,
                wording: attempt.consent_wording,
                recorded_at: attempt.consent_recorded_at.map(timestamp),
            },
            outcome: attempt.outcome,
            receipts,
        });
    }

    Ok(DemoEnvelope {
        workspace_token: token.to_owned(),
        workspace: DemoWorkspace {
            id: row.id,
            expires_at: timestamp(row.expires_at),
            practice: Practice {
                name: row.practice_name,
                timezone: row.practice_timezone,
            },
            service: Service {
                name: row.service_name,
                duration_minutes: row.service_duration_minutes,
                deposit_cents: row.deposit_cents,
                currency: row.currency,
            },
            attempts: response_attempts,
        },
    })
}

async fn load_workspace_row(pool: &SqlitePool, token: &str) -> Result<WorkspaceRow, ApiError> {
    sqlx::query_as::<_, WorkspaceRow>(
        "SELECT id, practice_name, practice_timezone, service_name, service_duration_minutes, \
         deposit_cents, currency, expires_at FROM demo_workspaces \
         WHERE token_hash = ? AND is_demo = 1 AND expires_at > ?",
    )
    .bind(token_hash(token))
    .bind(Utc::now().timestamp())
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::not_found)
}

async fn purge_expired(pool: &SqlitePool) -> Result<(), ApiError> {
    sqlx::query("DELETE FROM demo_workspaces WHERE is_demo = 1 AND expires_at <= ?")
        .bind(Utc::now().timestamp())
        .execute(pool)
        .await
        .map_err(|_| ApiError::internal())?;
    Ok(())
}

fn workspace_token(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get("x-demo-workspace")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() >= 32 && value.len() <= 128)
        .ok_or_else(|| {
            ApiError::bad_request(
                "demo_token_required",
                "This action needs a valid sample workspace. Start a fresh demo.",
            )
        })
}

fn idempotency_key(headers: &HeaderMap) -> Result<String, ApiError> {
    headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| value.len() >= 8 && value.len() <= 128)
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            ApiError::bad_request(
                "idempotency_key_required",
                "This action needs a request key. Try the action again.",
            )
        })
}

fn new_token() -> Result<String, ApiError> {
    let mut bytes = [0_u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|_| ApiError::internal())?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn token_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn timestamp(value: i64) -> String {
    DateTime::<Utc>::from_timestamp(value, 0)
        .expect("stored timestamps must be valid")
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use serde_json::Value;
    use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
    use tower::ServiceExt;

    use crate::{app_router, migrations};

    async fn test_app() -> (axum::Router, SqlitePool) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("memory database should open");
        migrations::up(&pool).await.expect("migration should apply");
        (app_router(pool.clone(), "test", "../dist"), pool)
    }

    async fn json(response: axum::response::Response) -> Value {
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body should read")
            .to_bytes();
        serde_json::from_slice(&bytes).expect("body should be json")
    }

    fn request(method: &str, uri: &str, key: &str, token: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("x-forwarded-for", "198.51.100.27")
            .header("idempotency-key", key);
        if let Some(token) = token {
            builder = builder.header("x-demo-workspace", token);
        }
        builder.body(Body::empty()).expect("valid request")
    }

    #[tokio::test]
    async fn demo_recovery_is_consent_gated_and_records_a_simulated_receipt() {
        let (app, _) = test_app().await;
        let response = app
            .clone()
            .oneshot(request(
                "POST",
                "/api/v1/demo/workspaces",
                "create-test-1",
                None,
            ))
            .await
            .expect("create route should respond");
        assert_eq!(response.status(), StatusCode::CREATED);
        let created = json(response).await;
        let token = created["workspaceToken"].as_str().expect("token");
        let attempts = created["workspace"]["attempts"]
            .as_array()
            .expect("attempts");
        let consented = attempts
            .iter()
            .find(|attempt| attempt["clientName"] == "Maya Patel")
            .expect("consented sample")["id"]
            .as_str()
            .expect("attempt id");
        let blocked = attempts
            .iter()
            .find(|attempt| attempt["clientName"] == "Jordan Lee")
            .expect("blocked sample")["id"]
            .as_str()
            .expect("attempt id");

        let denied = app
            .clone()
            .oneshot(request(
                "POST",
                &format!("/api/v1/demo/attempts/{blocked}/recover"),
                "recover-blocked-1",
                Some(token),
            ))
            .await
            .expect("blocked route should respond");
        assert_eq!(denied.status(), StatusCode::CONFLICT);
        assert_eq!(json(denied).await["error"], "consent_required");

        let recovered = app
            .clone()
            .oneshot(request(
                "POST",
                &format!("/api/v1/demo/attempts/{consented}/recover"),
                "recover-allowed-1",
                Some(token),
            ))
            .await
            .expect("recovery route should respond");
        assert_eq!(recovered.status(), StatusCode::OK);
        let recovered = json(recovered).await;
        let attempt = recovered["workspace"]["attempts"]
            .as_array()
            .expect("attempts")
            .iter()
            .find(|attempt| attempt["clientName"] == "Maya Patel")
            .expect("recovered sample");
        assert_eq!(attempt["state"], "recovered");
        assert_eq!(attempt["receipts"][0]["status"], "delivered");
        assert_eq!(attempt["receipts"][0]["simulated"], true);
    }

    #[tokio::test]
    async fn reset_replaces_the_token_and_restores_the_seed() {
        let (app, _) = test_app().await;
        let created = json(
            app.clone()
                .oneshot(request(
                    "POST",
                    "/api/v1/demo/workspaces",
                    "create-test-2",
                    None,
                ))
                .await
                .expect("create route should respond"),
        )
        .await;
        let old_token = created["workspaceToken"].as_str().expect("token");
        let reset = json(
            app.clone()
                .oneshot(request(
                    "POST",
                    "/api/v1/demo/reset",
                    "reset-test-2",
                    Some(old_token),
                ))
                .await
                .expect("reset route should respond"),
        )
        .await;
        assert_ne!(reset["workspaceToken"], created["workspaceToken"]);
        assert_eq!(reset["workspace"]["attempts"][0]["state"], "unfinished");

        let old = app
            .oneshot(request(
                "GET",
                "/api/v1/demo/workspace",
                "unused-get",
                Some(old_token),
            ))
            .await
            .expect("old token lookup should respond");
        assert_eq!(old.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn expired_and_non_demo_workspaces_are_never_returned() {
        let (app, pool) = test_app().await;
        let raw_token = "real-workspace-token-that-cannot-be-used";
        sqlx::query(
            "INSERT INTO demo_workspaces \
             (id, token_hash, idempotency_key, is_demo, practice_name, practice_timezone, \
              service_name, service_duration_minutes, deposit_cents, currency, created_at, expires_at) \
             VALUES ('real-practice', ?, 'real-key', 0, 'Private Practice', 'UTC', 'Private service', 60, 1000, 'GBP', 0, 4102444800)",
        )
        .bind(super::token_hash(raw_token))
        .execute(&pool)
        .await
        .expect("fixture should insert");

        let real_lookup = app
            .clone()
            .oneshot(request(
                "GET",
                "/api/v1/demo/workspace",
                "unused-get-2",
                Some(raw_token),
            ))
            .await
            .expect("lookup should respond");
        assert_eq!(real_lookup.status(), StatusCode::NOT_FOUND);

        let created = json(
            app.clone()
                .oneshot(request(
                    "POST",
                    "/api/v1/demo/workspaces",
                    "create-test-3",
                    None,
                ))
                .await
                .expect("create should respond"),
        )
        .await;
        let token = created["workspaceToken"].as_str().expect("token");
        sqlx::query("UPDATE demo_workspaces SET expires_at = 0 WHERE token_hash = ?")
            .bind(super::token_hash(token))
            .execute(&pool)
            .await
            .expect("fixture should expire");
        let expired = app
            .oneshot(request(
                "GET",
                "/api/v1/demo/workspace",
                "unused-get-3",
                Some(token),
            ))
            .await
            .expect("lookup should respond");
        assert_eq!(expired.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn migration_is_reversible() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("memory database should open");
        migrations::up(&pool).await.expect("up migration");
        migrations::up(&pool).await.expect("up migration can rerun");
        migrations::down(&pool).await.expect("down migration");
        let table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'demo_workspaces'",
        )
        .fetch_one(&pool)
        .await
        .expect("catalog should query");
        assert_eq!(table_count, 0);
    }

    #[tokio::test]
    async fn write_limit_uses_forwarded_ip_and_returns_retry_after() {
        let (app, _) = test_app().await;
        for request_number in 0..12 {
            let response = app
                .clone()
                .oneshot(request(
                    "POST",
                    "/api/v1/demo/workspaces",
                    &format!("rate-key-{request_number}"),
                    None,
                ))
                .await
                .expect("limited route should respond");
            assert_eq!(response.status(), StatusCode::CREATED);
        }
        let limited = app
            .oneshot(request(
                "POST",
                "/api/v1/demo/workspaces",
                "rate-key-final",
                None,
            ))
            .await
            .expect("limited route should respond");
        assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(limited.headers().contains_key("retry-after"));
    }
}
