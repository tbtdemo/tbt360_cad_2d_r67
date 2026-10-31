//! BIM coordination progress, issues and immutable-style audit events.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressStatus{NotStarted,InProgress,Review,Complete}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ScopeProgress{
    pub floor_id:String,
    pub discipline:String,
    pub workset_id:Option<String>,
    pub percent:u8,
    pub status:ProgressStatus,
    pub updated_by:String,
}

impl ScopeProgress{
    pub fn set_percent(&mut self,p:u8){
        self.percent=p.min(100);
        self.status=match self.percent{0=>ProgressStatus::NotStarted,100=>ProgressStatus::Complete,_=>ProgressStatus::InProgress};
    }

}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum IssueSeverity{Info,Warning,Error,Critical}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum IssueStatus{Open,InProgress,Resolved,Closed}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct CoordinationIssue{
    pub id:String,
    pub floor_id:String,
    pub source_discipline:String,
    pub target_discipline:String,
    pub source_handle:Option<String>,
    pub assigned_to:Option<String>,
    pub severity:IssueSeverity,
    pub status:IssueStatus,
    pub description:String,
}

impl CoordinationIssue {
    pub fn start(&mut self) -> bool {
        if self.status != IssueStatus::Open { return false; }
        self.status = IssueStatus::InProgress;
        true
    }
    pub fn resolve(&mut self) -> bool {
        if !matches!(self.status, IssueStatus::Open | IssueStatus::InProgress) { return false; }
        self.status = IssueStatus::Resolved;
        true
    }
    pub fn close(&mut self) -> bool {
        if self.status != IssueStatus::Resolved { return false; }
        self.status = IssueStatus::Closed;
        true
    }
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct AuditEvent{
    pub sequence:u64,
    pub account_id:String,
    pub action:String,
    pub object_handle:Option<String>,
    pub detail:String,
}

#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct QualityGate{
    pub unscoped_entities:usize,
    pub conflicts:usize,
    pub open_errors:usize,
    pub pending_changes:usize,
}

impl QualityGate{
    pub fn publishable(&self)->bool{self.unscoped_entities==0&&self.conflicts==0&&self.open_errors==0&&self.pending_changes==0}
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn issue_follows_open_inprogress_resolved_closed_lifecycle() {
        let mut i=CoordinationIssue{id:"I1".into(),floor_id:"F1".into(),source_discipline:"ARCH".into(),target_discipline:"STRUCT".into(),source_handle:None,assigned_to:None,severity:IssueSeverity::Error,status:IssueStatus::Open,description:"x".into()};
        assert!(i.start());
        assert!(i.resolve());
        assert!(i.close());
        assert_eq!(i.status,IssueStatus::Closed);
    }

    #[test]
    fn progress_reaches_complete_at_one_hundred_percent(){
        let mut p=ScopeProgress{floor_id:"F1".into(),discipline:"ARCH".into(),workset_id:None,percent:0,status:ProgressStatus::NotStarted,updated_by:"u".into()};
        p.set_percent(100);assert_eq!(p.status,ProgressStatus::Complete);
    }
    #[test]
    fn critical_open_issue_blocks_publish(){
        let q=QualityGate{open_errors:1,..Default::default()};assert!(!q.publishable());
    }
}
