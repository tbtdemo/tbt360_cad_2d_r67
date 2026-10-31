//! Shared BIM collaboration scope contract used by integration.
//! Feature branches should converge on this shape before main merge.

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ProjectScope {
    pub project_id: String,
    pub building_id: Option<String>,
    pub zone_id: Option<String>,
    pub floor_id: Option<String>,
    pub discipline: Option<String>,
    pub workset_id: Option<String>,
    pub owner_account_id: Option<String>,
}

impl ProjectScope {
    pub fn contains(&self, other: &Self) -> bool {
        self.project_id == other.project_id
            && self.building_id.as_ref().is_none_or(|v| other.building_id.as_ref() == Some(v))
            && self.zone_id.as_ref().is_none_or(|v| other.zone_id.as_ref() == Some(v))
            && self.floor_id.as_ref().is_none_or(|v| other.floor_id.as_ref() == Some(v))
            && self.discipline.as_ref().is_none_or(|v| other.discipline.as_ref() == Some(v))
            && self.workset_id.as_ref().is_none_or(|v| other.workset_id.as_ref() == Some(v))
            && self.owner_account_id.as_ref().is_none_or(|v| other.owner_account_id.as_ref() == Some(v))
    }

    pub fn editable_identity(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}",
            self.project_id,
            self.building_id.as_deref().unwrap_or("*"),
            self.zone_id.as_deref().unwrap_or("*"),
            self.floor_id.as_deref().unwrap_or("*"),
            self.discipline.as_deref().unwrap_or("*"),
            self.workset_id.as_deref().unwrap_or("*"),
            self.owner_account_id.as_deref().unwrap_or("*")
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollaborationMode {
    OfflinePersonal,
    OnlineCollaborative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewIntent {
    Edit,
    Reference,
    Coordination,
    Progress,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_scope_contains_more_specific_scope() {
        let parent = ProjectScope { project_id: "P".into(), floor_id: Some("F2".into()), ..Default::default() };
        let child = ProjectScope {
            project_id: "P".into(),
            floor_id: Some("F2".into()),
            discipline: Some("ELEC".into()),
            workset_id: Some("LIGHT".into()),
            ..Default::default()
        };
        assert!(parent.contains(&child));
        assert!(!child.contains(&parent));
    }
}
