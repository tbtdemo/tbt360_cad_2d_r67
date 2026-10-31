//! Stable BIM API DTO contract shared by desktop and future TBT Gateway adapter.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeDto {
    pub project_id: String,
    pub building_id: Option<String>,
    pub zone_id: Option<String>,
    pub floor_id: Option<String>,
    pub discipline: Option<String>,
    pub workset_id: Option<String>,
    pub owner_account_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootstrapRequest {
    pub project_id: String,
    pub device_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootstrapResponse {
    pub project_id: String,
    pub project_revision: u64,
    pub account_id: String,
    pub scopes: Vec<ScopeDto>,
    pub published_revision_ids: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeOperationDto { Create, Update, Delete }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeItemDto {
    pub object_id: String,
    pub source_handle: String,
    pub scope: ScopeDto,
    pub operation: ChangeOperationDto,
    pub base_revision: u64,
    pub local_revision: u64,
    pub payload_json: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PushChangeSetRequest {
    pub project_id: String,
    pub account_id: String,
    pub device_id: String,
    pub client_change_set_id: String,
    pub base_project_revision: u64,
    pub items: Vec<ChangeItemDto>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedItemDto {
    pub object_id: String,
    pub server_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConflictItemDto {
    pub object_id: String,
    pub source_handle: String,
    pub base_revision: u64,
    pub local_revision: u64,
    pub remote_revision: u64,
    pub remote_updated_by: String,
    pub remote_payload_json: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PushChangeSetResponse {
    pub client_change_set_id: String,
    pub project_revision: u64,
    pub accepted: Vec<AcceptedItemDto>,
    pub conflicts: Vec<ConflictItemDto>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishRevisionRequest {
    pub project_id: String,
    pub workset_id: String,
    pub revision_id: String,
    pub account_id: String,
    pub expected_state: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishRevisionResponse {
    pub revision_id: String,
    pub published_revision_no: u64,
    pub published_by: String,
    pub project_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsumeRevisionRequest {
    pub project_id: String,
    pub consumer_workset_id: String,
    pub source_workset_id: String,
    pub source_revision_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelLinkRefreshDto {
    pub link_id: String,
    pub source_model_id: String,
    pub current_revision: Option<u64>,
    pub latest_published_revision: Option<u64>,
    pub stale: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PresenceUpdateDto {
    pub project_id: String,
    pub account_id: String,
    pub device_id: String,
    pub scope: ScopeDto,
    pub selected_object_id: Option<String>,
    pub cursor_world: Option<[f64;3]>,
    pub client_time_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcquireLeaseRequest {
    pub project_id: String,
    pub object_id: String,
    pub account_id: String,
    pub device_id: String,
    pub object_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcquireLeaseResponse {
    pub granted: bool,
    pub owner_account_id: Option<String>,
    pub owner_device_id: Option<String>,
    pub expires_at_ms: Option<u64>,
}

impl PushChangeSetRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.project_id.trim().is_empty() { return Err("project_id trống".into()); }
        if self.account_id.trim().is_empty() { return Err("account_id trống".into()); }
        if self.device_id.trim().is_empty() { return Err("device_id trống".into()); }
        if self.client_change_set_id.trim().is_empty() { return Err("client_change_set_id trống".into()); }
        if self.items.is_empty() { return Err("ChangeSet không có item".into()); }
        for item in &self.items {
            if item.object_id.trim().is_empty() || item.source_handle.trim().is_empty() {
                return Err("Change item thiếu object_id/source_handle".into());
            }
            if item.local_revision <= item.base_revision && item.operation != ChangeOperationDto::Delete {
                return Err("local_revision phải lớn hơn base_revision".into());
            }
            if item.scope.project_id != self.project_id {
                return Err("Scope project_id không khớp ChangeSet".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope()->ScopeDto{
        ScopeDto{
            project_id:"P".into(),building_id:Some("B1".into()),zone_id:None,floor_id:Some("F2".into()),
            discipline:Some("ELEC".into()),workset_id:Some("LIGHT".into()),owner_account_id:Some("u1".into())
        }
    }

    #[test]
    fn publish_contract_requires_review_state_by_convention() {
        let req=PublishRevisionRequest{project_id:"P".into(),workset_id:"W".into(),revision_id:"R1".into(),account_id:"u".into(),expected_state:"REVIEW".into()};
        assert_eq!(req.expected_state,"REVIEW");
    }

    #[test]
    fn changeset_rejects_scope_from_other_project() {
        let mut bad=scope(); bad.project_id="OTHER".into();
        let req=PushChangeSetRequest{
            project_id:"P".into(),account_id:"u1".into(),device_id:"pc".into(),client_change_set_id:"c1".into(),base_project_revision:4,
            items:vec![ChangeItemDto{object_id:"o1".into(),source_handle:"A".into(),scope:bad,operation:ChangeOperationDto::Update,base_revision:1,local_revision:2,payload_json:"{}".into()}]
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn valid_incremental_update_passes_contract_validation() {
        let req=PushChangeSetRequest{
            project_id:"P".into(),account_id:"u1".into(),device_id:"pc".into(),client_change_set_id:"c1".into(),base_project_revision:4,
            items:vec![ChangeItemDto{object_id:"o1".into(),source_handle:"A".into(),scope:scope(),operation:ChangeOperationDto::Update,base_revision:1,local_revision:2,payload_json:"{}".into()}]
        };
        assert!(req.validate().is_ok());
    }
}
