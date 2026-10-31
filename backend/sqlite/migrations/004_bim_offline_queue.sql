CREATE TABLE IF NOT EXISTS local_offline_queue (
    local_sequence INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL,
    change_set_id TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('LOCAL','PENDING','SYNCED','CONFLICT','REJECTED')),
    retry_count INTEGER NOT NULL DEFAULT 0,
    next_retry_at TEXT NULL,
    last_error TEXT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS ix_local_offline_queue_pending ON local_offline_queue(project_id, state, local_sequence);

ALTER TABLE local_sync_checkpoint ADD COLUMN retry_cursor INTEGER NOT NULL DEFAULT 0;
ALTER TABLE local_sync_checkpoint ADD COLUMN connectivity_state TEXT NOT NULL DEFAULT 'OFFLINE';
