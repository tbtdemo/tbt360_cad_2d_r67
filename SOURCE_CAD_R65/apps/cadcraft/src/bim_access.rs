//! BIM authorization policy independent from UI and transport.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectRole { Owner, Coordinator, Designer, Viewer }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Principal {
    pub account_id: String,
    pub role: ProjectRole,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    pub project_id: String,
    pub building_id: Option<String>,
    pub zone_id: Option<String>,
    pub floor_id: Option<String>,
    pub discipline: Option<String>,
    pub workset_id: Option<String>,
    pub owner_account_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant {
    pub viewer_account_id: String,
    pub source_owner_account_id: Option<String>,
    pub project_id: String,
    pub building_id: Option<String>,
    pub zone_id: Option<String>,
    pub floor_id: Option<String>,
    pub discipline: Option<String>,
    pub workset_id: Option<String>,
    pub can_view: bool,
    pub can_edit: bool,
    pub can_publish: bool,
    pub deny: bool,
}

fn matches(expected: &Option<String>, actual: &Option<String>) -> bool {
    expected.is_none() || expected == actual
}

pub fn authorize(principal: &Principal, scope: &Scope, grants: &[Grant], edit: bool, publish: bool) -> bool {
    if principal.role == ProjectRole::Owner { return true; }
    let relevant = |g: &&Grant| {
        g.viewer_account_id == principal.account_id
            && g.project_id == scope.project_id
            && matches(&g.building_id, &scope.building_id)
            && matches(&g.zone_id, &scope.zone_id)
            && matches(&g.floor_id, &scope.floor_id)
            && matches(&g.discipline, &scope.discipline)
            && matches(&g.workset_id, &scope.workset_id)
            && g.source_owner_account_id.as_ref().is_none_or(|owner| scope.owner_account_id.as_ref() == Some(owner))
    };
    if grants.iter().filter(relevant).any(|g| g.deny) { return false; }
    grants.iter().filter(relevant).any(|g| if publish { g.can_publish } else if edit { g.can_edit } else { g.can_view })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_can_limit_viewer_to_one_owner_floor_discipline_and_workset() {
        let p=Principal{account_id:"u1".into(),role:ProjectRole::Designer};
        let g=Grant{
            viewer_account_id:"u1".into(),source_owner_account_id:Some("u2".into()),project_id:"P".into(),
            building_id:None,zone_id:None,floor_id:Some("F2".into()),discipline:Some("ELEC".into()),
            workset_id:Some("LIGHT".into()),can_view:true,can_edit:false,can_publish:false,deny:false
        };
        let s=Scope{project_id:"P".into(),building_id:None,zone_id:None,floor_id:Some("F2".into()),discipline:Some("ELEC".into()),workset_id:Some("LIGHT".into()),owner_account_id:Some("u2".into())};
        assert!(authorize(&p,&s,std::slice::from_ref(&g),false,false));
        assert!(!authorize(&p,&s,std::slice::from_ref(&g),true,false));
        let mut wrong=s.clone(); wrong.owner_account_id=Some("u3".into());
        assert!(!authorize(&p,&wrong,&[g],false,false));
    }

    #[test]
    fn explicit_deny_overrides_allow() {
        let p=Principal{account_id:"u1".into(),role:ProjectRole::Designer};
        let s=Scope{project_id:"P".into(),building_id:None,zone_id:None,floor_id:Some("F1".into()),discipline:Some("ARCH".into()),workset_id:None,owner_account_id:Some("u2".into())};
        let allow=Grant{viewer_account_id:"u1".into(),source_owner_account_id:None,project_id:"P".into(),building_id:None,zone_id:None,floor_id:Some("F1".into()),discipline:Some("ARCH".into()),workset_id:None,can_view:true,can_edit:true,can_publish:true,deny:false};
        let deny=Grant{deny:true,..allow.clone()};
        assert!(!authorize(&p,&s,&[allow,deny],false,false));
    }
}
