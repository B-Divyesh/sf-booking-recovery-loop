CREATE TABLE practices (
    id TEXT PRIMARY KEY NOT NULL,
    access_token_hash TEXT NOT NULL UNIQUE,
    receipt_token_hash TEXT NOT NULL UNIQUE,
    public_slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    timezone TEXT NOT NULL,
    service_name TEXT NOT NULL,
    duration_minutes INTEGER NOT NULL,
    deposit_cents INTEGER NOT NULL,
    currency TEXT NOT NULL,
    payment_url TEXT NOT NULL,
    delivery_webhook_url TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    deletion_requested_at INTEGER
);

CREATE TABLE practice_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    practice_id TEXT NOT NULL REFERENCES practices(id) ON DELETE CASCADE,
    client_name_encrypted TEXT NOT NULL,
    email_encrypted TEXT,
    phone_encrypted TEXT,
    scheduled_for INTEGER NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('awaiting_deposit', 'paid', 'recovery_due', 'recovered', 'cancelled')),
    email_consent INTEGER NOT NULL CHECK (email_consent IN (0, 1)),
    sms_consent INTEGER NOT NULL CHECK (sms_consent IN (0, 1)),
    consent_wording TEXT NOT NULL,
    consent_recorded_at INTEGER NOT NULL,
    payment_reference TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX practice_attempts_practice_idx ON practice_attempts (practice_id, created_at);
CREATE UNIQUE INDEX practice_attempts_slot_idx ON practice_attempts (practice_id, scheduled_for) WHERE state <> 'cancelled';

CREATE TABLE practice_delivery_events (
    id TEXT PRIMARY KEY NOT NULL,
    practice_id TEXT NOT NULL REFERENCES practices(id) ON DELETE CASCADE,
    attempt_id TEXT NOT NULL REFERENCES practice_attempts(id) ON DELETE CASCADE,
    channel TEXT NOT NULL CHECK (channel IN ('email', 'sms')),
    status TEXT NOT NULL CHECK (status IN ('accepted', 'delivered', 'bounced', 'failed')),
    detail TEXT NOT NULL,
    provider_event_id TEXT NOT NULL UNIQUE,
    occurred_at INTEGER NOT NULL
);

CREATE INDEX practice_delivery_attempt_idx ON practice_delivery_events (attempt_id, occurred_at);
