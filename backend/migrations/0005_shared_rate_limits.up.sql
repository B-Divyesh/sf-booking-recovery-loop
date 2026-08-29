CREATE TABLE api_rate_windows (
    client_key TEXT NOT NULL,
    window_start INTEGER NOT NULL,
    hits INTEGER NOT NULL,
    PRIMARY KEY (client_key, window_start)
);

CREATE INDEX api_rate_windows_expiry_idx ON api_rate_windows (window_start);
