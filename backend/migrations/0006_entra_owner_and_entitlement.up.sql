ALTER TABLE practices ADD COLUMN owner_oid TEXT;
CREATE INDEX practices_owner_oid_idx ON practices (owner_oid);

CREATE TABLE practice_entitlements (
    practice_id TEXT PRIMARY KEY NOT NULL REFERENCES practices(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider = 'sociobot_dodo'),
    state TEXT NOT NULL CHECK (state IN ('active', 'past_due', 'cancelled', 'unknown')),
    external_subscription_id TEXT UNIQUE,
    verified_at INTEGER NOT NULL,
    expires_at INTEGER
);
