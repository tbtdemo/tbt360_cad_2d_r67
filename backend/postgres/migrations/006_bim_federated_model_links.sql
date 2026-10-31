CREATE TABLE IF NOT EXISTS bim_model_links (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    source_model_id uuid NOT NULL,
    source_type text NOT NULL CHECK (source_type IN ('NATIVE_CAD','DXF','DWG','IFC','EXTERNAL_REFERENCE')),
    discipline text NOT NULL,
    owner_account_id uuid NOT NULL,
    building_id uuid NULL REFERENCES bim_buildings(id),
    zone_id uuid NULL REFERENCES bim_zones(id),
    floor_id uuid NULL REFERENCES bim_floors(id),
    published_revision bigint NULL,
    translate_x_m double precision NOT NULL DEFAULT 0,
    translate_y_m double precision NOT NULL DEFAULT 0,
    translate_z_m double precision NOT NULL DEFAULT 0,
    rotation_deg double precision NOT NULL DEFAULT 0,
    scale double precision NOT NULL DEFAULT 1 CHECK (scale > 0),
    opacity real NOT NULL DEFAULT 0.35 CHECK (opacity > 0 AND opacity <= 1),
    state text NOT NULL CHECK (state IN ('UNLOADED','LOADING','LOADED','MISSING','STALE','CONFLICT')),
    read_only boolean NOT NULL DEFAULT true,
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_bim_model_links_scope ON bim_model_links(project_id, building_id, zone_id, floor_id, discipline);
CREATE INDEX IF NOT EXISTS ix_bim_model_links_revision ON bim_model_links(project_id, source_model_id, published_revision);
