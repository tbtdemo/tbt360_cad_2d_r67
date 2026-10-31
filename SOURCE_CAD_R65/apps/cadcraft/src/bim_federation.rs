//! Federated/linked BIM model domain.
//! A project may present one coordinated workspace while each discipline/user keeps an independent source model.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkSource {
    NativeCad,
    Dxf,
    Dwg,
    Ifc,
    ExternalReference,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkState {
    Unloaded,
    Loading,
    Loaded,
    Missing,
    Stale,
    Conflict,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinkTransform {
    pub translation_m: [f64; 3],
    pub rotation_deg: f64,
    pub scale: f64,
}

impl Default for LinkTransform {
    fn default() -> Self {
        Self { translation_m: [0.0; 3], rotation_deg: 0.0, scale: 1.0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FederatedModelLink {
    pub id: String,
    pub project_id: String,
    pub source_model_id: String,
    pub source: LinkSource,
    pub discipline: String,
    pub owner_account_id: String,
    pub building_id: Option<String>,
    pub zone_id: Option<String>,
    pub floor_id: Option<String>,
    pub published_revision: Option<u64>,
    pub transform: LinkTransform,
    pub opacity: f32,
    pub state: LinkState,
    pub read_only: bool,
}

impl FederatedModelLink {
    pub fn display_opacity(&self) -> f32 {
        self.opacity.clamp(0.05, 1.0)
    }

    pub fn editable(&self) -> bool {
        !self.read_only && self.source == LinkSource::NativeCad
    }

    pub fn needs_refresh(&self, latest_published_revision: u64) -> bool {
        self.published_revision.is_some_and(|r| r < latest_published_revision)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Federation {
    pub links: Vec<FederatedModelLink>,
}

impl Federation {
    pub fn mark_stale_against_latest(&mut self, source_model_id:&str, latest_published_revision:u64) -> usize {
        let mut changed=0;
        for link in &mut self.links {
            if link.source_model_id==source_model_id && link.needs_refresh(latest_published_revision) {
                link.state=LinkState::Stale;
                changed+=1;
            }
        }
        changed
    }

    pub fn refresh_link(&mut self, link_id:&str, published_revision:u64) -> bool {
        let Some(link)=self.links.iter_mut().find(|l|l.id==link_id) else{return false};
        link.published_revision=Some(published_revision);
        link.state=LinkState::Loaded;
        true
    }

    pub fn add_or_replace(&mut self, link: FederatedModelLink) {
        if let Some(existing) = self.links.iter_mut().find(|x| x.id == link.id) {
            *existing = link;
        } else {
            self.links.push(link);
        }
    }

    pub fn visible_links_for_floor(&self, floor_id: &str) -> Vec<&FederatedModelLink> {
        self.links.iter()
            .filter(|l| matches!(l.state, LinkState::Loaded | LinkState::Stale))
            .filter(|l| l.floor_id.as_deref().is_none_or(|f| f == floor_id))
            .collect()
    }

    pub fn stale_count(&self) -> usize {
        self.links.iter().filter(|l| l.state == LinkState::Stale).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_link_can_be_refreshed_to_latest_published_revision() {
        let mut f=Federation::default();
        f.add_or_replace(FederatedModelLink{
            id:"L".into(),project_id:"P".into(),source_model_id:"M".into(),source:LinkSource::Ifc,
            discipline:"STRUCT".into(),owner_account_id:"u".into(),building_id:None,zone_id:None,floor_id:Some("F1".into()),
            published_revision:Some(2),transform:LinkTransform::default(),opacity:0.3,state:LinkState::Loaded,read_only:true
        });
        assert_eq!(f.mark_stale_against_latest("M",3),1);
        assert_eq!(f.links[0].state,LinkState::Stale);
        assert!(f.refresh_link("L",3));
        assert_eq!(f.links[0].published_revision,Some(3));
        assert_eq!(f.links[0].state,LinkState::Loaded);
    }

    #[test]
    fn external_ifc_link_is_reference_only() {
        let link = FederatedModelLink {
            id: "ifc-1".into(),
            project_id: "P".into(),
            source_model_id: "M".into(),
            source: LinkSource::Ifc,
            discipline: "STRUCT".into(),
            owner_account_id: "u2".into(),
            building_id: None,
            zone_id: None,
            floor_id: Some("F2".into()),
            published_revision: Some(4),
            transform: LinkTransform::default(),
            opacity: 0.25,
            state: LinkState::Loaded,
            read_only: true,
        };
        assert!(!link.editable());
        assert_eq!(link.display_opacity(), 0.25);
        assert!(link.needs_refresh(5));
    }

    #[test]
    fn federation_filters_links_by_floor_without_merging_source_models() {
        let mut f = Federation::default();
        for (id, floor) in [("A", "F1"), ("B", "F2")] {
            f.add_or_replace(FederatedModelLink {
                id: id.into(),
                project_id: "P".into(),
                source_model_id: id.into(),
                source: LinkSource::NativeCad,
                discipline: "ARCH".into(),
                owner_account_id: "u".into(),
                building_id: None,
                zone_id: None,
                floor_id: Some(floor.into()),
                published_revision: Some(1),
                transform: LinkTransform::default(),
                opacity: 1.0,
                state: LinkState::Loaded,
                read_only: true,
            });
        }
        assert_eq!(f.visible_links_for_floor("F2").len(), 1);
        assert_eq!(f.links.len(), 2);
    }
}
