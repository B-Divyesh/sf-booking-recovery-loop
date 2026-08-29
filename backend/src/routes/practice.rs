use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, SecondsFormat, Utc};
use hmac::{Hmac, Mac};
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
pub(crate) struct CompletePaymentInput {
    license: String,
}

#[derive(Deserialize)]
struct BillingCheckoutResponse {
    checkout_url: String,
    intent_id: String,
}

#[derive(Deserialize)]
struct BillingVerifyResponse {
    valid: bool,
    reason: String,
}

#[derive(Deserialize)]
struct BillingProductList {
    data: Vec<BillingProduct>,
}

#[derive(Deserialize)]
struct BillingProduct {
    slug: String,
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
    checkout_provider: &'static str,
    delivery_connected: bool,
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
    checkout_provider: &'static str,
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
    payment: Option<PaymentSessionView>,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
struct PaymentSessionView {
    provider: String,
    provider_intent_id: String,
    status: String,
    created_at: i64,
    verified_at: Option<i64>,
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
    validate_practice(&input)?;
    let owner_oid = owner_oid(&state, &headers).await?;
    let legacy_storage_token = random_token("retired")?;
    let receipt_token = random_token("receipt")?;
    let id = Uuid::now_v7().to_string();
    sqlx::query(
        "INSERT INTO practices (id, owner_oid, access_token_hash, receipt_token_hash, public_slug, name, timezone, \
         service_name, duration_minutes, deposit_cents, currency, payment_url, delivery_webhook_url, created_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
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
    // Legacy columns stay empty so existing databases can migrate without a
    // destructive table rewrite. Production destinations now come only from
    // the server-owned integration configuration.
    .bind("")
    .bind("")
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
    sqlx::query("INSERT INTO practice_entitlements (practice_id, provider, state, verified_at) VALUES ($1, 'sociobot_dodo', 'unknown', $2)")
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
        checkout_provider: "Sociobot / Dodo",
        consent_wording: CONSENT_WORDING,
    }))
}

pub(crate) async fn integration_status(State(state): State<AppState>) -> Json<Value> {
    // A non-empty slug is only configuration, not proof that Sociobot has the
    // product. The public product registry is the smallest non-mutating check
    // available before a real booking asks it to create a checkout.
    let billing_configured = registered_billing_product(&state).await;
    Json(json!({
        "delivery": {
            "configured": state.integrations.delivery_ready(),
            "requestAuthentication": "bearer",
            "callbackAuthentication": "hmac-sha256"
        },
        "billing": {
            "configured": billing_configured,
            "provider": "sociobot_dodo",
            "productSlug": state.integrations.billing_product_slug
        }
    }))
}

async fn registered_billing_product(state: &AppState) -> bool {
    if state.integrations.billing_product_slug.is_empty() {
        return false;
    }
    let endpoint = format!(
        "{}/products",
        state.integrations.billing_base_url.trim_end_matches('/')
    );
    let Ok(response) = state.http.get(endpoint).send().await else {
        return false;
    };
    if !response.status().is_success() {
        return false;
    }
    let Ok(products) = response.json::<BillingProductList>().await else {
        return false;
    };
    products
        .data
        .into_iter()
        .any(|product| product.slug == state.integrations.billing_product_slug)
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
         VALUES ($1, $2, $3, $4, $5, $6, 'awaiting_deposit', $7, $8, $9, $10, $11)",
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
    sqlx::query("INSERT INTO practice_scheduled_jobs (id, practice_id, attempt_id, kind, due_at, created_at) VALUES ($1, $2, $3, 'abandoned_recovery', $4, $5)")
        .bind(Uuid::now_v7().to_string()).bind(&practice.id).bind(&id)
        .bind(now + 15 * 60).bind(now).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    let (checkout_url, intent_id) =
        match create_owned_checkout(&state, &practice, &id, input.email.as_deref()).await {
            Ok(checkout) => checkout,
            Err(error) => {
                // The attempt and queued recovery form one product action with its
                // hosted checkout. Do not strand an occupied slot when billing is
                // unavailable; the client can safely submit again.
                sqlx::query("DELETE FROM practice_attempts WHERE id = $1 AND practice_id = $2")
                    .bind(&id)
                    .bind(&practice.id)
                    .execute(&state.pool)
                    .await
                    .map_err(|_| ApiError::internal())?;
                return Err(error);
            }
        };
    let payment_session = sqlx::query(
        "INSERT INTO practice_payment_sessions (id, practice_id, attempt_id, provider, provider_intent_id, checkout_url, status, created_at) \
         VALUES ($1, $2, $3, 'sociobot_dodo', $4, $5, 'pending', $6)",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(&practice.id)
    .bind(&id)
    .bind(&intent_id)
    .bind(&checkout_url)
    .bind(now)
    .execute(&state.pool)
    .await;
    if payment_session.is_err() {
        sqlx::query("DELETE FROM practice_attempts WHERE id = $1 AND practice_id = $2")
            .bind(&id)
            .bind(&practice.id)
            .execute(&state.pool)
            .await
            .map_err(|_| ApiError::internal())?;
        return Err(ApiError::internal());
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "attemptId": id,
            "checkoutUrl": checkout_url,
            "checkoutIntentId": intent_id,
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
         WHERE id = $1 AND practice_id = $2",
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
    if !state.integrations.delivery_ready() && practice.delivery_webhook_url.is_empty() {
        return Err(ApiError::conflict(
            "delivery_not_connected",
            "The server delivery provider is not configured.",
        ));
    }
    let response = state
        .http
        .post(delivery_target(&state, &practice.delivery_webhook_url)?)
        .header("authorization", delivery_authorization(&state)?)
        .header(
            "idempotency-key",
            format!("connection-test:{}", practice.id),
        )
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
         AND j.due_at <= $1 AND p.deletion_requested_at IS NULL ORDER BY j.due_at LIMIT 32",
    )
    .bind(now)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;

    for (job_id, attempt_id, kind, practice_id) in jobs {
        let claimed = sqlx::query("UPDATE practice_scheduled_jobs SET status = 'processing', attempts = attempts + 1, last_error = NULL WHERE id = $1 AND status IN ('queued', 'failed')")
            .bind(&job_id).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
        if claimed.rows_affected() != 1 {
            continue;
        }
        let result = deliver_scheduled_job(state, &practice_id, &attempt_id, &kind).await;
        match result {
            Ok(()) => {
                sqlx::query("UPDATE practice_scheduled_jobs SET status = 'sent', completed_at = $1 WHERE id = $2")
                    .bind(now).bind(&job_id).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
            }
            Err(error) if error.code == "consent_required" => {
                sqlx::query("UPDATE practice_scheduled_jobs SET status = 'stopped', completed_at = $1, last_error = $2 WHERE id = $3")
                    .bind(now).bind(error.message).bind(&job_id).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
            }
            Err(error) => {
                // A failed provider call is retried in five minutes. The
                // error remains visible to the owner instead of disappearing.
                sqlx::query("UPDATE practice_scheduled_jobs SET status = 'failed', due_at = $1, last_error = $2 WHERE id = $3")
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
    let practice: PracticeRow = sqlx::query_as("SELECT id, public_slug, name, timezone, service_name, duration_minutes, deposit_cents, currency, delivery_webhook_url FROM practices WHERE id = $1 AND deletion_requested_at IS NULL")
        .bind(practice_id).fetch_optional(&state.pool).await.map_err(|_| ApiError::internal())?.ok_or_else(ApiError::not_found)?;
    let attempt: AttemptRow = sqlx::query_as("SELECT id, client_name_encrypted, email_encrypted, phone_encrypted, scheduled_for, state, email_consent, sms_consent, consent_wording, consent_recorded_at FROM practice_attempts WHERE id = $1 AND practice_id = $2")
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
    if !state.integrations.delivery_ready() && practice.delivery_webhook_url.is_empty() {
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
    let idempotency_key = format!("{}:{}:{}", attempt.id, purpose.replace(' ', "-"), channel);
    let response = state.http.post(delivery_target(state, &practice.delivery_webhook_url)?)
        .header("authorization", delivery_authorization(state)?)
        .header("idempotency-key", &idempotency_key)
        .json(&json!({
        "attemptId": attempt.id, "channel": channel, "to": target,
        "template": if purpose.contains("reminder") { "Your session reminder" } else { "Complete your booking" },
        "purpose": purpose, "replyToPractice": practice.name,
        "receiptCallback": format!("{}/api/v1/provider/{}/receipts", state.integrations.public_base_url.trim_end_matches('/'), practice.id)
    })).send().await.map_err(|_| ApiError::bad_gateway("delivery_unavailable", "The delivery service did not answer. Nothing was marked as sent."))?;
    if !response.status().is_success() {
        return Err(ApiError::bad_gateway(
            "delivery_rejected",
            "The delivery service rejected the message. Check the connection and try again.",
        ));
    }
    let event_id = response
        .headers()
        .get("x-provider-message-id")
        .and_then(|value| value.to_str().ok())
        .map(clean_detail)
        .filter(|value| !value.is_empty())
        .unwrap_or(idempotency_key);
    sqlx::query("INSERT INTO practice_delivery_events (id, practice_id, attempt_id, channel, status, detail, provider_event_id, occurred_at) VALUES ($1, $2, $3, $4, 'accepted', $5, $6, $7) ON CONFLICT DO NOTHING")
        .bind(Uuid::now_v7().to_string()).bind(&practice.id).bind(&attempt.id).bind(channel)
        .bind(format!("Delivery service accepted the {purpose}.")).bind(&event_id).bind(Utc::now().timestamp())
        .execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    sqlx::query(
        "UPDATE practice_attempts SET state = 'recovery_due' WHERE id = $1 AND practice_id = $2",
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
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    authenticate_provider_callback(&state, &headers, &practice_id, &body).await?;
    let input: ReceiptInput = serde_json::from_slice(&body).map_err(|_| {
        ApiError::bad_request("invalid_receipt", "Send a valid JSON delivery receipt.")
    })?;
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
    let payload_digest = hash(std::str::from_utf8(&body).unwrap_or("invalid-utf8"));
    let callback = sqlx::query("INSERT INTO provider_callback_receipts (provider_event_id, practice_id, payload_digest, authenticated_at) SELECT $1, id, $2, $3 FROM practices WHERE id = $4 AND deletion_requested_at IS NULL ON CONFLICT DO NOTHING")
        .bind(&input.provider_event_id).bind(&payload_digest).bind(Utc::now().timestamp()).bind(&practice_id)
        .execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    if callback.rows_affected() == 0 {
        let existing: Option<String> = sqlx::query_scalar("SELECT payload_digest FROM provider_callback_receipts WHERE provider_event_id = $1 AND practice_id = $2")
            .bind(&input.provider_event_id).bind(&practice_id).fetch_optional(&state.pool).await.map_err(|_| ApiError::internal())?;
        if existing.as_deref() != Some(payload_digest.as_str()) {
            return Err(ApiError::conflict(
                "callback_replay_mismatch",
                "That provider event ID was already used with different data.",
            ));
        }
    }
    let inserted = sqlx::query("INSERT INTO practice_delivery_events (id, practice_id, attempt_id, channel, status, detail, provider_event_id, occurred_at) SELECT $1, $2, id, $3, $4, $5, $6, $7 FROM practice_attempts WHERE id = $8 AND practice_id = $9 ON CONFLICT DO NOTHING")
        .bind(Uuid::now_v7().to_string()).bind(&practice_id).bind(&input.channel).bind(&input.status).bind(clean_detail(&input.detail)).bind(&input.provider_event_id).bind(occurred).bind(&input.attempt_id).bind(&practice_id)
        .execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    if inserted.rows_affected() == 0 {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM practice_delivery_events WHERE provider_event_id = $1",
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
            "UPDATE practice_attempts SET state = 'recovered' WHERE id = $1 AND practice_id = $2",
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
             JOIN practices p ON p.id = a.practice_id WHERE a.id = $1 AND a.practice_id = $2 \
             AND a.sms_consent = 1 AND a.phone_encrypted IS NOT NULL",
        )
        .bind(&input.attempt_id)
        .bind(&practice_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
        if let Some((phone, test_webhook, practice_name)) = fallback {
            let response = state
                .http
                .post(delivery_target(&state, &test_webhook)?)
                .header("authorization", delivery_authorization(&state)?)
                .header("idempotency-key", format!("{}:email-bounce:sms", input.attempt_id))
                .json(&json!({
                    "attemptId": input.attempt_id, "channel": "sms", "to": decrypt(&state, &phone)?,
                    "template": "Complete your booking", "replyToPractice": practice_name,
                    "receiptCallback": format!("{}/api/v1/provider/{}/receipts", state.integrations.public_base_url.trim_end_matches('/'), practice_id)
                }))
                .send()
                .await;
            if response.is_ok_and(|value| value.status().is_success()) {
                sqlx::query("INSERT INTO practice_delivery_events (id, practice_id, attempt_id, channel, status, detail, provider_event_id, occurred_at) VALUES ($1, $2, $3, 'sms', 'accepted', 'Email bounced; the permitted SMS fallback was accepted.', $4, $5) ON CONFLICT DO NOTHING")
                    .bind(Uuid::now_v7().to_string()).bind(&practice_id).bind(&input.attempt_id).bind(format!("fallback:{}", input.provider_event_id)).bind(Utc::now().timestamp())
                    .execute(&state.pool).await.map_err(|_| ApiError::internal())?;
            }
        }
    }
    Ok(Json(
        json!({"recorded": true, "duplicate": inserted.rows_affected() == 0}),
    ))
}

pub(crate) async fn complete_payment(
    State(state): State<AppState>,
    Path(attempt_id): Path<String>,
    Json(input): Json<CompletePaymentInput>,
) -> Result<Json<Value>, ApiError> {
    if input.license.len() < 20 || input.license.len() > 4096 {
        return Err(ApiError::bad_request(
            "invalid_payment",
            "The hosted checkout completion token is not valid.",
        ));
    }
    let (practice_id, intent_id, status): (String, String, String) = sqlx::query_as(
        "SELECT practice_id, provider_intent_id, status FROM practice_payment_sessions WHERE attempt_id = $1",
    )
    .bind(&attempt_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?
    .ok_or_else(ApiError::not_found)?;
    if status == "paid" {
        return Ok(Json(json!({"recorded":true,"duplicate":true})));
    }
    let verification = verify_billing_license(&state, &input.license).await?;
    if !verification.valid {
        return Err(ApiError::bad_request(
            "payment_not_verified",
            format!(
                "Sociobot did not verify this checkout: {}.",
                clean_detail(&verification.reason)
            ),
        ));
    }
    let license_hash = hash(&input.license);
    let now = Utc::now().timestamp();
    let session = sqlx::query("UPDATE practice_payment_sessions SET status = 'paid', license_hash = $1, verified_at = $2 WHERE attempt_id = $3 AND status = 'pending'")
        .bind(&license_hash).bind(now).bind(&attempt_id).execute(&state.pool).await;
    let result = match session {
        Ok(value) => value,
        Err(error) if error.to_string().contains("UNIQUE") => {
            return Err(ApiError::conflict(
                "payment_token_used",
                "That checkout completion was already used for another booking.",
            ))
        }
        Err(_) => return Err(ApiError::internal()),
    };
    if result.rows_affected() != 1 {
        return Err(ApiError::conflict(
            "payment_already_recorded",
            "This booking already has a payment result.",
        ));
    }
    sqlx::query("UPDATE practice_attempts SET state = 'paid', payment_reference = $1 WHERE id = $2 AND practice_id = $3 AND payment_reference IS NULL")
        .bind(&intent_id).bind(&attempt_id).bind(&practice_id).execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    let scheduled_for: i64 = sqlx::query_scalar(
        "SELECT scheduled_for FROM practice_attempts WHERE id = $1 AND practice_id = $2",
    )
    .bind(&attempt_id)
    .bind(&practice_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| ApiError::internal())?;
    sqlx::query("INSERT INTO practice_scheduled_jobs (id, practice_id, attempt_id, kind, due_at, created_at) VALUES ($1, $2, $3, 'session_reminder', $4, $5) ON CONFLICT DO NOTHING")
        .bind(Uuid::now_v7().to_string()).bind(&practice_id).bind(&attempt_id)
        .bind((scheduled_for - 24 * 60 * 60).max(now)).bind(now)
        .execute(&state.pool).await.map_err(|_| ApiError::internal())?;
    Ok(Json(
        json!({"recorded":true,"duplicate":false,"provider":"sociobot_dodo"}),
    ))
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
    let result = sqlx::query("DELETE FROM practices WHERE owner_oid = $1")
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
    let attempts: Vec<AttemptRow> = sqlx::query_as("SELECT id, client_name_encrypted, email_encrypted, phone_encrypted, scheduled_for, state, email_consent, sms_consent, consent_wording, consent_recorded_at FROM practice_attempts WHERE practice_id = $1 ORDER BY created_at DESC")
        .bind(&row.id).fetch_all(&state.pool).await.map_err(|_| ApiError::internal())?;
    let mut views = Vec::with_capacity(attempts.len());
    for attempt in attempts {
        let event_rows = sqlx::query_as::<_, EventRow>("SELECT channel, status, detail, occurred_at FROM practice_delivery_events WHERE attempt_id = $1 ORDER BY occurred_at")
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
        let job_rows = sqlx::query_as::<_, ScheduledJobRow>("SELECT kind, due_at, status, last_error FROM practice_scheduled_jobs WHERE attempt_id = $1 ORDER BY due_at")
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
        let payment = sqlx::query_as::<_, PaymentSessionView>(
            "SELECT provider, provider_intent_id, status, created_at, verified_at FROM practice_payment_sessions WHERE attempt_id = $1",
        )
        .bind(&attempt.id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| ApiError::internal())?;
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
            payment,
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
        checkout_provider: "Sociobot / Dodo",
        delivery_connected: state.integrations.delivery_ready(),
        attempts: views,
    })
}

async fn practice_row_by_owner(state: &AppState, owner_oid: &str) -> Result<PracticeRow, ApiError> {
    sqlx::query_as("SELECT id, public_slug, name, timezone, service_name, duration_minutes, deposit_cents, currency, delivery_webhook_url FROM practices WHERE owner_oid = $1 AND deletion_requested_at IS NULL")
        .bind(owner_oid).fetch_optional(&state.pool).await.map_err(|_| ApiError::internal())?.ok_or_else(ApiError::unauthorized)
}

async fn public_practice(state: &AppState, slug: &str) -> Result<PracticeRow, ApiError> {
    sqlx::query_as("SELECT id, public_slug, name, timezone, service_name, duration_minutes, deposit_cents, currency, delivery_webhook_url FROM practices WHERE public_slug = $1 AND deletion_requested_at IS NULL")
        .bind(slug).fetch_optional(&state.pool).await.map_err(|_| ApiError::internal())?.ok_or_else(ApiError::not_found)
}

fn validate_practice(input: &CreatePractice) -> Result<(), ApiError> {
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
    Ok(())
}

async fn create_owned_checkout(
    state: &AppState,
    practice: &PracticeRow,
    attempt_id: &str,
    email: Option<&str>,
) -> Result<(String, String), ApiError> {
    if cfg!(test) && state.integrations.billing_base_url == "https://api.sociobot.in/api/v1" {
        return Ok((
            format!("https://checkout.dodopayments.com/session/test-{attempt_id}"),
            format!("test-intent-{attempt_id}"),
        ));
    }
    let endpoint = format!(
        "{}/products/{}/checkout",
        state.integrations.billing_base_url.trim_end_matches('/'),
        state.integrations.billing_product_slug
    );
    let return_url = format!(
        "{}/b/{}/complete?attempt={attempt_id}",
        state.integrations.public_base_url.trim_end_matches('/'),
        practice.public_slug
    );
    let response = state
        .http
        .post(endpoint)
        .json(&json!({
            "email": email,
            "reference": attempt_id,
            "return_url": return_url,
            "amount_cents": practice.deposit_cents,
            "currency": practice.currency,
            "description": format!("{} deposit", practice.service_name)
        }))
        .send()
        .await
        .map_err(|_| {
            ApiError::bad_gateway(
                "checkout_unavailable",
                "Sociobot checkout did not answer. Nothing was booked; try again shortly.",
            )
        })?;
    if !response.status().is_success() {
        return Err(ApiError::bad_gateway(
            "checkout_rejected",
            "Sociobot checkout could not create this booking payment. Nothing was booked; try again.",
        ));
    }
    let created: BillingCheckoutResponse = response.json().await.map_err(|_| {
        ApiError::bad_gateway(
            "checkout_invalid_response",
            "Sociobot checkout returned an unreadable response. Nothing was booked; try again.",
        )
    })?;
    let url = reqwest::Url::parse(&created.checkout_url).map_err(|_| {
        ApiError::bad_gateway(
            "checkout_invalid_response",
            "Sociobot checkout returned an invalid payment address.",
        )
    })?;
    let approved = url.scheme() == "https" && url.host_str() == Some("checkout.dodopayments.com");
    let loopback_fixture =
        state.allow_test_delivery_urls && is_loopback_test_url(&created.checkout_url);
    if !approved && !loopback_fixture {
        return Err(ApiError::bad_gateway(
            "checkout_invalid_response",
            "Sociobot checkout returned an unapproved payment address.",
        ));
    }
    if created.intent_id.trim().is_empty() || created.intent_id.len() > 200 {
        return Err(ApiError::bad_gateway(
            "checkout_invalid_response",
            "Sociobot checkout did not return a payment intent.",
        ));
    }
    Ok((created.checkout_url, created.intent_id))
}

async fn verify_billing_license(
    state: &AppState,
    license: &str,
) -> Result<BillingVerifyResponse, ApiError> {
    if cfg!(test) && state.integrations.billing_base_url == "https://api.sociobot.in/api/v1" {
        return Ok(BillingVerifyResponse {
            valid: license.starts_with("test_valid_license_"),
            reason: if license.starts_with("test_valid_license_") {
                "ok"
            } else {
                "invalid"
            }
            .to_owned(),
        });
    }
    let endpoint = format!(
        "{}/products/{}/verify",
        state.integrations.billing_base_url.trim_end_matches('/'),
        state.integrations.billing_product_slug
    );
    let response = state
        .http
        .get(endpoint)
        .query(&[("license", license)])
        .send()
        .await
        .map_err(|_| {
            ApiError::bad_gateway(
                "payment_verification_unavailable",
                "Sociobot could not verify this checkout. Try again shortly.",
            )
        })?;
    if !response.status().is_success() {
        return Err(ApiError::bad_gateway(
            "payment_verification_unavailable",
            "Sociobot could not verify this checkout. Try again shortly.",
        ));
    }
    response.json().await.map_err(|_| {
        ApiError::bad_gateway(
            "payment_verification_unavailable",
            "Sociobot returned an unreadable payment result. Try again shortly.",
        )
    })
}

async fn authenticate_provider_callback(
    state: &AppState,
    headers: &HeaderMap,
    practice_id: &str,
    body: &[u8],
) -> Result<(), ApiError> {
    if let Some(secret) = &state.integrations.delivery_callback_secret {
        let supplied = headers
            .get("x-provider-signature")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("sha256="))
            .and_then(|value| hex::decode(value).ok())
            .ok_or_else(ApiError::unauthorized)?;
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(secret.as_bytes())
            .map_err(|_| ApiError::internal())?;
        mac.update(body);
        return mac
            .verify_slice(&supplied)
            .map_err(|_| ApiError::unauthorized());
    }
    // Compatibility exists only inside compiled test harnesses. Production
    // never accepts the retired per-practice static callback token.
    if state.allow_test_delivery_urls {
        if let Some(token) = headers.get("x-receipt-token").and_then(|v| v.to_str().ok()) {
            let valid: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM practices WHERE id = $1 AND receipt_token_hash = $2 AND deletion_requested_at IS NULL")
                .bind(practice_id).bind(hash(token)).fetch_one(&state.pool).await.map_err(|_| ApiError::internal())?;
            if valid == 1 {
                return Ok(());
            }
        }
    }
    Err(ApiError::unauthorized())
}

/// Owner input is a provider identifier, never a destination URL. This keeps
/// contact data out of an arbitrary server-side request path. The loopback
/// exception is compiled into test-only harnesses and is guarded by an
/// explicit environment opt-in for browser integration tests.
fn delivery_target(state: &AppState, configured: &str) -> Result<String, ApiError> {
    if let Some(url) = &state.integrations.delivery_url {
        validate_delivery_provider_url(url, state.allow_test_delivery_urls)?;
        return Ok(url.clone());
    }
    if state.allow_test_delivery_urls && is_loopback_test_url(configured) {
        return Ok(configured.to_owned());
    }
    Err(ApiError::conflict(
        "delivery_not_connected",
        "Live delivery is not configured for this deployment.",
    ))
}

fn delivery_authorization(state: &AppState) -> Result<String, ApiError> {
    if let Some(token) = &state.integrations.delivery_bearer_token {
        return Ok(format!("Bearer {token}"));
    }
    if state.allow_test_delivery_urls {
        return Ok("Bearer test-provider-credential".to_owned());
    }
    Err(ApiError::conflict(
        "delivery_not_connected",
        "Live delivery credentials are not configured for this deployment.",
    ))
}

fn validate_delivery_provider_url(value: &str, allow_loopback: bool) -> Result<(), ApiError> {
    let url = reqwest::Url::parse(value).map_err(|_| ApiError::internal())?;
    if (url.scheme() == "https" && url.host_str().is_some())
        || (allow_loopback && is_loopback_test_url(value))
    {
        Ok(())
    } else {
        Err(ApiError::internal())
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
        http::{HeaderMap, Request, StatusCode},
        Json,
    };
    use chrono::Utc;
    use hmac::{Hmac, Mac};
    use http_body_util::BodyExt;
    use serde_json::{json, Value};
    use sha2::Sha256;
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

    use crate::{
        app_router, app_router_with_integrations, migrations, AppState, IntegrationConfig,
    };

    async fn test_app() -> (axum::Router, AnyPool) {
        test_app_with_integrations(IntegrationConfig::from_environment()).await
    }

    async fn test_app_with_integrations(
        integrations: IntegrationConfig,
    ) -> (axum::Router, AnyPool) {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        migrations::up(&pool).await.unwrap();
        (
            app_router_with_integrations(pool.clone(), "test", "../dist", [7_u8; 32], integrations),
            pool,
        )
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
            integrations: Arc::new(IntegrationConfig::from_environment()),
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
        sqlx::query("UPDATE practices SET delivery_webhook_url = $1 WHERE id = $2")
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
            "UPDATE practice_attempts SET email_consent = 0, email_encrypted = NULL WHERE id = $1",
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
            sqlx::query_scalar("SELECT status FROM practice_scheduled_jobs WHERE attempt_id = $1")
                .bind(consented["attemptId"].as_str().unwrap())
                .fetch_one(&pool)
                .await
                .unwrap();
        let stopped_status: String =
            sqlx::query_scalar("SELECT status FROM practice_scheduled_jobs WHERE attempt_id = $1")
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
            "SELECT due_at FROM practice_scheduled_jobs WHERE attempt_id = $1 AND kind = 'abandoned_recovery'",
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
        sqlx::query("UPDATE practices SET delivery_webhook_url = $1 WHERE id = $2")
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
        sqlx::query("INSERT INTO practice_delivery_events (id, practice_id, attempt_id, channel, status, detail, provider_event_id, occurred_at) VALUES ($1, $2, $3, 'email', 'accepted', 'Accepted for delivery', $4, $5)")
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
            "checkoutProvider",
            "deliveryConnected",
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
        assert!(
            record.get("payment").is_some(),
            "export must include the booking checkout intent"
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
        assert!(attempt["checkoutUrl"]
            .as_str()
            .unwrap()
            .starts_with("https://checkout.dodopayments.com/session/"));
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
        sqlx::query("UPDATE practices SET delivery_webhook_url = $1 WHERE id = $2")
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
            "SELECT COUNT(*) FROM practice_delivery_events WHERE attempt_id = $1",
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
    async fn configured_delivery_and_sociobot_checkout_fix_the_release_blocker_end_to_end() {
        let sends = Arc::new(AtomicUsize::new(0));
        let checkout_payload = Arc::new(Mutex::new(None::<Value>));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let captured_checkout = checkout_payload.clone();
        let send_count = sends.clone();
        let fixture = axum::Router::new()
            .route(
                "/products",
                axum::routing::get(|| async {
                    Json(json!({
                        "data": [{"slug": "booking-recovery-loop-deposit"}]
                    }))
                }),
            )
            .route(
                "/products/booking-recovery-loop-deposit/checkout",
                axum::routing::post(move |headers: HeaderMap, Json(body): Json<Value>| {
                    let captured_checkout = captured_checkout.clone();
                    async move {
                        assert_eq!(headers.get("content-type").unwrap(), "application/json");
                        *captured_checkout.lock().await = Some(body);
                        let reference = captured_checkout.lock().await.as_ref().unwrap()
                            ["reference"]
                            .as_str()
                            .unwrap()
                            .to_owned();
                        Json(json!({
                            "checkout_url": format!("http://{address}/hosted-checkout/{reference}"),
                            "intent_id": format!("intent-{reference}")
                        }))
                    }
                }),
            )
            .route(
                "/products/booking-recovery-loop-deposit/verify",
                axum::routing::get(|| async {
                    Json(json!({"valid": true, "reason": "ok", "expires_at": null}))
                }),
            )
            .route(
                "/send",
                axum::routing::post(move |headers: HeaderMap, Json(body): Json<Value>| {
                    let send_count = send_count.clone();
                    async move {
                        assert_eq!(
                            headers.get("authorization").unwrap(),
                            "Bearer approved-provider-token"
                        );
                        assert!(matches!(body["channel"].as_str(), Some("email" | "sms")));
                        send_count.fetch_add(1, Ordering::SeqCst);
                        StatusCode::ACCEPTED
                    }
                }),
            );
        tokio::spawn(async move { axum::serve(listener, fixture).await.unwrap() });

        let integrations = IntegrationConfig {
            delivery_url: Some(format!("http://{address}/send")),
            delivery_bearer_token: Some("approved-provider-token".to_owned()),
            delivery_callback_secret: Some("callback-secret-held-by-provider".to_owned()),
            billing_base_url: format!("http://{address}"),
            billing_product_slug: "booking-recovery-loop-deposit".to_owned(),
            public_base_url: "https://booking-recovery-loop.sociobot.in".to_owned(),
        };
        let (app, pool) = test_app_with_integrations(integrations).await;
        let (status, integration_status) =
            send(&app, "GET", "/api/v1/integrations/status", json!({}), None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(integration_status["delivery"]["configured"], true);
        assert_eq!(integration_status["billing"]["provider"], "sociobot_dodo");
        let owner = create_owner(&app, "release-blocker-regression").await;
        assert_eq!(owner["practice"]["deliveryConnected"], true);
        let owner_token = owner["accessToken"].as_str().unwrap();
        let practice_id = owner["practice"]["id"].as_str().unwrap();
        let (_, attempt) = send(
            &app,
            "POST",
            "/api/v1/public/release-blocker-regression/attempts",
            json!({
                "clientName":"Maya Patel", "email":"maya@example.test", "phone":"+447700900123",
                "scheduledFor":"2030-05-10T12:00:00Z", "emailConsent":true, "smsConsent":true
            }),
            None,
        )
        .await;
        let attempt_id = attempt["attemptId"].as_str().unwrap();
        assert_eq!(attempt["checkoutIntentId"], format!("intent-{attempt_id}"));
        assert_eq!(
            attempt["checkoutUrl"],
            format!("http://{address}/hosted-checkout/{attempt_id}")
        );
        let checkout = checkout_payload.lock().await.clone().unwrap();
        assert_eq!(checkout["reference"], attempt_id);
        assert_eq!(checkout["amount_cents"], 3500);
        assert_eq!(checkout["currency"], "GBP");
        assert!(checkout["return_url"]
            .as_str()
            .unwrap()
            .contains("/b/release-blocker-regression/complete?attempt="));

        let (status, recovery) = send(
            &app,
            "POST",
            &format!("/api/v1/practice/attempts/{attempt_id}/recover"),
            json!({}),
            Some(owner_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{recovery}");
        assert_eq!(sends.load(Ordering::SeqCst), 1);

        let receipt_body = json!({
            "attemptId": attempt_id,
            "providerEventId": "credentialed-provider-bounce-1",
            "channel": "email",
            "status": "bounced",
            "detail": "Recipient mailbox rejected the message"
        })
        .to_string();
        let unsigned = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/provider/{practice_id}/receipts"))
            .header("content-type", "application/json")
            .header("x-forwarded-for", "198.51.100.201")
            .body(Body::from(receipt_body.clone()))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(unsigned).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
        let mut mac =
            <Hmac<Sha256> as Mac>::new_from_slice(b"callback-secret-held-by-provider").unwrap();
        mac.update(receipt_body.as_bytes());
        let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        for duplicate in [false, true] {
            let signed = Request::builder()
                .method("POST")
                .uri(format!("/api/v1/provider/{practice_id}/receipts"))
                .header("content-type", "application/json")
                .header("x-provider-signature", &signature)
                .header("x-forwarded-for", "198.51.100.202")
                .body(Body::from(receipt_body.clone()))
                .unwrap();
            let response = app.clone().oneshot(signed).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body: Value =
                serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                    .unwrap();
            assert_eq!(body["duplicate"], duplicate);
        }
        assert_eq!(sends.load(Ordering::SeqCst), 2, "one SMS fallback only");

        let (status, paid) = send(
            &app,
            "POST",
            &format!("/api/v1/public/attempts/{attempt_id}/payments/complete"),
            json!({"license":"provider-signed-license-one-123456789"}),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{paid}");
        let durable: (String, i64, i64) = sqlx::query_as(
            "SELECT status, verified_at, (SELECT COUNT(*) FROM practice_delivery_events WHERE attempt_id = $1) FROM practice_payment_sessions WHERE attempt_id = $1",
        )
        .bind(attempt_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(durable.0, "paid");
        assert!(durable.1 > 0);
        assert_eq!(durable.2, 3);

        let (_, second_attempt) = send(
            &app,
            "POST",
            "/api/v1/public/release-blocker-regression/attempts",
            json!({
                "clientName":"Jordan Lee", "email":"jordan@example.test", "phone":null,
                "scheduledFor":"2030-05-10T13:00:00Z", "emailConsent":true, "smsConsent":false
            }),
            None,
        )
        .await;
        let second_id = second_attempt["attemptId"].as_str().unwrap();
        let (reuse_status, _) = send(
            &app,
            "POST",
            &format!("/api/v1/public/attempts/{second_id}/payments/complete"),
            json!({"license":"provider-signed-license-one-123456789"}),
            None,
        )
        .await;
        assert_eq!(reuse_status, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn hosted_payment_requires_verified_callback_and_is_idempotent() {
        let (app, pool) = test_app().await;
        let owner = create_owner(&app, "payment-test").await;
        let owner_token = owner["accessToken"].as_str().unwrap();
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
        let invalid = json!({"license":"invalid_completion_token_123456789"});
        let (status, _) = send(
            &app,
            "POST",
            &format!("/api/v1/public/attempts/{attempt_id}/payments/complete"),
            invalid,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        let payload = json!({"license":"test_valid_license_payment_123456789"});
        for duplicate in [false, true] {
            let request = Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/public/attempts/{attempt_id}/payments/complete"
                ))
                .header("content-type", "application/json")
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
        let reminders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM practice_scheduled_jobs WHERE attempt_id = $1 AND kind = 'session_reminder' AND status = 'queued'")
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
        let practice_id = owner["practice"]["id"].as_str().unwrap();
        sqlx::query("UPDATE practices SET delivery_webhook_url = $1 WHERE id = $2")
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
        let payload = json!({"license":"test_valid_license_reminder_123456789"});

        for duplicate in [false, true] {
            let request = Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/public/attempts/{attempt_id}/payments/complete"
                ))
                .header("content-type", "application/json")
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

        sqlx::query("UPDATE practice_scheduled_jobs SET due_at = 1 WHERE attempt_id = $1 AND kind = 'session_reminder'")
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
        let status: String = sqlx::query_scalar("SELECT status FROM practice_scheduled_jobs WHERE attempt_id = $1 AND kind = 'session_reminder'")
            .bind(attempt_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "sent");
        let detail: String = sqlx::query_scalar("SELECT detail FROM practice_delivery_events WHERE attempt_id = $1 ORDER BY occurred_at DESC LIMIT 1")
            .bind(attempt_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(detail.contains("automatic session reminder"));
    }

    #[tokio::test]
    async fn owner_supplied_delivery_urls_are_ignored_before_any_server_request() {
        let (app, _) = test_app().await;
        let (status, body) = send(&app, "POST", "/api/v1/practices", json!({
            "name":"North Star Coaching", "publicSlug":"blocked-delivery", "timezone":"Europe/London",
            "serviceName":"Focus session", "durationMinutes":45, "depositCents":3500,
            "currency":"GBP", "paymentUrl":"https://pay.example/session",
            "deliveryWebhookUrl":"https://169.254.169.254/latest/meta-data"
        }), None).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        assert_eq!(body["practice"]["deliveryConnected"], false);
    }

    #[tokio::test]
    async fn integration_status_requires_registered_billing_product_and_never_serializes_server_credentials(
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let fixture = axum::Router::new().route(
            "/products",
            axum::routing::get(|| async {
                Json(json!({"data": [{"slug": "a-different-product"}]}))
            }),
        );
        tokio::spawn(async move { axum::serve(listener, fixture).await.unwrap() });

        let integrations = IntegrationConfig {
            delivery_url: Some("https://relay.example.test/private-send-endpoint".to_owned()),
            delivery_bearer_token: Some("delivery-bearer-secret-never-for-browser".to_owned()),
            delivery_callback_secret: Some("callback-hmac-secret-never-for-browser".to_owned()),
            billing_base_url: format!("http://{address}"),
            billing_product_slug: "booking-recovery-loop-deposit".to_owned(),
            public_base_url: "https://booking-recovery-loop.sociobot.in".to_owned(),
        };
        let (app, _) = test_app_with_integrations(integrations).await;
        let (status, body) =
            send(&app, "GET", "/api/v1/integrations/status", json!({}), None).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["delivery"]["configured"], true);
        assert_eq!(
            body["billing"]["configured"], false,
            "a configured slug is not a billing integration until the provider registry has it"
        );
        let serialized = body.to_string();
        for server_only_value in [
            "https://relay.example.test/private-send-endpoint",
            "delivery-bearer-secret-never-for-browser",
            "callback-hmac-secret-never-for-browser",
            &format!("http://{address}"),
        ] {
            assert!(
                !serialized.contains(server_only_value),
                "browser JSON must not disclose {server_only_value}"
            );
        }
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
