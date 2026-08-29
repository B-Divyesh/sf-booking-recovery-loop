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
use sqlx::{AnyPool, FromRow};
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    channel: String,
    status: String,
    detail: String,
    occurred_at: String,
    simulated: bool,
}

#[derive(Debug, FromRow)]
struct ReceiptRow {
    channel: String,
    status: String,
    detail: String,
    occurred_at: i64,
    simulated: i64,
}

#[derive(Debug, FromRow)]
struct WorkspaceRow {
    id: String,
    is_demo: i64,
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
    email_consent: i64,
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
    let envelope = seed_workspace(&state.pool, idempotency_key).await?;
    Ok((StatusCode::CREATED, Json(envelope)))
}

pub(crate) async fn show(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DemoEnvelope>, ApiError> {
    let _guard = state.demo_lock.lock().await;
    let token = workspace_token(&headers)?;
    Ok(Json(load_workspace(&state.pool, token).await?))
}

pub(crate) async fn reset(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<DemoEnvelope>, ApiError> {
    let _guard = state.demo_lock.lock().await;
    let token = workspace_token(&headers)?.to_owned();
    let idempotency_key = idempotency_key(&headers)?;
    let current = load_workspace_row(&state.pool, &token).await?;

    let envelope = seed_workspace(&state.pool, idempotency_key).await?;
    sqlx::query("UPDATE demo_workspaces SET expires_at = 0 WHERE id = $1 AND is_demo = 1")
        .bind(current.id)
        .execute(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
    Ok(Json(envelope))
}

pub(crate) async fn recover(
    State(state): State<AppState>,
    Path(requested_attempt_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<DemoEnvelope>, ApiError> {
    let _guard = state.demo_lock.lock().await;
    let token = workspace_token(&headers)?.to_owned();
    let idempotency_key = idempotency_key(&headers)?;
    let workspace = load_workspace_row(&state.pool, &token).await?;
    let attempt_suffix = requested_attempt_id
        .rsplit_once(':')
        .map(|(_, suffix)| suffix)
        .unwrap_or(&requested_attempt_id);
    if !matches!(
        attempt_suffix,
        "maya-unfinished" | "jordan-no-consent" | "alex-completed"
    ) {
        return Err(ApiError::not_found());
    }
    let attempt_id = format!("{}:{attempt_suffix}", workspace.id);

    let attempt = sqlx::query_as::<_, AttemptRow>(
        "SELECT id, client_name, scheduled_for, state, reason, email_consent, \
         consent_wording, consent_recorded_at, outcome \
         FROM booking_attempts WHERE id = $1 AND workspace_id = $2",
    )
    .bind(&attempt_id)
    .bind(&workspace.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::not_found)?;

    if attempt.email_consent != 1 {
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
    if attempt.state != "recovered" {
        let message_id = Uuid::now_v7().to_string();
        let inserted = sqlx::query(
            "INSERT INTO outbound_messages \
             (id, workspace_id, attempt_id, idempotency_key, channel, state, created_at) \
             VALUES ($1, $2, $3, $4, 'email', 'delivered', $5) ON CONFLICT DO NOTHING",
        )
        .bind(&message_id)
        .bind(&workspace.id)
        .bind(&attempt_id)
        .bind(&idempotency_key)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
        if inserted.rows_affected() == 1 {
            sqlx::query(
                "INSERT INTO delivery_events \
                 (id, message_id, status, detail, occurred_at, simulated) \
                 VALUES ($1, $2, 'delivered', 'Sample email accepted by the in-process demo mailbox.', $3, 1)",
            )
            .bind(Uuid::now_v7().to_string())
            .bind(&message_id)
            .bind(now)
            .execute(&mut *transaction)
            .await
            .map_err(|_| ApiError::internal())?;
        }
        sqlx::query(
            "UPDATE booking_attempts SET state = 'recovered', \
             outcome = 'Sample follow-up delivered' WHERE id = $1 AND workspace_id = $2",
        )
        .bind(&attempt_id)
        .bind(&workspace.id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
    }

    let response_token = if attempt.state == "recovered" {
        token.clone()
    } else {
        token_with_state(&token, "recovered")?
    };
    if response_token != token {
        sqlx::query(
            "INSERT INTO demo_token_aliases (token_hash, workspace_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(token_hash(&response_token))
        .bind(&workspace.id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
    }

    transaction
        .commit()
        .await
        .map_err(|_| ApiError::internal())?;
    Ok(Json(load_workspace(&state.pool, &response_token).await?))
}

async fn seed_workspace(pool: &AnyPool, idempotency_key: String) -> Result<DemoEnvelope, ApiError> {
    let now = Utc::now().timestamp();
    let expires_at = now + DEMO_TTL.as_secs() as i64;
    let workspace_id = Uuid::now_v7().to_string();
    let token = new_token(now)?;
    let token_hash = token_hash(&token);
    let mut transaction = pool.begin().await.map_err(|_| ApiError::internal())?;

    let inserted_workspace = sqlx::query(
        "INSERT INTO demo_workspaces \
         (id, token_hash, idempotency_key, is_demo, practice_name, practice_timezone, \
          service_name, service_duration_minutes, deposit_cents, currency, created_at, expires_at) \
         VALUES ($1, $2, $3, 1, 'North Star Coaching', 'Europe/London', \
                 '45-minute focus session', 45, 3500, 'GBP', $4, $5) ON CONFLICT DO NOTHING",
    )
    .bind(&workspace_id)
    .bind(&token_hash)
    .bind(&idempotency_key)
    .bind(now)
    .bind(expires_at)
    .execute(&mut *transaction)
    .await
    .map_err(|_| ApiError::internal())?;
    if inserted_workspace.rows_affected() != 1 {
        return Err(ApiError::conflict(
            "request_already_used",
            "That demo request was already used. Start a fresh demo request.",
        ));
    }

    sqlx::query("INSERT INTO demo_token_aliases (token_hash, workspace_id) VALUES ($1, $2)")
        .bind(&token_hash)
        .bind(&workspace_id)
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
              consent_wording, consent_recorded_at, outcome) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(format!("{workspace_id}:{suffix}"))
        .bind(&workspace_id)
        .bind(name)
        .bind(scheduled)
        .bind(status)
        .bind(reason)
        .bind(i64::from(consent))
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
         VALUES ($1, $2, $3, $4, 'email', 'delivered', $5)",
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
         VALUES ($1, $2, 'delivered', 'Sample confirmation reached the demo mailbox.', $3, 1)",
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
    Ok(seeded_envelope(workspace_id, token, now, expires_at))
}

fn seeded_envelope(workspace_id: String, token: String, now: i64, expires_at: i64) -> DemoEnvelope {
    let maya_id = format!("{workspace_id}:maya-unfinished");
    let jordan_id = format!("{workspace_id}:jordan-no-consent");
    let alex_id = format!("{workspace_id}:alex-completed");
    DemoEnvelope {
        workspace_token: token,
        workspace: DemoWorkspace {
            id: workspace_id,
            expires_at: timestamp(expires_at),
            practice: Practice {
                name: "North Star Coaching".to_owned(),
                timezone: "Europe/London".to_owned(),
            },
            service: Service {
                name: "45-minute focus session".to_owned(),
                duration_minutes: 45,
                deposit_cents: 3500,
                currency: "GBP".to_owned(),
            },
            attempts: vec![
                Attempt {
                    id: maya_id,
                    client_name: "Maya Patel".to_owned(),
                    scheduled_for: timestamp(now + 2 * 24 * 60 * 60),
                    state: "unfinished".to_owned(),
                    reason: "Left before the sample deposit step".to_owned(),
                    consent: Consent {
                        email: true,
                        wording: Some(CONSENT_WORDING.to_owned()),
                        recorded_at: Some(timestamp(now - 18 * 60)),
                    },
                    outcome: None,
                    receipts: Vec::new(),
                },
                Attempt {
                    id: jordan_id,
                    client_name: "Jordan Lee".to_owned(),
                    scheduled_for: timestamp(now + 3 * 24 * 60 * 60),
                    state: "unfinished".to_owned(),
                    reason: "Email consent was not recorded".to_owned(),
                    consent: Consent {
                        email: false,
                        wording: None,
                        recorded_at: None,
                    },
                    outcome: None,
                    receipts: Vec::new(),
                },
                Attempt {
                    id: alex_id,
                    client_name: "Alex Morgan".to_owned(),
                    scheduled_for: timestamp(now + 4 * 24 * 60 * 60),
                    state: "completed".to_owned(),
                    reason: "Deposit received and booking confirmed".to_owned(),
                    consent: Consent {
                        email: true,
                        wording: Some(CONSENT_WORDING.to_owned()),
                        recorded_at: Some(timestamp(now - 2 * 24 * 60 * 60)),
                    },
                    outcome: Some("Booking confirmed".to_owned()),
                    receipts: vec![Receipt {
                        channel: "email".to_owned(),
                        status: "delivered".to_owned(),
                        detail: "Sample confirmation reached the demo mailbox.".to_owned(),
                        occurred_at: timestamp(now - 24 * 60 * 60),
                        simulated: true,
                    }],
                },
            ],
        },
    }
}

async fn load_workspace(pool: &AnyPool, token: &str) -> Result<DemoEnvelope, ApiError> {
    let row = load_workspace_row(pool, token).await?;
    workspace_from_row(pool, token, row).await
}

async fn workspace_from_row(
    pool: &AnyPool,
    token: &str,
    row: WorkspaceRow,
) -> Result<DemoEnvelope, ApiError> {
    let attempts = sqlx::query_as::<_, AttemptRow>(
        "SELECT id, client_name, scheduled_for, state, reason, email_consent, \
         consent_wording, consent_recorded_at, outcome FROM booking_attempts \
         WHERE workspace_id = $1 ORDER BY scheduled_for",
    )
    .bind(&row.id)
    .fetch_all(pool)
    .await
    .map_err(|_| ApiError::internal())?;

    let mut response_attempts = Vec::with_capacity(attempts.len());
    for attempt in attempts {
        let receipt_rows = sqlx::query_as::<_, ReceiptRow>(
            "SELECT m.channel, e.status, e.detail, e.occurred_at, e.simulated FROM delivery_events e \
             JOIN outbound_messages m ON m.id = e.message_id \
             WHERE m.attempt_id = $1 ORDER BY e.occurred_at",
        )
        .bind(&attempt.id)
        .fetch_all(pool)
        .await
        .map_err(|_| ApiError::internal())?;

        let receipts = receipt_rows
            .into_iter()
            .map(|receipt| Receipt {
                channel: receipt.channel,
                status: receipt.status,
                detail: receipt.detail,
                occurred_at: timestamp(receipt.occurred_at),
                simulated: receipt.simulated == 1,
            })
            .collect();
        response_attempts.push(Attempt {
            id: attempt.id,
            client_name: attempt.client_name,
            scheduled_for: timestamp(attempt.scheduled_for),
            state: attempt.state,
            reason: attempt.reason,
            consent: Consent {
                email: attempt.email_consent == 1,
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

async fn load_workspace_row(pool: &AnyPool, token: &str) -> Result<WorkspaceRow, ApiError> {
    if let Some(row) = query_workspace_row(pool, token).await? {
        if row.is_demo == 1 && row.expires_at > Utc::now().timestamp() {
            return Ok(row);
        }
        return Err(ApiError::not_found());
    }
    let portable = parse_token(token).ok_or_else(ApiError::not_found)?;
    hydrate_workspace(pool, token, portable).await?;
    query_workspace_row(pool, token)
        .await?
        .ok_or_else(ApiError::not_found)
}

async fn query_workspace_row(
    pool: &AnyPool,
    token: &str,
) -> Result<Option<WorkspaceRow>, ApiError> {
    sqlx::query_as::<_, WorkspaceRow>(
        "SELECT w.id, w.is_demo, w.practice_name, w.practice_timezone, w.service_name, \
         service_duration_minutes, deposit_cents, currency, expires_at \
         FROM demo_workspaces w LEFT JOIN demo_token_aliases a ON a.workspace_id = w.id \
         WHERE w.token_hash = $1 OR a.token_hash = $2 LIMIT 1",
    )
    .bind(token_hash(token))
    .bind(token_hash(token))
    .fetch_optional(pool)
    .await
    .map_err(|_| ApiError::internal())
}

#[derive(Clone, Copy)]
struct PortableToken<'a> {
    entropy: &'a str,
    created_at: i64,
    state: &'a str,
}

async fn hydrate_workspace(
    pool: &AnyPool,
    token: &str,
    portable: PortableToken<'_>,
) -> Result<(), ApiError> {
    let hash = token_hash(token);
    let seeded = seed_workspace(pool, format!("rehydrate:{}", &hash[..24])).await?;
    let workspace_id = seeded.workspace.id;
    let expires_at = portable.created_at + DEMO_TTL.as_secs() as i64;
    sqlx::query(
        "UPDATE demo_workspaces SET token_hash = $1, created_at = $2, expires_at = $3 WHERE id = $4",
    )
    .bind(&hash)
    .bind(portable.created_at)
    .bind(expires_at)
    .bind(&workspace_id)
    .execute(pool)
    .await
    .map_err(|_| ApiError::internal())?;

    if portable.state == "recovered" {
        let attempt_id = format!("{workspace_id}:maya-unfinished");
        let message_id = Uuid::now_v7().to_string();
        let now = Utc::now().timestamp();
        let mut transaction = pool.begin().await.map_err(|_| ApiError::internal())?;
        sqlx::query(
            "INSERT INTO outbound_messages \
             (id, workspace_id, attempt_id, idempotency_key, channel, state, created_at) \
             VALUES ($1, $2, $3, $4, 'email', 'delivered', $5)",
        )
        .bind(&message_id)
        .bind(&workspace_id)
        .bind(&attempt_id)
        .bind(format!("rehydrated-recovery:{}", portable.entropy))
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
        sqlx::query(
            "INSERT INTO delivery_events \
             (id, message_id, status, detail, occurred_at, simulated) \
             VALUES ($1, $2, 'delivered', 'Sample email accepted by the in-process demo mailbox.', $3, 1)",
        )
        .bind(Uuid::now_v7().to_string())
        .bind(&message_id)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
        sqlx::query(
            "UPDATE booking_attempts SET state = 'recovered', \
             outcome = 'Sample follow-up delivered' WHERE id = $1 AND workspace_id = $2",
        )
        .bind(&attempt_id)
        .bind(&workspace_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| ApiError::internal())?;
        transaction
            .commit()
            .await
            .map_err(|_| ApiError::internal())?;
    }
    Ok(())
}

pub(crate) async fn purge_expired(pool: &AnyPool) -> Result<(), ApiError> {
    sqlx::query(
        "DELETE FROM demo_workspaces WHERE is_demo = 1 AND expires_at <= $1 AND created_at <= $2",
    )
    .bind(Utc::now().timestamp())
    .bind(Utc::now().timestamp() - DEMO_TTL.as_secs() as i64)
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

fn new_token(created_at: i64) -> Result<String, ApiError> {
    let mut bytes = [0_u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|_| ApiError::internal())?;
    Ok(format!(
        "v1.{}.{}.fresh",
        URL_SAFE_NO_PAD.encode(bytes),
        created_at
    ))
}

fn parse_token(token: &str) -> Option<PortableToken<'_>> {
    let mut parts = token.split('.');
    if parts.next()? != "v1" {
        return None;
    }
    let entropy = parts.next()?;
    let created_at = parts.next()?.parse::<i64>().ok()?;
    let state = parts.next()?;
    if parts.next().is_some()
        || !matches!(state, "fresh" | "recovered")
        || URL_SAFE_NO_PAD.decode(entropy).ok()?.len() != TOKEN_BYTES
    {
        return None;
    }
    let now = Utc::now().timestamp();
    if created_at > now + 5 * 60 || created_at + DEMO_TTL.as_secs() as i64 <= now {
        return None;
    }
    Some(PortableToken {
        entropy,
        created_at,
        state,
    })
}

fn token_with_state(token: &str, state: &str) -> Result<String, ApiError> {
    let portable = parse_token(token).ok_or_else(ApiError::not_found)?;
    Ok(format!(
        "v1.{}.{}.{}",
        portable.entropy, portable.created_at, state
    ))
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
    use base64::Engine;
    use http_body_util::BodyExt;
    use serde_json::Value;
    use sqlx::{any::AnyPoolOptions, AnyPool};
    use tower::ServiceExt;

    use crate::{app_router, migrations};

    async fn test_pool() -> AnyPool {
        sqlx::any::install_default_drivers();
        AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("memory database should open")
    }

    async fn test_app() -> (axum::Router, AnyPool) {
        let pool = test_pool().await;
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
            .header("x-forwarded-for", "198.51.100.27, 10.0.0.7")
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
    async fn demo_never_reads_or_mutates_real_practice_fixture() {
        let (app, pool) = test_app().await;
        let raw_token = "real-workspace-token-that-cannot-be-used";
        sqlx::query(
            "INSERT INTO demo_workspaces \
             (id, token_hash, idempotency_key, is_demo, practice_name, practice_timezone, \
              service_name, service_duration_minutes, deposit_cents, currency, created_at, expires_at) \
             VALUES ('real-practice', $1, 'real-key', 0, 'Private Practice', 'UTC', 'Private service', 60, 1000, 'GBP', 0, 4102444800)",
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

        let real_mutation = app
            .clone()
            .oneshot(request(
                "POST",
                "/api/v1/demo/attempts/real-practice:anything/recover",
                "real-mutation-key",
                Some(raw_token),
            ))
            .await
            .expect("mutation should respond");
        assert_eq!(real_mutation.status(), StatusCode::NOT_FOUND);
        let unchanged: String = sqlx::query_scalar(
            "SELECT practice_name FROM demo_workspaces WHERE id = 'real-practice'",
        )
        .fetch_one(&pool)
        .await
        .expect("real fixture should remain");
        assert_eq!(unchanged, "Private Practice");

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
        sqlx::query("UPDATE demo_workspaces SET expires_at = 0 WHERE token_hash = $1")
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
        let pool = test_pool().await;
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
            assert_eq!(
                response.headers()["x-ratelimit-limit"],
                crate::WRITE_BURST.to_string()
            );
            assert!(!response.headers().contains_key("x-ratelimit-whitelisted"));
        }
        let limited = app
            .clone()
            .oneshot(request(
                "POST",
                "/api/v1/demo/workspaces",
                "rate-key-final",
                None,
            ))
            .await
            .expect("limited route should respond");
        assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(limited.headers()["x-ratelimit-limit"], "12");
        assert_eq!(limited.headers()["x-ratelimit-remaining"], "0");
        assert!(!limited.headers().contains_key("x-ratelimit-whitelisted"));
        let retry_after = limited.headers()["retry-after"]
            .to_str()
            .expect("Retry-After should be text")
            .parse::<u64>()
            .expect("Retry-After should be seconds");
        assert!((1..=crate::WRITE_REPLENISH_SECONDS).contains(&retry_after));
        assert_eq!(
            limited.headers()["x-ratelimit-after"],
            retry_after.to_string()
        );

        let other_client = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/demo/workspaces")
                    .header("x-forwarded-for", "198.51.100.28, 10.0.0.8")
                    .header("idempotency-key", "other-forwarded-client")
                    .body(Body::empty())
                    .expect("valid request"),
            )
            .await
            .expect("a different client should respond");
        assert_eq!(other_client.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn portable_token_has_256_random_bits_and_24_hour_expiry() {
        let now = super::Utc::now().timestamp();
        let token = super::new_token(now).expect("token");
        let parsed = super::parse_token(&token).expect("portable token");
        assert_eq!(
            super::URL_SAFE_NO_PAD
                .decode(parsed.entropy)
                .expect("entropy")
                .len(),
            32
        );
        assert_eq!(now + super::DEMO_TTL.as_secs() as i64, now + 24 * 60 * 60);
        let expired = token.replacen(&now.to_string(), &(now - 24 * 60 * 60).to_string(), 1);
        assert!(super::parse_token(&expired).is_none());
    }

    #[tokio::test]
    async fn eight_concurrent_recoveries_never_return_server_error() {
        let database_path = std::env::temp_dir().join(format!(
            "booking-recovery-concurrency-{}.db",
            uuid::Uuid::now_v7()
        ));
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite://{}?mode=rwc", database_path.display()))
            .await
            .expect("file database");
        migrations::up(&pool).await.expect("migration");
        let (source, _) = test_app().await;
        let created = json(
            source
                .oneshot(request(
                    "POST",
                    "/api/v1/demo/workspaces",
                    "concurrent-create",
                    None,
                ))
                .await
                .expect("create"),
        )
        .await;
        let token = created["workspaceToken"]
            .as_str()
            .expect("token")
            .to_owned();
        let attempt = created["workspace"]["attempts"][0]["id"]
            .as_str()
            .expect("attempt")
            .to_owned();
        let app = app_router(pool.clone(), "test", "../dist");

        let mut tasks = tokio::task::JoinSet::new();
        for number in 0..8 {
            let service = app.clone();
            let token = token.clone();
            let uri = format!("/api/v1/demo/attempts/{attempt}/recover");
            tasks.spawn(async move {
                service
                    .oneshot(request(
                        "POST",
                        &uri,
                        &format!("concurrent-key-{number}"),
                        Some(&token),
                    ))
                    .await
                    .expect("recovery response")
                    .status()
            });
        }
        let mut statuses = Vec::new();
        while let Some(status) = tasks.join_next().await {
            statuses.push(status.expect("task"));
        }
        assert_eq!(statuses.len(), 8);
        assert!(statuses.iter().all(|status| *status == StatusCode::OK));
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM outbound_messages WHERE idempotency_key LIKE 'concurrent-key-%'",
        )
        .fetch_one(&pool)
        .await
        .expect("message count");
        assert_eq!(count, 1);
        pool.close().await;
        let _ = std::fs::remove_file(database_path);
    }

    #[tokio::test]
    async fn portable_token_preserves_state_across_replica_databases() {
        async fn replica() -> (axum::Router, AnyPool) {
            let pool = test_pool().await;
            migrations::up(&pool).await.expect("migration should apply");
            (app_router(pool.clone(), "test", "../dist"), pool)
        }

        let (first, _) = replica().await;
        let created = json(
            first
                .oneshot(request(
                    "POST",
                    "/api/v1/demo/workspaces",
                    "replica-create",
                    None,
                ))
                .await
                .expect("first replica should create"),
        )
        .await;
        let initial_token = created["workspaceToken"].as_str().expect("token");
        let initial_attempt = created["workspace"]["attempts"][0]["id"]
            .as_str()
            .expect("attempt");

        let (second, _) = replica().await;
        let recovered = second
            .oneshot(request(
                "POST",
                &format!("/api/v1/demo/attempts/{initial_attempt}/recover"),
                "replica-recovery",
                Some(initial_token),
            ))
            .await
            .expect("second replica should recover");
        assert_eq!(recovered.status(), StatusCode::OK);
        let recovered = json(recovered).await;
        let recovered_token = recovered["workspaceToken"]
            .as_str()
            .expect("recovered token");
        assert!(recovered_token.ends_with(".recovered"));

        let (third, _) = replica().await;
        let reloaded = third
            .oneshot(request(
                "GET",
                "/api/v1/demo/workspace",
                "unused-replica-get",
                Some(recovered_token),
            ))
            .await
            .expect("third replica should load");
        assert_eq!(reloaded.status(), StatusCode::OK);
        let reloaded = json(reloaded).await;
        assert_eq!(reloaded["workspace"]["attempts"][0]["state"], "recovered");
        assert_eq!(
            reloaded["workspace"]["attempts"][0]["receipts"][0]["simulated"],
            true
        );
    }

    async fn shared_replica_apps(
        database_path: &std::path::Path,
        replicas: usize,
    ) -> (Vec<axum::Router>, Vec<AnyPool>) {
        sqlx::any::install_default_drivers();
        let database_url = format!("sqlite://{}?mode=rwc", database_path.display());
        let first_pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("shared database should open");
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&first_pool)
            .await
            .expect("WAL should enable concurrent test readers");
        sqlx::query("PRAGMA busy_timeout = 10000")
            .execute(&first_pool)
            .await
            .expect("first connection should wait for a write lock");
        migrations::up(&first_pool)
            .await
            .expect("shared migrations should apply");

        let mut pools = vec![first_pool];
        for _ in 1..replicas {
            let pool = AnyPoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await
                .expect("replica database connection should open");
            sqlx::query("PRAGMA busy_timeout = 10000")
                .execute(&pool)
                .await
                .expect("replica should wait for a write lock");
            pools.push(pool);
        }
        let apps = pools
            .iter()
            .map(|pool| app_router(pool.clone(), "shared-replica", "../dist"))
            .collect();
        (apps, pools)
    }

    #[tokio::test]
    async fn reset_revokes_old_token_across_shared_replicas_under_concurrent_reads() {
        let database_path = std::env::temp_dir().join(format!(
            "booking-recovery-reset-replica-{}.db",
            uuid::Uuid::now_v7()
        ));
        let (apps, pools) = shared_replica_apps(&database_path, 4).await;
        let created = json(
            apps[0]
                .clone()
                .oneshot(request(
                    "POST",
                    "/api/v1/demo/workspaces",
                    "shared-reset-create",
                    None,
                ))
                .await
                .expect("workspace should create"),
        )
        .await;
        let old_token = created["workspaceToken"]
            .as_str()
            .expect("old token")
            .to_owned();
        let reset = json(
            apps[1]
                .clone()
                .oneshot(request(
                    "POST",
                    "/api/v1/demo/reset",
                    "shared-reset-request",
                    Some(&old_token),
                ))
                .await
                .expect("reset should respond"),
        )
        .await;
        assert_ne!(reset["workspaceToken"], old_token);

        let mut requests = tokio::task::JoinSet::new();
        for number in 0..24 {
            let app = apps[number % apps.len()].clone();
            let token = old_token.clone();
            requests.spawn(async move {
                app.oneshot(request(
                    "GET",
                    "/api/v1/demo/workspace",
                    "unused-read-key",
                    Some(&token),
                ))
                .await
                .expect("old-token read should respond")
                .status()
            });
        }
        let mut statuses = Vec::new();
        while let Some(result) = requests.join_next().await {
            statuses.push(result.expect("read task should finish"));
        }
        assert_eq!(statuses.len(), 24);
        assert!(
            statuses
                .iter()
                .all(|status| *status == StatusCode::NOT_FOUND),
            "a reset token must stay revoked on every shared-store replica: {statuses:?}"
        );

        for pool in pools {
            pool.close().await;
        }
        let _ = std::fs::remove_file(database_path);
    }

    #[tokio::test]
    async fn shared_write_limit_allows_only_twelve_of_forty_concurrent_cross_replica_writes() {
        let database_path = std::env::temp_dir().join(format!(
            "booking-recovery-rate-replica-{}.db",
            uuid::Uuid::now_v7()
        ));
        let (apps, pools) = shared_replica_apps(&database_path, 4).await;
        let mut requests = tokio::task::JoinSet::new();
        for number in 0..40 {
            let app = apps[number % apps.len()].clone();
            requests.spawn(async move {
                app.oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/demo/workspaces")
                        .header("x-forwarded-for", "203.0.113.240, 10.0.0.7")
                        .header(
                            "idempotency-key",
                            format!("shared-concurrent-write-{number}"),
                        )
                        .body(Body::empty())
                        .expect("valid write request"),
                )
                .await
                .expect("write should respond")
            });
        }
        let mut created = 0;
        let mut limited = 0;
        while let Some(result) = requests.join_next().await {
            let response = result.expect("write task should finish");
            match response.status() {
                StatusCode::CREATED => created += 1,
                StatusCode::TOO_MANY_REQUESTS => {
                    limited += 1;
                    assert_eq!(response.headers()["x-ratelimit-limit"], "12");
                    assert_eq!(response.headers()["retry-after"], "60");
                }
                status => panic!("shared limiter must return 201 or 429, got {status}"),
            }
        }
        assert_eq!(
            created, 12,
            "one client must have one global write allowance"
        );
        assert_eq!(limited, 28);

        for pool in pools {
            pool.close().await;
        }
        let _ = std::fs::remove_file(database_path);
    }
}
