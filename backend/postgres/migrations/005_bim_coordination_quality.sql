ALTER TABLE bim_coordination_issues ADD COLUMN IF NOT EXISTS severity text NOT NULL DEFAULT 'WARNING'
    CHECK (severity IN ('INFO','WARNING','ERROR','CRITICAL'));
ALTER TABLE bim_coordination_issues ADD COLUMN IF NOT EXISTS workset_id uuid NULL;

ALTER TABLE bim_scope_progress ADD COLUMN IF NOT EXISTS workset_id uuid NULL;
CREATE INDEX IF NOT EXISTS ix_bim_scope_progress_workset ON bim_scope_progress(project_id, floor_id, discipline, workset_id);

ALTER TABLE bim_audit_log ADD COLUMN IF NOT EXISTS workset_id uuid NULL;
ALTER TABLE bim_audit_log ADD COLUMN IF NOT EXISTS issue_id uuid NULL REFERENCES bim_coordination_issues(id);
CREATE INDEX IF NOT EXISTS ix_bim_audit_issue ON bim_audit_log(project_id, issue_id, created_at DESC);
