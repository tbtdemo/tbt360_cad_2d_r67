CREATE TABLE IF NOT EXISTS bim_buildings (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    code text NOT NULL,
    name text NOT NULL,
    origin_x_m double precision NOT NULL DEFAULT 0,
    origin_y_m double precision NOT NULL DEFAULT 0,
    origin_z_m double precision NOT NULL DEFAULT 0,
    rotation_deg double precision NOT NULL DEFAULT 0,
    UNIQUE(project_id, code)
);

CREATE TABLE IF NOT EXISTS bim_zones (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    building_id uuid NOT NULL REFERENCES bim_buildings(id) ON DELETE CASCADE,
    code text NOT NULL,
    name text NOT NULL,
    origin_x_m double precision NOT NULL DEFAULT 0,
    origin_y_m double precision NOT NULL DEFAULT 0,
    origin_z_m double precision NOT NULL DEFAULT 0,
    rotation_deg double precision NOT NULL DEFAULT 0,
    UNIQUE(building_id, code)
);

ALTER TABLE bim_floors ADD COLUMN IF NOT EXISTS building_id uuid NULL REFERENCES bim_buildings(id);
ALTER TABLE bim_floors ADD COLUMN IF NOT EXISTS zone_id uuid NULL REFERENCES bim_zones(id);

ALTER TABLE bim_projects ADD COLUMN IF NOT EXISTS survey_x_m double precision NOT NULL DEFAULT 0;
ALTER TABLE bim_projects ADD COLUMN IF NOT EXISTS survey_y_m double precision NOT NULL DEFAULT 0;
ALTER TABLE bim_projects ADD COLUMN IF NOT EXISTS survey_z_m double precision NOT NULL DEFAULT 0;
ALTER TABLE bim_projects ADD COLUMN IF NOT EXISTS project_base_x_m double precision NOT NULL DEFAULT 0;
ALTER TABLE bim_projects ADD COLUMN IF NOT EXISTS project_base_y_m double precision NOT NULL DEFAULT 0;
ALTER TABLE bim_projects ADD COLUMN IF NOT EXISTS project_base_z_m double precision NOT NULL DEFAULT 0;
ALTER TABLE bim_projects ADD COLUMN IF NOT EXISTS true_north_deg double precision NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS ix_bim_floors_building_zone ON bim_floors(project_id, building_id, zone_id, sort_order);
