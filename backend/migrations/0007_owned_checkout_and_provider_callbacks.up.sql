CREATE TABLE practice_payment_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    practice_id TEXT NOT NULL REFERENCES practices(id) ON DELETE CASCADE,
    attempt_id TEXT NOT NULL UNIQUE REFERENCES practice_attempts(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider = 'sociobot_dodo'),
    provider_intent_id TEXT NOT NULL UNIQUE,
    checkout_url TEXT NOT NULL,
    license_hash TEXT UNIQUE,
    status TEXT NOT NULL CHECK (status IN ('pending', 'paid', 'failed')),
    created_at INTEGER NOT NULL,
    verified_at INTEGER
);

CREATE INDEX practice_payment_sessions_practice_idx
    ON practice_payment_sessions (practice_id, created_at);

CREATE TABLE provider_callback_receipts (
    provider_event_id TEXT PRIMARY KEY NOT NULL,
    practice_id TEXT NOT NULL REFERENCES practices(id) ON DELETE CASCADE,
    payload_digest TEXT NOT NULL,
    authenticated_at INTEGER NOT NULL
);
