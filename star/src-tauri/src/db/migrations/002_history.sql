CREATE TABLE history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    query TEXT,
    visited_at INTEGER NOT NULL,
    visit_count INTEGER NOT NULL DEFAULT 1
);

CREATE INDEX idx_history_visited_At ON history (visited_at DESC);