-- CAD 360 Viet Nam - TBT
-- Preliminary BIM collaboration schema for PostgreSQL.
-- account_id values are issued by TBT.Auth; no direct FK is declared here so Auth can remain a separate service/database.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS bim_projects (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    name text NOT NULL,
    owner_account_id uuid NOT NULL,
    origin_x_m double precision NOT NULL DEFAULT 0,
    origin_y_m double precision NOT NULL DEFAULT 0,
    origin_z_m double precision NOT NULL DEFAULT 0,
    rotation_deg double precision NOT NULL DEFAULT 0,
    revision bigint NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS bim_project_members (
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    account_id uuid NOT NULL,
    role text NOT NULL CHECK (role IN ('OWNER','COORDINATOR','DESIGNER','VIEWER')),
    can_edit boolean NOT NULL DEFAULT false,
    can_view_other_disciplines boolean NOT NULL DEFAULT false,
    can_view_other_floors boolean NOT NULL DEFAULT false,
    joined_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, account_id)
);

CREATE TABLE IF NOT EXISTS bim_floors (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    code text NOT NULL,
    name text NOT NULL,
    sort_order integer NOT NULL,
    elevation_m double precision NOT NULL,
    height_m double precision NOT NULL CHECK (height_m > 0),
    UNIQUE (project_id, code),
    UNIQUE (project_id, sort_order)
);

CREATE TABLE IF NOT EXISTS bim_scope_permissions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    account_id uuid NOT NULL,
    floor_id uuid NULL REFERENCES bim_floors(id) ON DELETE CASCADE,
    discipline text NULL CHECK (discipline IS NULL OR discipline IN ('ARCH','STRUCT','ELEC','PLUMB','HVAC','COORD')),
    can_view boolean NOT NULL DEFAULT true,
    can_edit boolean NOT NULL DEFAULT false,
    UNIQUE NULLS NOT DISTINCT (project_id, account_id, floor_id, discipline)
);

CREATE TABLE IF NOT EXISTS bim_design_objects (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    source_handle text NOT NULL,
    floor_id uuid NOT NULL REFERENCES bim_floors(id),
    discipline text NOT NULL CHECK (discipline IN ('ARCH','STRUCT','ELEC','PLUMB','HVAC','COORD')),
    owner_account_id uuid NOT NULL,
    entity_type text NOT NULL,
    geometry_payload jsonb NOT NULL DEFAULT '{}'::jsonb,
    local_x_m double precision NOT NULL DEFAULT 0,
    local_y_m double precision NOT NULL DEFAULT 0,
    local_z_m double precision NOT NULL DEFAULT 0,
    revision bigint NOT NULL DEFAULT 1,
    deleted boolean NOT NULL DEFAULT false,
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (project_id, source_handle)
);

CREATE INDEX IF NOT EXISTS ix_bim_design_objects_scope ON bim_design_objects(project_id, floor_id, discipline, owner_account_id) WHERE deleted = false;
CREATE INDEX IF NOT EXISTS ix_bim_design_objects_revision ON bim_design_objects(project_id, revision);

CREATE TABLE IF NOT EXISTS bim_change_sets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    account_id uuid NOT NULL,
    device_id text NOT NULL,
    base_project_revision bigint NOT NULL,
    status text NOT NULL CHECK (status IN ('LOCAL','PENDING','SYNCED','CONFLICT','REJECTED')),
    created_at timestamptz NOT NULL DEFAULT now(),
    synced_at timestamptz NULL
);

CREATE TABLE IF NOT EXISTS bim_change_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    change_set_id uuid NOT NULL REFERENCES bim_change_sets(id) ON DELETE CASCADE,
    design_object_id uuid NULL REFERENCES bim_design_objects(id),
    source_handle text NOT NULL,
    operation text NOT NULL CHECK (operation IN ('CREATE','UPDATE','DELETE')),
    base_revision bigint NOT NULL DEFAULT 0,
    local_revision bigint NOT NULL,
    payload jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS ix_bim_change_items_handle ON bim_change_items(source_handle);

CREATE TABLE IF NOT EXISTS bim_object_leases (
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    design_object_id uuid NOT NULL REFERENCES bim_design_objects(id) ON DELETE CASCADE,
    account_id uuid NOT NULL,
    device_id text NOT NULL,
    acquired_revision bigint NOT NULL,
    expires_at timestamptz NOT NULL,
    PRIMARY KEY (project_id, design_object_id)
);

CREATE INDEX IF NOT EXISTS ix_bim_object_leases_expiry ON bim_object_leases(expires_at);

CREATE TABLE IF NOT EXISTS bim_presence (
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    account_id uuid NOT NULL,
    device_id text NOT NULL,
    floor_id uuid NULL REFERENCES bim_floors(id),
    discipline text NULL CHECK (discipline IS NULL OR discipline IN ('ARCH','STRUCT','ELEC','PLUMB','HVAC','COORD')),
    selected_object_id uuid NULL REFERENCES bim_design_objects(id),
    cursor_x_m double precision NULL,
    cursor_y_m double precision NULL,
    cursor_z_m double precision NULL,
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, account_id, device_id)
);

CREATE TABLE IF NOT EXISTS bim_view_presets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    account_id uuid NOT NULL,
    name text NOT NULL,
    layout smallint NOT NULL CHECK (layout BETWEEN 1 AND 4),
    slots jsonb NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (project_id, account_id, name)
);

CREATE TABLE IF NOT EXISTS bim_scope_progress (
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    floor_id uuid NOT NULL REFERENCES bim_floors(id) ON DELETE CASCADE,
    discipline text NOT NULL CHECK (discipline IN ('ARCH','STRUCT','ELEC','PLUMB','HVAC','COORD')),
    percent smallint NOT NULL DEFAULT 0 CHECK (percent BETWEEN 0 AND 100),
    status text NOT NULL CHECK (status IN ('NOT_STARTED','IN_PROGRESS','REVIEW','COMPLETE')),
    updated_by uuid NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, floor_id, discipline)
);

CREATE TABLE IF NOT EXISTS bim_coordination_issues (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    floor_id uuid NULL REFERENCES bim_floors(id),
    source_discipline text NOT NULL,
    target_discipline text NOT NULL,
    design_object_id uuid NULL REFERENCES bim_design_objects(id),
    created_by uuid NOT NULL,
    assigned_to uuid NULL,
    description text NOT NULL,
    status text NOT NULL CHECK (status IN ('OPEN','IN_PROGRESS','RESOLVED','CLOSED')),
    world_x_m double precision NULL,
    world_y_m double precision NULL,
    world_z_m double precision NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_bim_coordination_issue_scope ON bim_coordination_issues(project_id, floor_id, status);


CREATE TABLE IF NOT EXISTS bim_audit_log (
    id bigserial PRIMARY KEY,
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    account_id uuid NOT NULL,
    device_id text NULL,
    action text NOT NULL,
    floor_id uuid NULL REFERENCES bim_floors(id),
    discipline text NULL,
    design_object_id uuid NULL REFERENCES bim_design_objects(id),
    source_handle text NULL,
    base_revision bigint NULL,
    result_revision bigint NULL,
    detail jsonb NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_bim_audit_project_time ON bim_audit_log(project_id, created_at DESC);
CREATE INDEX IF NOT EXISTS ix_bim_audit_object ON bim_audit_log(project_id, design_object_id, created_at DESC);
