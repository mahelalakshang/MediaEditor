CREATE TABLE media_assets (
    id TEXT PRIMARY KEY,
    original_filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE media_renditions (
    id TEXT PRIMARY KEY,
    asset_id TEXT NOT NULL REFERENCES media_assets(id),
    rendition_type TEXT NOT NULL,
    path TEXT NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    score REAL,
    is_selected INTEGER NOT NULL DEFAULT 0,
    timestamp_seconds REAL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_media_renditions_asset_id ON media_renditions(asset_id);
