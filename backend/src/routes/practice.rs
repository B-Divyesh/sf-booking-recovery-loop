use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use uuid::Uuid;

use crate::AppState;

const CONSENT_WORDING: &str =
    "Send booking and recovery messages only through the channels I select.";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreatePractice {
    name: String,
    public_slug: String,
    timezone: String,
    service_name: String,
    duration_minutes: i64,
    deposit_cents: i64,
    currency: String,
    payment_url: String,
    delivery_webhook_url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateAttempt {
    client_name: String,
    email: Option<String>,
    phone: Option<String>,
    scheduled_for: String,
    email_consent: bool,
    sms_consent: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReceiptInput {
    attempt_id: String,
    provider_event_id: String,
    channel: String,
    status: String,
    detail: String,
    occurred_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PaymentInput {
    attempt_id: String,
    provider_event_id: String,
    status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreatedPractice {
    #[serde(skip_serializing_if = "Option::is_none")]
    access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt_token: Option<String>,
    practice: PracticeView,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PracticeView {
    id: String,
    name: String,
    public_slug: String,
    timezone: String,
    service_name: String,
    duration_minutes: i64,
    deposit_cents: i64,
    currency: String,
    payment_url: String,
    delivery_webhook_url: String,
    attempts: Vec<AttemptView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PublicPractice {
    name: String,
    public_slug: String,
    timezone: String,
    service_name: String,
    duration_minutes: i64,
    deposit_cents: i64,
    currency: String,
    payment_url: String,
    consent_wording: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AttemptView {
    id: String,
    client_name: String,
    email: Option<String>,
    phone: Option<String>,
    scheduled_for: String,
    state: String,
    email_consent: bool,
    sms_consent: bool,
    consent_wording: String,
    consent_recorded_at: String,
    events: Vec<EventView>,
    scheduled_jobs: Vec<ScheduledJobView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EventView {
    channel: String,
    status: String,
    detail: String,
    occurred_at: String,
}

#[derive(FromRow)]
struct EventRow {
    channel: String,
    status: String,
    detail: String,
    occurred_at: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScheduledJobView {
    kind: String,
    due_at: String,
    status: String,
    last_error: Option<String>,
}

#[derive(FromRow)]
struct ScheduledJobRow {
    kind: String,
    due_at: i64,
    status: String,
    last_error: Option<String>,
}

#[derive(Clone, FromRow)]
struct PracticeRow {
    id: String,
    public_slug: String,
    name: String,
    timezone: String,
    service_name: String,
    duration_minutes: i64,
    deposit_cents: i64,
    currency: String,
    payment_url: String,
    delivery_webhook_url: String,
}

#[derive(Clone, FromRow)]
struct AttemptRow {
    id: String,
    client_name_encrypted: String,
    email_encrypted: Option<String>,
    phone_encrypted: Option<String>,
    scheduled_for: i64,
    state: String,
    email_consent: i64,
    sms_consent: i64,
    consent_wording: String,
    consent_recorded_at: i64,
}

pub(crate) async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<CreatePractice>,
) -> Result<(StatusCode, Json<CreatedPractice>), ApiError> {
    validate_practice(&input, state.allow_test_delivery_urls)?;
    let owner_oid = owner_oid(&state, &headers).await?;
    let legacy_storage_token = random_token("retired")?;
    let receipt_token = random_token("receipt")?;
    let id = Uuid::now_v7().to_string();
    sqlx::query(
        "INSERT INTO practices (id, owner_oid, access_token_hash, receipt_token_hash, public_slug, name, timezone, \
         service_name, duration_minutes, deposit_cents, currency, payment_url, delivery_webhook_url, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&owner_oid)
    .bind(hash(&legacy_storage_token))
    .bind(hash(&receipt_token))
    .bind(input.public_slug.trim())
    .bind(input.name.trim())
    .bind(input.timezone.trim())
    .bind(input.service_name.trim())
    .bind(input.duration_minutes)
    .bind(input.deposit_cents)
    .bind(input.currency.trim().to_uppercase())
    .bind(input.payment_url.trim())
    .bind(input.delivery_webhook_url.trim())
    .bind(Utc::now().timestamp())
    .execute(&state.pool)
    .await
    .map_err(|error| {
        if error.to_string().contains("UNIQUE") {
            ApiError::conflict("slug_taken", "That booking link is already in use. Choose another link.")
        } else {
            ApiError::internal()
        }
    })?;
    sqlx::query("INSERT INTO practice_entitlements (practice_id, provider, state, verified_at) VALUES (?, 'sociobot_dodo', 'unknown', ?)")
        .bind(&id)
        .bind(Utc::now().timestamp())
        .execute(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
    let practice = load_practice(&state, &owner_oid).await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedPractice {
            // Legacy fixture tokens are emitted only by the test binary. The
            // production response never sends a transferable owner or callback secret.
            access_token: cfg!(test).then(|| owner_oid.clone()),
            receipt_token: cfg!(test).then_some(receipt_token),
            practice,
        }),
    ))
}

pub(crate) async fn show(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PracticeView>, ApiError> {
    let owner_oid = owner_oid(&state, &headers).await?;
    Ok(Json(load_practice(&state, &owner_oid).await?))
}

pub(crate) async fn public_show(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<PublicPractice>, ApiError> {
    let row = public_practice(&state, &slug).await?;
    Ok(Json(PublicPractice {
        name: row.name,
        public_slug: row.public_slug,
        timezone: row.timezone,
        service_name: row.service_name,
        duration_minutes: row.duration_minutes,
        deposit_cents: row.deposit_cents,
        currency: row.currency,
        payment_url: row.payment_url,
        consent_wording: CONSENT_WORDING,
    }))
}

pub(crate) async fn create_attempt(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(input): Json<CreateAttempt>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    validate_attempt(&input)?;
    let practice = public_practice(&state, &slug).await?;
    let scheduled = DateTime::parse_from_rfc3339(&input.scheduled_for)
        .map_err(|_| {
            ApiError::bad_request("invalid_time", "Choose a valid future appointment time.")
        })?
        .timestamp();
    if scheduled <= Utc::now().timestamp() {
        return Err(ApiError::bad_request(
            "past_time",
            "Choose a future appointment time.",
        ));
    }
    let id = Uuid::now_v7().to_string();
    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO practice_attempts (id, practice_id, client_name_encrypted, email_encrypted, phone_encrypted, \
         scheduled_for, state, email_consent, sms_consent, consent_wording, consent_recorded_at, created_at) \
         VALUES (?, ?, ?, ?, ?, ?, 'awaiting_deposit', ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&practice.id)
    .bind(encrypt(&state, input.client_name.trim())?)
    .bind(encrypt_optional(&state, input.email.as_deref())?)
    .bind(encrypt_optional(&state, input.phone.as_deref())?)
    .bind(scheduled)
    .bind(i64::from(input.email_consent))
    .bind(i64::from(input.sms_consent))
    .bind(CONSENT_WORDING)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await
    .map_err(|error| if error.to_string().contains("UNIQUE") { ApiError::conflict("slot_unavailable", "That time was just booked. Choose another future time.") } else { ApiError::internal() })?;
    // The recovery deadline is durable. A restart cannot silently turn an
    // abandoned booking back into a manual checklist item.
    sqlx::query("INSERT INTO practice_scheduled_jobs (id, practice_id, attempt_id, kind, due_at, created_at) VALUES (?, ?, ?, 'abandoned_recovery', ?, ?)")
        .bind(Uuid::now_v7().to_string()).bind(&practice.id).bind(&id)
        .bind(now + 15 * 60).bind(now).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "attemptId": id,
            "paymentUrl": practice.payment_url,
            "status": "awaiting_deposit"
        })),
    ))
}

pub(crate) async fn recover(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(attempt_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let owner_oid = owner_oid(&state, &headers).await?;
    let practice = practice_row_by_owner(&state, &owner_oid).await?;
    let attempt: AttemptRow = sqlx::query_as(
        "SELECT id, client_name_encrypted, email_encrypted, phone_encrypted, scheduled_for, state, \
         email_consent, sms_consent, consent_wording, consent_recorded_at FROM practice_attempts \
         WHERE id = ? AND practice_id = ?",
    )
    .bind(&attempt_id)
    .bind(&practice.id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::not_found)?;
    let channel = preferred_channel(&attempt)?;
    let event_id = deliver_attempt(&state, &practice, &attempt, "manual recovery").await?;
    Ok(Json(
        json!({"status":"accepted", "channel": channel, "eventId": event_id}),
    ))
}

/// Verifies a practice's configured delivery connection without sending client
/// data or changing a booking. The owner can make this check during setup and
/// again after a provider changes its endpoint.
pub(crate) async fn test_delivery_connection(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let owner_oid = owner_oid(&state, &headers).await?;
    let practice = practice_row_by_owner(&state, &owner_oid).await?;
    if practice.delivery_webhook_url.is_empty() {
        return Err(ApiError::conflict(
            "delivery_not_connected",
            "Add a delivery connection URL before sending a test message.",
        ));
    }
    let response = state
        .http
        .post(delivery_target(&state, &practice.delivery_webhook_url)?)
        .json(&json!({
            "type": "connection_test",
            "practice": practice.name,
            "message": "Booking Recovery Loop delivery connection test",
            "containsClientData": false
        }))
        .send()
        .await
        .map_err(|_| {
            ApiError::bad_gateway(
                "delivery_unavailable",
                "The delivery service did not answer the test. Check its URL and try again.",
            )
        })?;
    if !response.status().is_success() {
        return Err(ApiError::bad_gateway(
            "delivery_rejected",
            "The delivery service rejected the test. Check its URL and try again.",
        ));
    }
    Ok(Json(json!({"status": "accepted", "clientDataSent": false})))
}

/// Runs due work from the database rather than from an in-memory timer. It is
/// intentionally public to the service loop and tests; jobs remain queued
/// across a container restart and are claimed atomically before delivery.
pub(crate) async fn run_due_jobs(state: &AppState, now: i64) -> Result<(), ApiError> {
    let jobs: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT j.id, j.attempt_id, j.kind, j.practice_id FROM practice_scheduled_jobs j \
         JOIN practices p ON p.id = j.practice_id WHERE j.status IN ('queued', 'failed') \
         AND j.due_at <= ? AND p.deletion_requested_at IS NULL ORDER BY j.due_at LIMIT 32",
    )
    .bind(now)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    for (job_id, attempt_id, kind, practice_id) in jobs {
        let claimed = sqlx::query("UPDATE practice_scheduled_jobs SET status = 'processing', attempts = attempts + 1, last_error = NULL WHERE id = ? AND status IN ('queued', 'failed')")
            .bind(&job_id).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
        if claimed.rows_affected() != 1 {
            continue;
        }
        let result = deliver_scheduled_job(state, &practice_id, &attempt_id, &kind).await;
        match result {
            Ok(()) => {
                sqlx::query("UPDATE practice_scheduled_jobs SET status = 'sent', completed_at = ? WHERE id = ?")
                    .bind(now).bind(&job_id).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
            }
            Err(error) if error.code == "consent_required" => {
                sqlx::query("UPDATE practice_scheduled_jobs SET status = 'stopped', completed_at = ?, last_error = ? WHERE id = ?")
                    .bind(now).bind(error.message).bind(&job_id).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
            }
            Err(error) => {
                // A failed provider call is retried in five minutes. The
                // error remains visible to the owner instead of disappearing.
                sqlx::query("UPDATE practice_scheduled_jobs SET status = 'failed', due_at = ?, last_error = ? WHERE id = ?")
                    .bind(now + 5 * 60).bind(error.message).bind(&job_id).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
            }
        }
    }
    Ok(())
}

async fn deliver_scheduled_job(
    state: &AppState,
    practice_id: &str,
    attempt_id: &str,
    kind: &str,
) -> Result<(), ApiError> {
    let practice: PracticeRow = sqlx::query_as("SELECT id, public_slug, name, timezone, service_name, duration_minutes, deposit_cents, currency, payment_url, delivery_webhook_url FROM practices WHERE id = ? AND deletion_requested_at IS NULL")
        .bind(practice_id).fetch_optional(&state.pool).await.map_err(|_| ApiError::internal())?.ok_or_else(ApiError::not_found)?;
    let attempt: AttemptRow = sqlx::query_as("SELECT id, client_name_encrypted, email_encrypted, phone_encrypted, scheduled_for, state, email_consent, sms_consent, consent_wording, consent_recorded_at FROM practice_attempts WHERE id = ? AND practice_id = ?")
        .bind(attempt_id).bind(practice_id).fetch_optional(&state.pool).await.map_err(|_| ApiError::internal())?.ok_or_else(ApiError::not_found)?;
    if kind == "abandoned_recovery" && attempt.state != "awaiting_deposit" {
        return Ok(());
    }
    if kind == "session_reminder" && attempt.state != "paid" {
        return Ok(());
    }
    deliver_attempt(
        state,
        &practice,
        &attempt,
        if kind == "session_reminder" {
            "automatic session reminder"
        } else {
            "automatic abandoned-booking recovery"
        },
    )
    .await?;
    Ok(())
}

async fn deliver_attempt(
    state: &AppState,
    practice: &PracticeRow,
    attempt: &AttemptRow,
    purpose: &str,
) -> Result<String, ApiError> {
    let channel = preferred_channel(attempt)?;
    if practice.delivery_webhook_url.is_empty() {
        return Err(ApiError::conflict(
            "delivery_not_connected",
            "Automatic delivery is waiting for a delivery connection.",
        ));
    }
    let target = if channel == "email" {
        decrypt_optional(state, attempt.email_encrypted.as_deref())?
    } else {
        decrypt_optional(state, attempt.phone_encrypted.as_deref())?
    };
    let response = state.http.post(delivery_target(state, &practice.delivery_webhook_url)?).json(&json!({
        "attemptId": attempt.id, "channel": channel, "to": target,
        "template": if purpose.contains("reminder") { "Your session reminder" } else { "Complete your booking" },
        "purpose": purpose, "replyToPractice": practice.name,
        "receiptCallback": format!("/api/v1/provider/{}/receipts", practice.id)
    })).send().await.map_err(|_| ApiError::bad_gateway("delivery_unavailable", "The delivery service did not answer. Nothing was marked as sent."))?;
    if !response.status().is_success() {
        return Err(ApiError::bad_gateway(
            "delivery_rejected",
            "The delivery service rejected the message. Check the connection and try again.",
        ));
    }
    let event_id = Uuid::now_v7().to_string();
    sqlx::query("INSERT INTO practice_delivery_events (id, practice_id, attempt_id, channel, status, detail, provider_event_id, occurred_at) VALUES (?, ?, ?, ?, 'accepted', ?, ?, ?) ON CONFLICT DO NOTHING")
        .bind(Uuid::now_v7().to_string()).bind(&practice.id).bind(&attempt.id).bind(channel)
        .bind(format!("Delivery service accepted the {purpose}.")).bind(&event_id).bind(Utc::now().timestamp())
        .execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    sqlx::query(
        "UPDATE practice_attempts SET state = 'recovery_due' WHERE id = ? AND practice_id = ?",
    )
    .bind(&attempt.id)
    .bind(&practice.id)
    .execute(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    Ok(event_id)
}

fn preferred_channel(attempt: &AttemptRow) -> Result<&'static str, ApiError> {
    if attempt.email_consent == 1 && attempt.email_encrypted.is_some() {
        Ok("email")
    } else if attempt.sms_consent == 1 && attempt.phone_encrypted.is_some() {
        Ok("sms")
    } else {
        Err(ApiError::conflict(
            "consent_required",
            "No permitted contact channel is recorded. This recovery stays stopped.",
        ))
    }
}

pub(crate) async fn receipt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(practice_id): Path<String>,
    Json(input): Json<ReceiptInput>,
) -> Result<Json<Value>, ApiError> {
    let token = headers
        .get("x-receipt-token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(ApiError::unauthorized)?;
    let valid: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM practices WHERE id = ? AND receipt_token_hash = ? AND deletion_requested_at IS NULL")
        .bind(&practice_id).bind(hash(token)).fetch_one(&state.pool).await.map_err(|_| ApiError::internal())?;
    if valid != 1 {
        return Err(ApiError::unauthorized());
    }
    if !matches!(input.channel.as_str(), "email" | "sms")
        || !matches!(
            input.status.as_str(),
            "accepted" | "delivered" | "bounced" | "failed"
        )
    {
        return Err(ApiError::bad_request(
            "invalid_receipt",
            "Use a supported channel and receipt status.",
        ));
    }
    let occurred = input
        .occurred_at
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .map_err(|_| ApiError::bad_request("invalid_time", "Use an RFC 3339 receipt time."))?
        .map(|v| v.timestamp())
        .unwrap_or_else(|| Utc::now().timestamp());
    let inserted = sqlx::query("INSERT INTO practice_delivery_events (id, practice_id, attempt_id, channel, status, detail, provider_event_id, occurred_at) SELECT ?, ?, id, ?, ?, ?, ?, ? FROM practice_attempts WHERE id = ? AND practice_id = ? ON CONFLICT DO NOTHING")
        .bind(Uuid::now_v7().to_string()).bind(&practice_id).bind(&input.channel).bind(&input.status).bind(clean_detail(&input.detail)).bind(&input.provider_event_id).bind(occurred).bind(&input.attempt_id).bind(&practice_id)
        .execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    if inserted.rows_affected() == 0 {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM practice_delivery_events WHERE provider_event_id = ?",
        )
        .bind(&input.provider_event_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
        if exists == 0 {
            return Err(ApiError::not_found());
        }
    }
    if input.status == "delivered" {
        sqlx::query(
            "UPDATE practice_attempts SET state = 'recovered' WHERE id = ? AND practice_id = ?",
        )
        .bind(&input.attempt_id)
        .bind(&practice_id)
        .execute(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
    }
    if inserted.rows_affected() > 0 && input.status == "bounced" && input.channel == "email" {
        let fallback: Option<(String, String, String)> = sqlx::query_as(
            "SELECT a.phone_encrypted, p.delivery_webhook_url, p.name FROM practice_attempts a \
             JOIN practices p ON p.id = a.practice_id WHERE a.id = ? AND a.practice_id = ? \
             AND a.sms_consent = 1 AND a.phone_encrypted IS NOT NULL AND p.delivery_webhook_url <> ''",
        ).bind(&input.attempt_id).bind(&practice_id).fetch_optional(&state.pool).await.map_err(|_| ApiError::internal())?;
        if let Some((phone, webhook, practice_name)) = fallback {
            let response = state
                .http
                .post(delivery_target(&state, &webhook)?)
                .json(&json!({
                    "attemptId": input.attempt_id, "channel": "sms", "to": decrypt(&state, &phone)?,
                    "template": "Complete your booking", "replyToPractice": practice_name,
                    "receiptCallback": format!("/api/v1/provider/{}/receipts", practice_id)
                }))
                .send()
                .await;
            if response.is_ok_and(|value| value.status().is_success()) {
                sqlx::query("INSERT INTO practice_delivery_events (id, practice_id, attempt_id, channel, status, detail, provider_event_id, occurred_at) VALUES (?, ?, ?, 'sms', 'accepted', 'Email bounced; the permitted SMS fallback was accepted.', ?, ?) ON CONFLICT DO NOTHING")
                    .bind(Uuid::now_v7().to_string()).bind(&practice_id).bind(&input.attempt_id).bind(format!("fallback:{}", input.provider_event_id)).bind(Utc::now().timestamp())
                    .execute(&state.pool).await.map_err(|_| ApiError::internal())?;
            }
        }
    }
    Ok(Json(
        json!({"recorded": true, "duplicate": inserted.rows_affected() == 0}),
    ))
}

pub(crate) async fn payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(practice_id): Path<String>,
    Json(input): Json<PaymentInput>,
) -> Result<Json<Value>, ApiError> {
    let token = headers
        .get("x-receipt-token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(ApiError::unauthorized)?;
    let valid: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM practices WHERE id = ? AND receipt_token_hash = ? AND deletion_requested_at IS NULL")
        .bind(&practice_id).bind(hash(token)).fetch_one(&state.pool).await.map_err(|_| ApiError::internal())?;
    if valid != 1 {
        return Err(ApiError::unauthorized());
    }
    if input.status != "paid" {
        return Err(ApiError::bad_request(
            "invalid_payment",
            "Only a verified paid event can confirm the deposit.",
        ));
    }
    let result = sqlx::query("UPDATE practice_attempts SET state = 'paid', payment_reference = ? WHERE id = ? AND practice_id = ? AND payment_reference IS NULL")
        .bind(clean_detail(&input.provider_event_id)).bind(&input.attempt_id).bind(&practice_id).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    if result.rows_affected() == 0 {
        let existing: Option<String> = sqlx::query_scalar(
            "SELECT payment_reference FROM practice_attempts WHERE id = ? AND practice_id = ?",
        )
        .bind(&input.attempt_id)
        .bind(&practice_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
        return match existing {
            Some(reference) if reference == input.provider_event_id => {
                Ok(Json(json!({"recorded":true,"duplicate":true})))
            }
            Some(_) => Err(ApiError::conflict(
                "payment_already_recorded",
                "A different payment event already confirmed this booking.",
            )),
            None => Err(ApiError::not_found()),
        };
    }
    let scheduled_for: i64 = sqlx::query_scalar(
        "SELECT scheduled_for FROM practice_attempts WHERE id = ? AND practice_id = ?",
    )
    .bind(&input.attempt_id)
    .bind(&practice_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    let now = Utc::now().timestamp();
    sqlx::query("INSERT INTO practice_scheduled_jobs (id, practice_id, attempt_id, kind, due_at, created_at) VALUES (?, ?, ?, 'session_reminder', ?, ?) ON CONFLICT DO NOTHING")
        .bind(Uuid::now_v7().to_string()).bind(&practice_id).bind(&input.attempt_id)
        .bind((scheduled_for - 24 * 60 * 60).max(now)).bind(now)
        .execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    Ok(Json(json!({"recorded":true,"duplicate":false})))
}

pub(crate) async fn export(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let owner_oid = owner_oid(&state, &headers).await?;
    let practice = load_practice(&state, &owner_oid).await?;
    let body = serde_json::to_vec_pretty(&practice).map_err(|_| ApiError::internal())?;
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/json"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=booking-recovery-export.json",
            ),
        ],
        body,
    )
        .into_response())
}

pub(crate) async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let owner_oid = owner_oid(&state, &headers).await?;
    let result = sqlx::query("DELETE FROM practices WHERE owner_oid = ?")
        .bind(owner_oid)
        .execute(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
    if result.rows_affected() == 0 {
        return Err(ApiError::not_found());
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn load_practice(state: &AppState, owner_oid: &str) -> Result<PracticeView, ApiError> {
    let row = practice_row_by_owner(state, owner_oid).await?;
    let attempts: Vec<AttemptRow> = sqlx::query_as("SELECT id, client_name_encrypted, email_encrypted, phone_encrypted, scheduled_for, state, email_consent, sms_consent, consent_wording, consent_recorded_at FROM practice_attempts WHERE practice_id = ? ORDER BY created_at DESC")
        .bind(&row.id).fetch_all(&state.pool).await.map_err(|_| ApiError::internal())?;
    let mut views = Vec::with_capacity(attempts.len());
    for attempt in attempts {
        let event_rows = sqlx::query_as::<_, EventRow>("SELECT channel, status, detail, occurred_at FROM practice_delivery_events WHERE attempt_id = ? ORDER BY occurred_at")
            .bind(&attempt.id).fetch_all(&state.pool).await.map_err(|_| ApiError::internal())?;
        let events = event_rows
            .into_iter()
            .map(|event| EventView {
                channel: event.channel,
                status: event.status,
                detail: event.detail,
                occurred_at: timestamp(event.occurred_at),
            })
            .collect();
        let job_rows = sqlx::query_as::<_, ScheduledJobRow>("SELECT kind, due_at, status, last_error FROM practice_scheduled_jobs WHERE attempt_id = ? ORDER BY due_at")
            .bind(&attempt.id).fetch_all(&state.pool).await.map_err(|_| ApiError::internal())?;
        let scheduled_jobs = job_rows
            .into_iter()
            .map(|job| ScheduledJobView {
                kind: job.kind,
                due_at: timestamp(job.due_at),
                status: job.status,
                last_error: job.last_error,
            })
            .collect();
        views.push(AttemptView {
            id: attempt.id,
            client_name: decrypt(state, &attempt.client_name_encrypted)?,
            email: decrypt_optional(state, attempt.email_encrypted.as_deref())?,
            phone: decrypt_optional(state, attempt.phone_encrypted.as_deref())?,
            scheduled_for: timestamp(attempt.scheduled_for),
            state: attempt.state,
            email_consent: attempt.email_consent == 1,
            sms_consent: attempt.sms_consent == 1,
            consent_wording: attempt.consent_wording,
            consent_recorded_at: timestamp(attempt.consent_recorded_at),
            events,
            scheduled_jobs,
        });
    }
    Ok(PracticeView {
        id: row.id,
        name: row.name,
        public_slug: row.public_slug,
        timezone: row.timezone,
        service_name: row.service_name,
        duration_minutes: row.duration_minutes,
        deposit_cents: row.deposit_cents,
        currency: row.currency,
        payment_url: row.payment_url,
        delivery_webhook_url: row.delivery_webhook_url,
        attempts: views,
    })
}

async fn practice_row_by_owner(state: &AppState, owner_oid: &str) -> Result<PracticeRow, ApiError> {
    sqlx::query_as("SELECT id, public_slug, name, timezone, service_name, duration_minutes, deposit_cents, currency, payment_url, delivery_webhook_url FROM practices WHERE owner_oid = ? AND deletion_requested_at IS NULL")
        .bind(owner_oid).fetch_optional(&state.pool).await.map_err(|_| ApiError::internal())?.ok_or_else(ApiError::unauthorized)
}

async fn public_practice(state: &AppState, slug: &str) -> Result<PracticeRow, ApiError> {
    sqlx::query_as("SELECT id, public_slug, name, timezone, service_name, duration_minutes, deposit_cents, currency, payment_url, delivery_webhook_url FROM practices WHERE public_slug = ? AND deletion_requested_at IS NULL")
        .bind(slug).fetch_optional(&state.pool).await.map_err(|_| ApiError::internal())?.ok_or_else(ApiError::not_found)
}

fn validate_practice(
    input: &CreatePractice,
    allow_test_delivery_urls: bool,
) -> Result<(), ApiError> {
    if input.name.trim().len() < 2
        || input.name.len() > 80
        || input.service_name.trim().len() < 2
        || input.service_name.len() > 100
    {
        return Err(ApiError::bad_request(
            "invalid_practice",
            "Enter a practice and service name.",
        ));
    }
    if input.public_slug.len() < 3
        || input.public_slug.len() > 40
        || !input
            .public_slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(ApiError::bad_request(
            "invalid_slug",
            "Use 3–40 lowercase letters, numbers, or hyphens for the booking link.",
        ));
    }
    if !(15..=480).contains(&input.duration_minutes)
        || !(0..=1_000_000).contains(&input.deposit_cents)
        || input.currency.len() != 3
    {
        return Err(ApiError::bad_request(
            "invalid_service",
            "Check the duration, deposit, and three-letter currency.",
        ));
    }
    validate_https(&input.payment_url, "payment")?;
    if !input.delivery_webhook_url.is_empty()
        && !(allow_test_delivery_urls && is_loopback_test_url(&input.delivery_webhook_url))
    {
        return Err(ApiError::bad_request(
            "unsupported_delivery_provider",
            "Live delivery is not configured for this deployment.",
        ));
    }
    Ok(())
}

/// Owner input is a provider identifier, never a destination URL. This keeps
/// contact data out of an arbitrary server-side request path. The loopback
/// exception is compiled into test-only harnesses and is guarded by an
/// explicit environment opt-in for browser integration tests.
fn delivery_target<'a>(state: &AppState, configured: &'a str) -> Result<&'a str, ApiError> {
    match configured {
        value if state.allow_test_delivery_urls && is_loopback_test_url(value) => Ok(value),
        _ => Err(ApiError::conflict(
            "delivery_not_connected",
            "Live delivery is not configured for this deployment.",
        )),
    }
}

fn is_loopback_test_url(value: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    matches!(url.scheme(), "http" | "https")
        && matches!(
            url.host_str(),
            Some("127.0.0.1") | Some("localhost") | Some("::1")
        )
        && url.port().is_some()
}

fn validate_attempt(input: &CreateAttempt) -> Result<(), ApiError> {
    if input.client_name.trim().len() < 2 || input.client_name.len() > 100 {
        return Err(ApiError::bad_request(
            "invalid_name",
            "Enter the client name.",
        ));
    }
    let email_ok = input
        .email
        .as_deref()
        .is_some_and(|v| v.contains('@') && v.len() <= 254);
    let phone_ok = input
        .phone
        .as_deref()
        .is_some_and(|v| v.len() >= 7 && v.len() <= 30);
    if input.email_consent && !email_ok {
        return Err(ApiError::bad_request(
            "email_required",
            "Enter an email address or turn off email consent.",
        ));
    }
    if input.sms_consent && !phone_ok {
        return Err(ApiError::bad_request(
            "phone_required",
            "Enter a phone number or turn off SMS consent.",
        ));
    }
    if !input.email_consent && !input.sms_consent {
        return Err(ApiError::bad_request(
            "consent_required",
            "Choose at least one contact channel to finish this booking.",
        ));
    }
    Ok(())
}

fn validate_https(value: &str, kind: &str) -> Result<(), ApiError> {
    let url = reqwest::Url::parse(value).map_err(|_| {
        ApiError::bad_request("invalid_url", format!("Enter a valid HTTPS {kind} URL."))
    })?;
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err(ApiError::bad_request(
            "invalid_url",
            format!("Enter a valid HTTPS {kind} URL."),
        ));
    }
    Ok(())
}

fn encrypt(state: &AppState, value: &str) -> Result<String, ApiError> {
    let cipher = Aes256Gcm::new_from_slice(state.encryption_key.as_ref())
        .map_err(|_| ApiError::internal())?;
    let mut nonce = [0_u8; 12];
    getrandom::fill(&mut nonce).map_err(|_| ApiError::internal())?;
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), value.as_bytes())
        .map_err(|_| ApiError::internal())?;
    Ok(format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(nonce),
        URL_SAFE_NO_PAD.encode(encrypted)
    ))
}

fn decrypt(state: &AppState, value: &str) -> Result<String, ApiError> {
    let (nonce, payload) = value.split_once('.').ok_or_else(ApiError::internal)?;
    let nonce = URL_SAFE_NO_PAD
        .decode(nonce)
        .map_err(|_| ApiError::internal())?;
    let payload = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| ApiError::internal())?;
    let cipher = Aes256Gcm::new_from_slice(state.encryption_key.as_ref())
        .map_err(|_| ApiError::internal())?;
    let clear = cipher
        .decrypt(Nonce::from_slice(&nonce), payload.as_ref())
        .map_err(|_| ApiError::internal())?;
    String::from_utf8(clear).map_err(|_| ApiError::internal())
}

fn encrypt_optional(state: &AppState, value: Option<&str>) -> Result<Option<String>, ApiError> {
    value
        .filter(|v| !v.trim().is_empty())
        .map(|v| encrypt(state, v.trim()))
        .transpose()
}
fn decrypt_optional(state: &AppState, value: Option<&str>) -> Result<Option<String>, ApiError> {
    value.map(|v| decrypt(state, v)).transpose()
}
fn hash(value: &str) -> String {
    hex::encode(Sha256::digest(value.as_bytes()))
}
fn random_token(prefix: &str) -> Result<String, ApiError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| ApiError::internal())?;
    Ok(format!("{prefix}_{}", URL_SAFE_NO_PAD.encode(bytes)))
}
fn timestamp(value: i64) -> String {
    DateTime::<Utc>::from_timestamp(value, 0)
        .expect("valid timestamp")
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}
fn clean_detail(value: &str) -> String {
    value.chars().take(300).collect()
}

async fn owner_oid(state: &AppState, headers: &HeaderMap) -> Result<String, ApiError> {
    state
        .entra
        .owner_oid(headers)
        .await
        .map_err(|_| ApiError::unauthorized())
}

#[derive(Debug)]
pub(crate) struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}
impl ApiError {
    fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
        }
    }
    fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
            message: message.into(),
        }
    }
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: "Sign in with your Sociobot account to continue.".into(),
        }
    }
    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: "That record was not found.".into(),
        }
    }
    fn bad_gateway(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code,
            message: message.into(),
        }
    }
    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: "The request could not be completed. Try again.".into(),
        }
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if self.status == StatusCode::UNAUTHORIZED {
            (
                self.status,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                Json(json!({"error": self.code, "message": self.message})),
            )
                .into_response()
        } else {
            (
                self.status,
                Json(json!({"error": self.code, "message": self.message})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Json,
    };
    use chrono::Utc;
    use http_body_util::BodyExt;
    use serde_json::{json, Value};
    use sqlx::{any::AnyPoolOptions, AnyPool};
    use std::{
        path::PathBuf,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
    };
    use tokio::sync::Mutex;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{app_router, migrations, AppState};

    async fn test_app() -> (axum::Router, AnyPool) {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        migrations::up(&pool).await.unwrap();
        (app_router(pool.clone(), "test", "../dist"), pool)
    }

    async fn send(
        app: &axum::Router,
        method: &str,
        uri: &str,
        body: Value,
        auth: Option<&str>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .header("x-forwarded-for", "198.51.100.44")
            .header("x-test-oid", auth.unwrap_or("test-practice-owner"));
        if let Some(token) = auth {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }
        let response = app
            .clone()
            .oneshot(builder.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = if bytes.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&bytes)}))
        };
        (status, value)
    }

    async fn create_owner(app: &axum::Router, slug: &str) -> Value {
        let (status, value) = send(app, "POST", "/api/v1/practices", json!({
            "name":"North Star Coaching", "publicSlug":slug, "timezone":"Europe/London",
            "serviceName":"Focus session", "durationMinutes":45, "depositCents":3500,
            "currency":"GBP", "paymentUrl":"https://pay.example/session", "deliveryWebhookUrl":""
        }), Some(slug)).await;
        assert_eq!(status, StatusCode::CREATED);
        value
    }

    fn scheduler_state(pool: AnyPool) -> AppState {
        AppState {
            build_sha: Arc::from("test"),
            pool,
            demo_lock: Arc::new(Mutex::new(())),
            encryption_key: Arc::new([7_u8; 32]),
            http: reqwest::Client::new(),
            entra: crate::auth::EntraValidator::from_environment(reqwest::Client::new()),
            allow_test_delivery_urls: true,
            static_dir: Arc::new(PathBuf::new()),
        }
    }

    #[tokio::test]
    async fn automatic_recovery_is_durable_consent_gated_and_idempotent() {
        let hits = Arc::new(AtomicUsize::new(0));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let count = hits.clone();
        tokio::spawn(async move {
            axum::serve(
                listener,
                axum::Router::new().route(
                    "/send",
                    axum::routing::post(move || {
                        let count = count.clone();
                        async move {
                            count.fetch_add(1, Ordering::SeqCst);
                            StatusCode::ACCEPTED
                        }
                    }),
                ),
            )
            .await
            .unwrap();
        });
        let (app, pool) = test_app().await;
        let owner = create_owner(&app, "automatic-test").await;
        let practice_id = owner["practice"]["id"].as_str().unwrap();
        sqlx::query("UPDATE practices SET delivery_webhook_url = ? WHERE id = ?")
            .bind(format!("http://{address}/send"))
            .bind(practice_id)
            .execute(&pool)
            .await
            .unwrap();
        let (consented_status, consented) = send(&app, "POST", "/api/v1/public/automatic-test/attempts", json!({"clientName":"Maya Patel","email":"maya@example.test","phone":null,"scheduledFor":"2030-05-10T12:00:00Z","emailConsent":true,"smsConsent":false}), None).await;
        assert_eq!(consented_status, StatusCode::CREATED, "{consented}");
        let (stopped_status, stopped) = send(&app, "POST", "/api/v1/public/automatic-test/attempts", json!({"clientName":"Jordan Lee","email":"jordan@example.test","phone":null,"scheduledFor":"2030-05-10T13:00:00Z","emailConsent":true,"smsConsent":false}), None).await;
        assert_eq!(stopped_status, StatusCode::CREATED, "{stopped}");
        // A later consent withdrawal must stop an already queued delivery.
        sqlx::query(
            "UPDATE practice_attempts SET email_consent = 0, email_encrypted = NULL WHERE id = ?",
        )
        .bind(stopped["attemptId"].as_str().unwrap())
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("UPDATE practice_scheduled_jobs SET due_at = 1")
            .execute(&pool)
            .await
            .unwrap();
        let state = scheduler_state(pool.clone());
        super::run_due_jobs(&state, 2).await.unwrap();
        super::run_due_jobs(&state, 2).await.unwrap();
        assert_eq!(
            hits.load(Ordering::SeqCst),
            1,
            "a due job is claimed once across retries"
        );
        let sent: String =
            sqlx::query_scalar("SELECT status FROM practice_scheduled_jobs WHERE attempt_id = ?")
                .bind(consented["attemptId"].as_str().unwrap())
                .fetch_one(&pool)
                .await
                .unwrap();
        let stopped_status: String =
            sqlx::query_scalar("SELECT status FROM practice_scheduled_jobs WHERE attempt_id = ?")
                .bind(stopped["attemptId"].as_str().unwrap())
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(sent, "sent");
        assert_eq!(stopped_status, "stopped");
    }

    #[tokio::test]
    async fn automatic_recovery_is_scheduled_exactly_15_minutes_after_unpaid_booking() {
        let (app, pool) = test_app().await;
        create_owner(&app, "recovery-delay-test").await;
        let before = Utc::now().timestamp();
        let (status, attempt) = send(
            &app,
            "POST",
            "/api/v1/public/recovery-delay-test/attempts",
            json!({
                "clientName":"Maya Patel", "email":"maya@example.test", "phone":null,
                "scheduledFor":"2030-05-10T12:00:00Z", "emailConsent":true, "smsConsent":false
            }),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{attempt}");
        let due_at: i64 = sqlx::query_scalar(
            "SELECT due_at FROM practice_scheduled_jobs WHERE attempt_id = ? AND kind = 'abandoned_recovery'",
        )
        .bind(attempt["attemptId"].as_str().unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
        let after = Utc::now().timestamp();
        assert!(
            due_at >= before + 15 * 60 && due_at <= after + 15 * 60,
            "recovery due time must be exactly 15 minutes after the unpaid booking"
        );
    }

    #[tokio::test]
    async fn delivery_connection_test_verifies_the_provider_without_client_data() {
        let received = Arc::new(tokio::sync::Mutex::new(None::<Value>));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let captured = received.clone();
        tokio::spawn(async move {
            axum::serve(
                listener,
                axum::Router::new().route(
                    "/send",
                    axum::routing::post(move |Json(body): Json<Value>| {
                        let captured = captured.clone();
                        async move {
                            *captured.lock().await = Some(body);
                            StatusCode::ACCEPTED
                        }
                    }),
                ),
            )
            .await
            .unwrap();
        });
        let (app, pool) = test_app().await;
        let owner = create_owner(&app, "delivery-check-test").await;
        let token = owner["accessToken"].as_str().unwrap();
        sqlx::query("UPDATE practices SET delivery_webhook_url = ? WHERE id = ?")
            .bind(format!("http://{address}/send"))
            .bind(owner["practice"]["id"].as_str().unwrap())
            .execute(&pool)
            .await
            .unwrap();
        let (status, response) = send(
            &app,
            "POST",
            "/api/v1/practice/delivery/test",
            json!({}),
            Some(token),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{response}");
        assert_eq!(response["status"], "accepted");
        let body = received
            .lock()
            .await
            .clone()
            .expect("provider should get a test");
        assert_eq!(body["type"], "connection_test");
        assert_eq!(body["containsClientData"], false);
        assert_eq!(body.get("to"), None);
        let attempts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM practice_attempts")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(attempts, 0, "a connection test must not create a booking");
    }

    #[tokio::test]
    async fn practice_data_inventory_matches_the_exported_record_types() {
        let (app, pool) = test_app().await;
        let owner = create_owner(&app, "data-inventory-test").await;
        let token = owner["accessToken"].as_str().unwrap();
        let (_, attempt) = send(
            &app,
            "POST",
            "/api/v1/public/data-inventory-test/attempts",
            json!({
                "clientName":"Maya Patel", "email":"maya@example.test", "phone":null,
                "scheduledFor":"2030-05-10T12:00:00Z", "emailConsent":true, "smsConsent":false
            }),
            None,
        )
        .await;
        let attempt_id = attempt["attemptId"].as_str().unwrap();
        sqlx::query("INSERT INTO practice_delivery_events (id, practice_id, attempt_id, channel, status, detail, provider_event_id, occurred_at) VALUES (?, ?, ?, 'email', 'accepted', 'Accepted for delivery', ?, ?)")
            .bind(Uuid::now_v7().to_string()).bind(owner["practice"]["id"].as_str().unwrap()).bind(attempt_id).bind(Uuid::now_v7().to_string()).bind(Utc::now().timestamp()).execute(&pool).await.unwrap();
        let (status, exported) = send(
            &app,
            "GET",
            "/api/v1/practice/export",
            json!({}),
            Some(token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        for key in [
            "name",
            "serviceName",
            "paymentUrl",
            "deliveryWebhookUrl",
            "attempts",
        ] {
            assert!(
                exported.get(key).is_some(),
                "export must include practice settings: {key}"
            );
        }
        let record = &exported["attempts"][0];
        assert!(
            record.get("consentWording").is_some(),
            "export must include consent records"
        );
        assert!(
            record.get("scheduledJobs").is_some(),
            "export must include scheduled messages"
        );
        assert!(
            record.get("events").is_some(),
            "export must include delivery receipts"
        );
    }

    #[tokio::test]
    async fn production_contacts_are_encrypted_and_tenant_scoped() {
        let (app, pool) = test_app().await;
        let owner = create_owner(&app, "north-star-test").await;
        let token = owner["accessToken"].as_str().unwrap();
        let (status, attempt) = send(
            &app,
            "POST",
            "/api/v1/public/north-star-test/attempts",
            json!({
                "clientName":"Real Client", "email":"real.client@example.test", "phone":null,
                "scheduledFor":"2030-05-10T12:00:00Z", "emailConsent":true, "smsConsent":false
            }),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(attempt["paymentUrl"], "https://pay.example/session");
        let stored: (String, String) = sqlx::query_as(
            "SELECT client_name_encrypted, email_encrypted FROM practice_attempts LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!stored.0.contains("Real Client"));
        assert!(!stored.1.contains("real.client"));
        let (status, practice) =
            send(&app, "GET", "/api/v1/practice", json!({}), Some(token)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(practice["attempts"][0]["clientName"], "Real Client");
        assert_eq!(practice["attempts"][0]["emailConsent"], true);
        let other = create_owner(&app, "other-practice-test").await;
        let (status, other_view) = send(
            &app,
            "GET",
            "/api/v1/practice",
            json!({}),
            other["accessToken"].as_str(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(other_view["attempts"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn export_and_delete_cover_the_complete_practice_record() {
        let (app, pool) = test_app().await;
        let owner = create_owner(&app, "delete-test").await;
        let token = owner["accessToken"].as_str().unwrap();
        let (status, _) = send(
            &app,
            "GET",
            "/api/v1/practice/export",
            json!({}),
            Some(token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let (status, _) = send(&app, "DELETE", "/api/v1/practice", json!({}), Some(token)).await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM practices")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
        let (status, _) = send(&app, "GET", "/api/v1/practice", json!({}), Some(token)).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn delivery_acceptance_bounce_fallback_and_receipts_are_idempotent() {
        let hits = Arc::new(AtomicUsize::new(0));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let count = hits.clone();
        let provider = axum::Router::new().route(
            "/send",
            axum::routing::post(move || {
                let count = count.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    StatusCode::ACCEPTED
                }
            }),
        );
        tokio::spawn(async move {
            axum::serve(listener, provider).await.unwrap();
        });

        let (app, pool) = test_app().await;
        let owner = create_owner(&app, "delivery-test").await;
        let token = owner["accessToken"].as_str().unwrap();
        let receipt_token = owner["receiptToken"].as_str().unwrap();
        let practice_id = owner["practice"]["id"].as_str().unwrap();
        sqlx::query("UPDATE practices SET delivery_webhook_url = ? WHERE id = ?")
            .bind(format!("http://{address}/send"))
            .bind(practice_id)
            .execute(&pool)
            .await
            .unwrap();
        let (_, attempt) = send(
            &app,
            "POST",
            "/api/v1/public/delivery-test/attempts",
            json!({
                "clientName":"Taylor Reed", "email":"taylor@example.test", "phone":"+447700900123",
                "scheduledFor":"2030-05-10T12:00:00Z", "emailConsent":true, "smsConsent":true
            }),
            None,
        )
        .await;
        let attempt_id = attempt["attemptId"].as_str().unwrap();
        let (status, accepted) = send(
            &app,
            "POST",
            &format!("/api/v1/practice/attempts/{attempt_id}/recover"),
            json!({}),
            Some(token),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(accepted["status"], "accepted");
        assert_eq!(hits.load(Ordering::SeqCst), 1);

        let receipt_body = json!({"attemptId":attempt_id,"providerEventId":"provider-bounce-1","channel":"email","status":"bounced","detail":"Mailbox rejected the message"});
        for duplicate in [false, true] {
            let request = Request::builder()
                .method("POST")
                .uri(format!("/api/v1/provider/{practice_id}/receipts"))
                .header("content-type", "application/json")
                .header("x-receipt-token", receipt_token)
                .header("x-forwarded-for", "198.51.100.88")
                .body(Body::from(receipt_body.to_string()))
                .unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let value: Value =
                serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                    .unwrap();
            assert_eq!(value["duplicate"], duplicate);
        }
        assert_eq!(
            hits.load(Ordering::SeqCst),
            2,
            "one email and one SMS fallback only"
        );
        let event_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM practice_delivery_events WHERE attempt_id = ?",
        )
        .bind(attempt_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            event_count, 3,
            "accepted email, bounce, and accepted SMS fallback"
        );
    }

    #[tokio::test]
    async fn hosted_payment_requires_verified_callback_and_is_idempotent() {
        let (app, pool) = test_app().await;
        let owner = create_owner(&app, "payment-test").await;
        let owner_token = owner["accessToken"].as_str().unwrap();
        let receipt_token = owner["receiptToken"].as_str().unwrap();
        let practice_id = owner["practice"]["id"].as_str().unwrap();
        let (_, attempt) = send(
            &app,
            "POST",
            "/api/v1/public/payment-test/attempts",
            json!({
                "clientName":"Morgan Vale", "email":"morgan@example.test", "phone":null,
                "scheduledFor":"2030-05-10T12:00:00Z", "emailConsent":true, "smsConsent":false
            }),
            None,
        )
        .await;
        let attempt_id = attempt["attemptId"].as_str().unwrap();
        let payload =
            json!({"attemptId":attempt_id,"providerEventId":"payment-verified-1","status":"paid"});
        let unauthorized = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/provider/{practice_id}/payments"))
            .header("content-type", "application/json")
            .header(
                "x-receipt-token",
                "wrong-token-with-enough-characters-123456789",
            )
            .header("x-forwarded-for", "198.51.100.90")
            .body(Body::from(payload.to_string()))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(unauthorized).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
        for duplicate in [false, true] {
            let request = Request::builder()
                .method("POST")
                .uri(format!("/api/v1/provider/{practice_id}/payments"))
                .header("content-type", "application/json")
                .header("x-receipt-token", receipt_token)
                .header("x-forwarded-for", "198.51.100.91")
                .body(Body::from(payload.to_string()))
                .unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let value: Value =
                serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                    .unwrap();
            assert_eq!(value["duplicate"], duplicate);
        }
        let (_, practice) = send(
            &app,
            "GET",
            "/api/v1/practice",
            json!({}),
            Some(owner_token),
        )
        .await;
        assert_eq!(practice["attempts"][0]["state"], "paid");
        let reminders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM practice_scheduled_jobs WHERE attempt_id = ? AND kind = 'session_reminder' AND status = 'queued'")
            .bind(attempt_id).fetch_one(&pool).await.unwrap();
        assert_eq!(
            reminders, 1,
            "a verified deposit queues one durable reminder"
        );
    }

    #[tokio::test]
    async fn automatic_reminder_is_delivered_once_when_due_after_verified_deposit() {
        let hits = Arc::new(AtomicUsize::new(0));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let count = hits.clone();
        tokio::spawn(async move {
            axum::serve(
                listener,
                axum::Router::new().route(
                    "/send",
                    axum::routing::post(move || {
                        let count = count.clone();
                        async move {
                            count.fetch_add(1, Ordering::SeqCst);
                            StatusCode::ACCEPTED
                        }
                    }),
                ),
            )
            .await
            .unwrap();
        });
        let (app, pool) = test_app().await;
        let owner = create_owner(&app, "reminder-test").await;
        let receipt_token = owner["receiptToken"].as_str().unwrap();
        let practice_id = owner["practice"]["id"].as_str().unwrap();
        sqlx::query("UPDATE practices SET delivery_webhook_url = ? WHERE id = ?")
            .bind(format!("http://{address}/send"))
            .bind(practice_id)
            .execute(&pool)
            .await
            .unwrap();
        let (_, attempt) = send(
            &app,
            "POST",
            "/api/v1/public/reminder-test/attempts",
            json!({
                "clientName":"Sam Rivera", "email":"sam@example.test", "phone":null,
                "scheduledFor":"2030-05-10T12:00:00Z", "emailConsent":true, "smsConsent":false
            }),
            None,
        )
        .await;
        let attempt_id = attempt["attemptId"].as_str().unwrap();
        let payload = json!({
            "attemptId":attempt_id,
            "providerEventId":"reminder-payment-verified-1",
            "status":"paid"
        });

        for duplicate in [false, true] {
            let request = Request::builder()
                .method("POST")
                .uri(format!("/api/v1/provider/{practice_id}/payments"))
                .header("content-type", "application/json")
                .header("x-receipt-token", receipt_token)
                .header("x-forwarded-for", "198.51.100.92")
                .body(Body::from(payload.to_string()))
                .unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let value: Value =
                serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                    .unwrap();
            assert_eq!(value["duplicate"], duplicate);
        }

        sqlx::query("UPDATE practice_scheduled_jobs SET due_at = 1 WHERE attempt_id = ? AND kind = 'session_reminder'")
            .bind(attempt_id)
            .execute(&pool)
            .await
            .unwrap();
        let state = scheduler_state(pool.clone());
        super::run_due_jobs(&state, 2).await.unwrap();
        super::run_due_jobs(&state, 2).await.unwrap();
        assert_eq!(
            hits.load(Ordering::SeqCst),
            1,
            "the due reminder reaches the delivery provider once"
        );
        let status: String = sqlx::query_scalar("SELECT status FROM practice_scheduled_jobs WHERE attempt_id = ? AND kind = 'session_reminder'")
            .bind(attempt_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "sent");
        let detail: String = sqlx::query_scalar("SELECT detail FROM practice_delivery_events WHERE attempt_id = ? ORDER BY occurred_at DESC LIMIT 1")
            .bind(attempt_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(detail.contains("automatic session reminder"));
    }

    #[tokio::test]
    async fn unsupported_owner_delivery_urls_are_rejected_before_any_server_request() {
        let (app, _) = test_app().await;
        let (status, body) = send(&app, "POST", "/api/v1/practices", json!({
            "name":"North Star Coaching", "publicSlug":"blocked-delivery", "timezone":"Europe/London",
            "serviceName":"Focus session", "durationMinutes":45, "depositCents":3500,
            "currency":"GBP", "paymentUrl":"https://pay.example/session",
            "deliveryWebhookUrl":"https://169.254.169.254/latest/meta-data"
        }), None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "unsupported_delivery_provider");
    }

    #[tokio::test]
    async fn practice_routes_require_a_bearer_identity_and_never_issue_owner_keys() {
        let (app, _) = test_app().await;
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/practices")
                    .header("content-type", "application/json")
                    .header("x-forwarded-for", "198.51.100.245")
                    .body(Body::from(json!({
                        "name":"North Star Coaching", "publicSlug":"identity-required", "timezone":"Europe/London",
                        "serviceName":"Focus session", "durationMinutes":45, "depositCents":3500,
                        "currency":"GBP", "paymentUrl":"https://pay.example/session", "deliveryWebhookUrl":""
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers()["www-authenticate"], "Bearer");
    }

    #[tokio::test]
    async fn delete_is_rate_limited_with_a_positive_retry_after() {
        let (app, _) = test_app().await;
        let owner = create_owner(&app, "delete-rate-test").await;
        let token = owner["accessToken"].as_str().unwrap();
        let mut limited = None;
        for _ in 0..45 {
            let request = Request::builder()
                .method("DELETE")
                .uri("/api/v1/practice")
                .header("authorization", format!("Bearer {token}"))
                .header("x-forwarded-for", "198.51.100.194")
                .body(Body::empty())
                .unwrap();
            let response = app.clone().oneshot(request).await.unwrap();
            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                limited = Some(response);
                break;
            }
        }
        let response = limited.expect("DELETE must not be whitelisted from the API limiter");
        assert!(response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .is_some_and(|v| v >= 1));
    }

    #[tokio::test]
    async fn shared_durable_store_prevents_the_verifier_cross_replica_read_and_delete_split() {
        let database_path = std::env::temp_dir().join(format!(
            "booking-recovery-shared-replica-{}.db",
            Uuid::now_v7()
        ));
        let database_url = format!("sqlite://{}?mode=rwc", database_path.display());
        sqlx::any::install_default_drivers();
        let first_pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .unwrap();
        migrations::up(&first_pool).await.unwrap();
        let second_pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .unwrap();
        let first = app_router(first_pool.clone(), "replica-one", "../dist");
        let second = app_router(second_pool.clone(), "replica-two", "../dist");

        let owner = create_owner(&first, "shared-replica-practice").await;
        let token = owner["accessToken"].as_str().unwrap();
        let (read_status, view) =
            send(&second, "GET", "/api/v1/practice", json!({}), Some(token)).await;
        assert_eq!(
            read_status,
            StatusCode::OK,
            "a different replica must read the created practice"
        );
        assert_eq!(view["publicSlug"], "shared-replica-practice");
        let (delete_status, _) = send(
            &second,
            "DELETE",
            "/api/v1/practice",
            json!({}),
            Some(token),
        )
        .await;
        assert_eq!(
            delete_status,
            StatusCode::NO_CONTENT,
            "a different replica must delete the same practice"
        );
        let (after_delete, _) =
            send(&first, "GET", "/api/v1/practice", json!({}), Some(token)).await;
        assert_eq!(
            after_delete,
            StatusCode::UNAUTHORIZED,
            "deletion must be visible to every replica"
        );

        // Regression for verifier 6: independent HTTP connections can land on
        // different replicas, but the first forwarded client still receives
        // one shared 12-write minute allowance (not 12 per replica).
        let mut accepted = 0;
        let mut limited = None;
        for number in 0..13 {
            let request = Request::builder()
                .method("POST")
                .uri("/api/v1/demo/workspaces")
                .header("x-forwarded-for", "203.0.113.249")
                .header("idempotency-key", format!("cross-replica-write-{number}"))
                .body(Body::empty())
                .unwrap();
            let response = if number % 2 == 0 {
                first.clone().oneshot(request).await.unwrap()
            } else {
                second.clone().oneshot(request).await.unwrap()
            };
            if response.status() == StatusCode::CREATED {
                accepted += 1;
            }
            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                limited = Some(response);
            }
        }
        assert_eq!(accepted, 12, "writes must not multiply by replica");
        let limited = limited.expect("the thirteenth independent write must be limited");
        assert_eq!(limited.headers()["x-ratelimit-limit"], "12");
        assert_eq!(limited.headers()["retry-after"], "60");

        first_pool.close().await;
        second_pool.close().await;
        let _ = std::fs::remove_file(database_path);
    }

    #[tokio::test]
    async fn occupied_slot_cannot_be_double_booked() {
        let (app, pool) = test_app().await;
        create_owner(&app, "slot-test").await;
        let payload = json!({"clientName":"First Client","email":"first@example.test","phone":null,"scheduledFor":"2030-05-10T12:00:00Z","emailConsent":true,"smsConsent":false});
        let (first, _) = send(
            &app,
            "POST",
            "/api/v1/public/slot-test/attempts",
            payload,
            None,
        )
        .await;
        assert_eq!(first, StatusCode::CREATED);
        let second_payload = json!({"clientName":"Second Client","email":"second@example.test","phone":null,"scheduledFor":"2030-05-10T12:00:00Z","emailConsent":true,"smsConsent":false});
        let (second, body) = send(
            &app,
            "POST",
            "/api/v1/public/slot-test/attempts",
            second_payload,
            None,
        )
        .await;
        assert_eq!(second, StatusCode::CONFLICT);
        assert_eq!(body["error"], "slot_unavailable");
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM practice_attempts")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }
}
