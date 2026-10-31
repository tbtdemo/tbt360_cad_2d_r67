//! BIM viewport composition policy. The actual drawing uses CAD native VPORTS.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewRole { Edit, Reference, Coordination, Progress }

#[derive(Clone, Debug, PartialEq)]
pub struct ViewScope {
    pub floor_id: String,
    pub discipline: Option<String>,
    pub owner_account_id: Option<String>,
    pub workset_id: Option<String>,
    pub role: ViewRole,
    pub reference_opacity: f32,
}

impl ViewScope {
    pub fn editable(&self) -> bool { self.role == ViewRole::Edit }
    pub fn opacity(&self) -> f32 { if self.editable() { 1.0 } else { self.reference_opacity.clamp(0.05,1.0) } }
    pub fn show_all_disciplines(&self) -> bool { matches!(self.role, ViewRole::Coordination | ViewRole::Progress) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectScope {
    pub floor_id: String,
    pub discipline: String,
    pub owner_account_id: String,
    pub workset_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderClass { Primary, Reference, Hidden }

pub fn classify(view:&ViewScope,obj:&ObjectScope,can_view_other:bool)->RenderClass{
    if obj.floor_id!=view.floor_id { return RenderClass::Hidden; }
    let discipline_matches=view.show_all_disciplines() || view.discipline.as_ref().is_none_or(|d|d==&obj.discipline);
    let owner_matches=view.owner_account_id.as_ref().is_none_or(|o|o==&obj.owner_account_id);
    let workset_matches=view.workset_id.as_ref().is_none_or(|w|obj.workset_id.as_ref()==Some(w));
    if discipline_matches&&owner_matches&&workset_matches { return RenderClass::Primary; }
    if can_view_other { RenderClass::Reference } else { RenderClass::Hidden }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ViewportLayout {
    pub scopes: Vec<ViewScope>,
}

impl ViewportLayout {
    pub fn validate(&self)->Result<(),String>{
        if self.scopes.is_empty()||self.scopes.len()>4{return Err("Bố cục BIM chỉ hỗ trợ 1..=4 viewport".into());}
        if self.scopes.iter().filter(|s|s.role==ViewRole::Edit).count()>1{return Err("Chỉ một viewport được phép ở chế độ Edit".into());}
        Ok(())
    }

}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn coordination_view_can_reference_all_disciplines_on_same_floor() {
        let v=ViewScope{floor_id:"F2".into(),discipline:Some("ARCH".into()),owner_account_id:None,workset_id:None,role:ViewRole::Coordination,reference_opacity:0.2};
        let o=ObjectScope{floor_id:"F2".into(),discipline:"ELEC".into(),owner_account_id:"u".into(),workset_id:None};
        assert_eq!(classify(&v,&o,true),RenderClass::Primary);
    }

    #[test]
    fn one_edit_view_and_reference_views_are_valid(){
        let l=ViewportLayout{scopes:vec![
            ViewScope{floor_id:"F2".into(),discipline:Some("ARCH".into()),owner_account_id:Some("u1".into()),workset_id:None,role:ViewRole::Edit,reference_opacity:1.0},
            ViewScope{floor_id:"F2".into(),discipline:Some("STRUCT".into()),owner_account_id:Some("u2".into()),workset_id:None,role:ViewRole::Reference,reference_opacity:0.25},
        ]};
        assert!(l.validate().is_ok());
        assert_eq!(l.scopes[1].opacity(),0.25);
    }
    #[test]
    fn second_edit_view_is_rejected(){
        let s=ViewScope{floor_id:"F1".into(),discipline:None,owner_account_id:None,workset_id:None,role:ViewRole::Edit,reference_opacity:1.0};
        assert!(ViewportLayout{scopes:vec![s.clone(),s]}.validate().is_err());
    }
}
