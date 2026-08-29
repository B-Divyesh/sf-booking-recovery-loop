CREATE TABLE practice_scheduled_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    practice_id TEXT NOT NULL REFERENCES practices(id) ON DELETE CASCADE,
    attempt_id TEXT NOT NULL REFERENCES practice_attempts(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('abandoned_recovery', 'session_reminder')),
    due_at INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('queued', 'processing', 'sent', 'stopped', 'failed')) DEFAULT 'queued',
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at INTEGER NOT NULL,
    completed_at INTEGER,
    UNIQUE(attempt_id, kind)
);

CREATE INDEX practice_scheduled_jobs_due_idx ON practice_scheduled_jobs (status, due_at);
