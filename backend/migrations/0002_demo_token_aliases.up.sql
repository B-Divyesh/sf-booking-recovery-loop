CREATE TABLE demo_token_aliases (
    token_hash TEXT PRIMARY KEY NOT NULL,
    workspace_id TEXT NOT NULL REFERENCES demo_workspaces(id) ON DELETE CASCADE
);

CREATE INDEX demo_token_aliases_workspace_idx ON demo_token_aliases (workspace_id);

INSERT INTO demo_token_aliases (token_hash, workspace_id)
SELECT token_hash, id FROM demo_workspaces;
