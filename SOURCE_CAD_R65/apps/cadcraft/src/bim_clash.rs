//! BIM IFC/3D clash domain model. This is transport/renderer independent and does not pretend
//! 2D bounds are authoritative 3D clash results.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb3 {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl Aabb3 {
    pub fn valid(&self) -> bool {
        self.min.iter().zip(self.max).all(|(a,b)| a.is_finite() && b.is_finite() && *a <= b)
    }
    pub fn intersects(&self, other: &Self, tolerance: f64) -> bool {
        (0..3).all(|i| self.max[i] + tolerance >= other.min[i] && other.max[i] + tolerance >= self.min[i])
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClashObject {
    pub id: String,
    pub floor_id: String,
    pub discipline: String,
    pub owner_account_id: String,
    pub source_system: String,
    pub aabb: Aabb3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClashType { Hard, Clearance, Duplicate }

#[derive(Clone, Debug, PartialEq)]
pub struct ClashCandidate {
    pub a: String,
    pub b: String,
    pub clash_type: ClashType,
}

pub fn broadphase(objects: &[ClashObject], tolerance: f64, max_pairs: usize) -> Vec<ClashCandidate> {
    let mut out=Vec::new();
    let mut checked=0usize;
    for i in 0..objects.len() {
        for j in (i+1)..objects.len() {
            if checked>=max_pairs { return out; }
            checked+=1;
            let a=&objects[i]; let b=&objects[j];
            if a.floor_id!=b.floor_id || a.discipline==b.discipline || !a.aabb.valid() || !b.aabb.valid() { continue; }
            if a.aabb.intersects(&b.aabb,tolerance) {
                out.push(ClashCandidate{a:a.id.clone(),b:b.id.clone(),clash_type:ClashType::Hard});
            }
        }
    }
    out
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn broadphase_only_pairs_other_disciplines_on_same_floor(){
        let box3=Aabb3{min:[0.0,0.0,0.0],max:[1.0,1.0,1.0]};
        let a=ClashObject{id:"A".into(),floor_id:"F1".into(),discipline:"STRUCT".into(),owner_account_id:"u1".into(),source_system:"IFC".into(),aabb:box3};
        let b=ClashObject{id:"B".into(),floor_id:"F1".into(),discipline:"MEP".into(),owner_account_id:"u2".into(),source_system:"IFC".into(),aabb:box3};
        let c=ClashObject{id:"C".into(),floor_id:"F2".into(),discipline:"MEP".into(),owner_account_id:"u3".into(),source_system:"IFC".into(),aabb:box3};
        assert_eq!(broadphase(&[a,b,c],0.0,100).len(),1);
    }
}
