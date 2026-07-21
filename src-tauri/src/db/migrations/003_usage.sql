CREATE TABLE usage_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    used_at INTEGER NOT NULL
);

CREATE INDEX idx_usage_used_at ON usage_log (used_at DESC);