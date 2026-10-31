//! BIM worksets and publish/consume lifecycle.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevisionState {
    Working,
    Review,
    Published,
    Superseded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workset {
    pub id: String,
    pub project_id: String,
    pub floor_id: String,
    pub discipline: String,
    pub code: String,
    pub name: String,
    pub owner_account_id: String,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelRevision {
    pub id: u64,
    pub project_id: String,
    pub workset_id: String,
    pub number: u64,
    pub state: RevisionState,
    pub created_by: String,
    pub reviewed_by: Option<String>,
    pub published_by: Option<String>,
}

impl ModelRevision {
    pub fn submit_for_review(&mut self) -> bool {
        if self.state != RevisionState::Working { return false; }
        self.state = RevisionState::Review;
        true
    }

    pub fn publish(&mut self, account_id: String) -> bool {
        if self.state != RevisionState::Review { return false; }
        self.state = RevisionState::Published;
        self.published_by = Some(account_id);
        true
    }

}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsumedRevision {
    pub consumer_workset_id: String,
    pub source_workset_id: String,
    pub source_revision: u64,
}

pub fn consume_revision(consumer_workset_id: String, source: &ModelRevision) -> Result<ConsumedRevision, String> {
    if source.state != RevisionState::Published {
        return Err("Chỉ được consume revision đã Published".into());
    }
    Ok(ConsumedRevision {
        consumer_workset_id,
        source_workset_id: source.workset_id.clone(),
        source_revision: source.number,
    })
}

pub fn supersede_previous(previous: &mut ModelRevision, next: &ModelRevision) -> bool {
    if previous.workset_id != next.workset_id || previous.state != RevisionState::Published || next.state != RevisionState::Published || next.number <= previous.number {
        return false;
    }
    previous.state = RevisionState::Superseded;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consumer_can_only_reference_published_revision() {
        let mut r=ModelRevision{id:1,project_id:"P".into(),workset_id:"W".into(),number:1,state:RevisionState::Working,created_by:"u".into(),reviewed_by:None,published_by:None};
        assert!(consume_revision("C".into(),&r).is_err());
        r.state=RevisionState::Published;
        assert_eq!(consume_revision("C".into(),&r).unwrap().source_revision,1);
    }

    #[test]
    fn revision_requires_review_before_publish() {
        let mut r = ModelRevision {
            id: 1, project_id: "P".into(), workset_id: "W".into(), number: 1,
            state: RevisionState::Working, created_by: "u1".into(), reviewed_by: None, published_by: None
        };
        assert!(!r.publish("u2".into()));
        assert!(r.submit_for_review());
        assert!(r.publish("u2".into()));
        assert_eq!(r.state, RevisionState::Published);
    }
}
