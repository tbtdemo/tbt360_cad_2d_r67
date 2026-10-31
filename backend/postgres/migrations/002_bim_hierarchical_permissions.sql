CREATE TABLE IF NOT EXISTS bim_access_grants (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    viewer_account_id uuid NOT NULL,
    source_owner_account_id uuid NULL,
    building_id uuid NULL REFERENCES bim_buildings(id) ON DELETE CASCADE,
    zone_id uuid NULL REFERENCES bim_zones(id) ON DELETE CASCADE,
    floor_id uuid NULL REFERENCES bim_floors(id) ON DELETE CASCADE,
    discipline text NULL,
    workset_id uuid NULL,
    can_view boolean NOT NULL DEFAULT false,
    can_edit boolean NOT NULL DEFAULT false,
    can_publish boolean NOT NULL DEFAULT false,
    deny boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_bim_access_grants_lookup ON bim_access_grants(project_id, viewer_account_id, source_owner_account_id, floor_id, discipline);
