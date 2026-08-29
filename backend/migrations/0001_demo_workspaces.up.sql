CREATE TABLE demo_workspaces (
    id TEXT PRIMARY KEY NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    idempotency_key TEXT NOT NULL UNIQUE,
    is_demo INTEGER NOT NULL CHECK (is_demo IN (0, 1)),
    practice_name TEXT NOT NULL,
    practice_timezone TEXT NOT NULL,
    service_name TEXT NOT NULL,
    service_duration_minutes INTEGER NOT NULL,
    deposit_cents INTEGER NOT NULL,
    currency TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);

CREATE INDEX demo_workspaces_expiry_idx ON demo_workspaces (expires_at);

CREATE TABLE booking_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES demo_workspaces(id) ON DELETE CASCADE,
    client_name TEXT NOT NULL,
    scheduled_for INTEGER NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('unfinished', 'recovered', 'completed')),
    reason TEXT NOT NULL,
    email_consent INTEGER NOT NULL CHECK (email_consent IN (0, 1)),
    consent_wording TEXT,
    consent_recorded_at INTEGER,
    outcome TEXT
);

CREATE INDEX booking_attempts_workspace_idx ON booking_attempts (workspace_id);

CREATE TABLE outbound_messages (
    id TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES demo_workspaces(id) ON DELETE CASCADE,
    attempt_id TEXT NOT NULL REFERENCES booking_attempts(id) ON DELETE CASCADE,
    idempotency_key TEXT NOT NULL,
    channel TEXT NOT NULL CHECK (channel IN ('email', 'sms')),
    state TEXT NOT NULL CHECK (state IN ('accepted', 'delivered', 'failed')),
    created_at INTEGER NOT NULL,
    UNIQUE (workspace_id, idempotency_key),
    UNIQUE (attempt_id, channel)
);

CREATE TABLE delivery_events (
    id TEXT PRIMARY KEY NOT NULL,
    message_id TEXT NOT NULL REFERENCES outbound_messages(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('accepted', 'delivered', 'bounced', 'failed')),
    detail TEXT NOT NULL,
    occurred_at INTEGER NOT NULL,
    simulated INTEGER NOT NULL CHECK (simulated IN (0, 1))
);

CREATE INDEX delivery_events_message_idx ON delivery_events (message_id, occurred_at);
