-- CAD 360 Viet Nam - TBT
-- Local offline BIM cache schema (SQLite)

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS local_project (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    owner_account_id TEXT NOT NULL,
    origin_x_m REAL NOT NULL DEFAULT 0,
    origin_y_m REAL NOT NULL DEFAULT 0,
    origin_z_m REAL NOT NULL DEFAULT 0,
    rotation_deg REAL NOT NULL DEFAULT 0,
    remote_revision INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS local_floor (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    elevation_m REAL NOT NULL,
    height_m REAL NOT NULL,
    FOREIGN KEY(project_id) REFERENCES local_project(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS local_permission (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    floor_id TEXT NULL,
    discipline TEXT NULL,
    can_view INTEGER NOT NULL DEFAULT 1,
    can_edit INTEGER NOT NULL DEFAULT 0,
    expires_at TEXT NULL,
    FOREIGN KEY(project_id) REFERENCES local_project(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS local_design_object (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    source_handle TEXT NOT NULL,
    floor_id TEXT NOT NULL,
    discipline TEXT NOT NULL,
    owner_account_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    geometry_payload TEXT NOT NULL DEFAULT '{}',
    local_x_m REAL NOT NULL DEFAULT 0,
    local_y_m REAL NOT NULL DEFAULT 0,
    local_z_m REAL NOT NULL DEFAULT 0,
    revision INTEGER NOT NULL DEFAULT 1,
    deleted INTEGER NOT NULL DEFAULT 0,
    dirty INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    UNIQUE(project_id, source_handle),
    FOREIGN KEY(project_id) REFERENCES local_project(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_local_design_object_scope ON local_design_object(project_id, floor_id, discipline, owner_account_id, deleted);
CREATE INDEX IF NOT EXISTS ix_local_design_object_dirty ON local_design_object(project_id, dirty);

CREATE TABLE IF NOT EXISTS local_change_set (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    base_project_revision INTEGER NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    synced_at TEXT NULL,
    FOREIGN KEY(project_id) REFERENCES local_project(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS local_change_item (
    id TEXT PRIMARY KEY,
    change_set_id TEXT NOT NULL,
    source_handle TEXT NOT NULL,
    operation TEXT NOT NULL,
    base_revision INTEGER NOT NULL,
    local_revision INTEGER NOT NULL,
    payload TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY(change_set_id) REFERENCES local_change_set(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS local_view_preset (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    layout INTEGER NOT NULL,
    slots_json TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(project_id) REFERENCES local_project(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS local_scope_progress (
    project_id TEXT NOT NULL,
    floor_id TEXT NOT NULL,
    discipline TEXT NOT NULL,
    percent INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    updated_by TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY(project_id, floor_id, discipline),
    FOREIGN KEY(project_id) REFERENCES local_project(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS local_coordination_issue (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    floor_id TEXT NULL,
    source_discipline TEXT NOT NULL,
    target_discipline TEXT NOT NULL,
    source_handle TEXT NULL,
    created_by TEXT NOT NULL,
    assigned_to TEXT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    world_x_m REAL NULL,
    world_y_m REAL NULL,
    world_z_m REAL NULL,
    dirty INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL,
    FOREIGN KEY(project_id) REFERENCES local_project(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS local_sync_checkpoint (
    project_id TEXT PRIMARY KEY,
    remote_revision INTEGER NOT NULL DEFAULT 0,
    last_pull_at TEXT NULL,
    last_push_at TEXT NULL,
    last_error TEXT NULL,
    FOREIGN KEY(project_id) REFERENCES local_project(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS local_auth_cache (
    account_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    email TEXT NULL,
    authorization_snapshot TEXT NOT NULL DEFAULT '{}',
    authorization_expires_at TEXT NULL,
    last_online_login_at TEXT NULL
);


CREATE TABLE IF NOT EXISTS local_audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,
    sequence_no INTEGER NOT NULL,
    account_id TEXT NOT NULL,
    device_id TEXT NULL,
    action TEXT NOT NULL,
    floor_id TEXT NULL,
    discipline TEXT NULL,
    source_handle TEXT NULL,
    detail_json TEXT NOT NULL DEFAULT '{}',
    synced INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    UNIQUE(project_id, sequence_no),
    FOREIGN KEY(project_id) REFERENCES local_project(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_local_audit_unsynced ON local_audit_log(project_id, synced, sequence_no);
