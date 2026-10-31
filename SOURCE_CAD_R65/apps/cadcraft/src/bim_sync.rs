//! BIM collaboration sync core.
//! Network transport is intentionally abstract so the R65 dependency graph stays pinned.
//! A future TBT Gateway adapter can implement this trait with the project's approved HTTP stack.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteConflict {
    pub source_handle: String,
    pub base_revision: u64,
    pub local_revision: u64,
    pub remote_revision: u64,
    pub remote_updated_by: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PushResult {
    pub accepted_change_set_ids: Vec<u64>,
    pub project_revision: u64,
    pub conflicts: Vec<RemoteConflict>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bootstrap {
    pub project_id: String,
    pub project_revision: u64,
    pub floors: Vec<FloorDto>,
    pub permissions: Vec<PermissionDto>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FloorDto {
    pub id: String,
    pub code: String,
    pub name: String,
    pub elevation_m: f64,
    pub height_m: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionDto {
    pub account_id: String,
    pub floor_id: Option<String>,
    pub discipline: Option<String>,
    pub can_view: bool,
    pub can_edit: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresenceDto {
    pub account_id: String,
    pub floor_id: String,
    pub discipline: String,
    pub selected_object_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseResult {
    pub granted: bool,
    pub owner_account_id: Option<String>,
    pub expires_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeEnvelope {
    pub id: u64,
    pub account_id: String,
    pub device_id: String,
    pub base_project_revision: u64,
    pub items: Vec<ChangeEnvelopeItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeEnvelopeItem {
    pub source_handle: String,
    pub operation: String,
    pub base_revision: u64,
    pub local_revision: u64,
    pub payload_json: String,
}

pub trait CollaborationTransport {
    fn bootstrap(&mut self, project_id: &str) -> Result<Bootstrap, String>;
    fn pull_changes(&mut self, project_id: &str, after_revision: u64) -> Result<(u64, Vec<ChangeEnvelope>), String>;
    fn push_changes(&mut self, project_id: &str, changes: &[ChangeEnvelope]) -> Result<PushResult, String>;
    fn update_presence(&mut self, project_id: &str, presence: &PresenceDto) -> Result<(), String>;
    fn acquire_lease(&mut self, project_id: &str, source_handle: &str, revision: u64) -> Result<LeaseResult, String>;
    fn release_lease(&mut self, project_id: &str, source_handle: &str) -> Result<(), String>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SyncHealth {
    #[default]
    Offline,
    Healthy,
    Pending,
    Conflict,
    Error,
}

#[derive(Clone, Debug, Default)]
pub struct SyncRuntime {
    pub project_revision: u64,
    pub last_pull_revision: u64,
    pub last_error: Option<String>,
    pub health: SyncHealth,
}

impl SyncRuntime {
    pub fn mark_offline(&mut self) {
        self.health = SyncHealth::Offline;
        self.last_error = None;
    }

    pub fn apply_push_result(&mut self, result: &PushResult) {
        self.project_revision = result.project_revision;
        self.health = if result.conflicts.is_empty() { SyncHealth::Healthy } else { SyncHealth::Conflict };
        self.last_error = None;
    }

    pub fn fail(&mut self, message: impl Into<String>) {
        self.health = SyncHealth::Error;
        self.last_error = Some(message.into());
    }
}

/// In-memory transport used by branch tests and UI prototyping.
/// It deliberately models revision conflicts instead of last-write-wins.
#[derive(Default)]
pub struct MemoryTransport {
    pub project_revision: u64,
    pub object_revisions: std::collections::BTreeMap<String, u64>,
    pub presences: std::collections::BTreeMap<String, PresenceDto>,
    pub leases: std::collections::BTreeMap<String, String>,
}

impl CollaborationTransport for MemoryTransport {
    fn bootstrap(&mut self, project_id: &str) -> Result<Bootstrap, String> {
        Ok(Bootstrap { project_id: project_id.to_string(), project_revision: self.project_revision, floors: Vec::new(), permissions: Vec::new() })
    }

    fn pull_changes(&mut self, _project_id: &str, after_revision: u64) -> Result<(u64, Vec<ChangeEnvelope>), String> {
        Ok((self.project_revision.max(after_revision), Vec::new()))
    }

    fn push_changes(&mut self, _project_id: &str, changes: &[ChangeEnvelope]) -> Result<PushResult, String> {
        let mut accepted = Vec::new();
        let mut conflicts = Vec::new();
        for change in changes {
            let mut change_conflict = false;
            for item in &change.items {
                let remote_revision = self.object_revisions.get(&item.source_handle).copied().unwrap_or(0);
                if remote_revision != item.base_revision {
                    conflicts.push(RemoteConflict {
                        source_handle: item.source_handle.clone(),
                        base_revision: item.base_revision,
                        local_revision: item.local_revision,
                        remote_revision,
                        remote_updated_by: "remote-user".into(),
                    });
                    change_conflict = true;
                }
            }
            if change_conflict {
                continue;
            }
            for item in &change.items {
                self.object_revisions.insert(item.source_handle.clone(), item.local_revision);
            }
            accepted.push(change.id);
            self.project_revision += 1;
        }
        Ok(PushResult { accepted_change_set_ids: accepted, project_revision: self.project_revision, conflicts })
    }

    fn update_presence(&mut self, _project_id: &str, presence: &PresenceDto) -> Result<(), String> {
        self.presences.insert(presence.account_id.clone(), presence.clone());
        Ok(())
    }

    fn acquire_lease(&mut self, _project_id: &str, source_handle: &str, _revision: u64) -> Result<LeaseResult, String> {
        match self.leases.get(source_handle) {
            Some(owner) => Ok(LeaseResult { granted: false, owner_account_id: Some(owner.clone()), expires_unix_ms: None }),
            None => {
                self.leases.insert(source_handle.to_string(), "local-user".into());
                Ok(LeaseResult { granted: true, owner_account_id: Some("local-user".into()), expires_unix_ms: None })
            }
        }
    }

    fn release_lease(&mut self, _project_id: &str, source_handle: &str) -> Result<(), String> {
        self.leases.remove(source_handle);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(id: u64, handle: &str, base: u64, local: u64) -> ChangeEnvelope {
        ChangeEnvelope {
            id,
            account_id: "u1".into(),
            device_id: "pc1".into(),
            base_project_revision: 0,
            items: vec![ChangeEnvelopeItem {
                source_handle: handle.into(),
                operation: "UPDATE".into(),
                base_revision: base,
                local_revision: local,
                payload_json: "{}".into(),
            }],
        }
    }

    #[test]
    fn memory_transport_accepts_matching_revision_and_rejects_stale_update() {
        let mut transport = MemoryTransport::default();
        transport.object_revisions.insert("A".into(), 2);
        let ok = transport.push_changes("p1", &[change(1, "A", 2, 3)]).unwrap();
        assert_eq!(ok.accepted_change_set_ids, vec![1]);
        assert!(ok.conflicts.is_empty());
        let stale = transport.push_changes("p1", &[change(2, "A", 2, 4)]).unwrap();
        assert!(stale.accepted_change_set_ids.is_empty());
        assert_eq!(stale.conflicts[0].remote_revision, 3);
    }

    #[test]
    fn lease_denies_second_editor_until_release() {
        let mut transport = MemoryTransport::default();
        assert!(transport.acquire_lease("p1", "A", 1).unwrap().granted);
        assert!(!transport.acquire_lease("p1", "A", 1).unwrap().granted);
        transport.release_lease("p1", "A").unwrap();
        assert!(transport.acquire_lease("p1", "A", 1).unwrap().granted);
    }

    #[test]
    fn sync_runtime_reports_conflict_state() {
        let mut runtime = SyncRuntime::default();
        runtime.apply_push_result(&PushResult {
            accepted_change_set_ids: vec![],
            project_revision: 5,
            conflicts: vec![RemoteConflict {
                source_handle: "A".into(),
                base_revision: 1,
                local_revision: 2,
                remote_revision: 3,
                remote_updated_by: "u2".into(),
            }],
        });
        assert_eq!(runtime.health, SyncHealth::Conflict);
        assert_eq!(runtime.project_revision, 5);
    }
}
