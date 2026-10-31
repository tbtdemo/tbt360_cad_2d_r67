CREATE TABLE IF NOT EXISTS bim_worksets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    floor_id uuid NULL REFERENCES bim_floors(id),
    discipline text NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    owner_account_id uuid NOT NULL,
    active boolean NOT NULL DEFAULT true,
    UNIQUE(project_id, code)
);

CREATE TABLE IF NOT EXISTS bim_model_revisions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id uuid NOT NULL REFERENCES bim_projects(id) ON DELETE CASCADE,
    workset_id uuid NOT NULL REFERENCES bim_worksets(id) ON DELETE CASCADE,
    revision_no bigint NOT NULL,
    state text NOT NULL CHECK (state IN ('WORKING','REVIEW','PUBLISHED','SUPERSEDED')),
    created_by uuid NOT NULL,
    reviewed_by uuid NULL,
    published_by uuid NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    published_at timestamptz NULL,
    UNIQUE(workset_id, revision_no)
);

CREATE TABLE IF NOT EXISTS bim_consumed_revisions (
    consumer_workset_id uuid NOT NULL REFERENCES bim_worksets(id) ON DELETE CASCADE,
    source_workset_id uuid NOT NULL REFERENCES bim_worksets(id) ON DELETE CASCADE,
    source_revision_id uuid NOT NULL REFERENCES bim_model_revisions(id) ON DELETE CASCADE,
    consumed_by uuid NOT NULL,
    consumed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(consumer_workset_id, source_workset_id)
);
