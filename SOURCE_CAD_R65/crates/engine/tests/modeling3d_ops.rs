use cadcraft_engine::{
    Session,
    doc::{EntityKind, LwPolyline, Polyline3d},
    geom::{PolyVertex, Vec2, Vec3},
};
use serde_json::json;

fn square(z: f64, size: f64) -> LwPolyline {
    LwPolyline {
        vertices: vec![
            PolyVertex::new(Vec2::new(0.0, 0.0)),
            PolyVertex::new(Vec2::new(size, 0.0)),
            PolyVertex::new(Vec2::new(size, size)),
            PolyVertex::new(Vec2::new(0.0, size)),
        ],
        closed: true,
        const_width: 0.0,
        elevation: z,
        plinegen: false,
    }
}

fn faces(s: &Session) -> usize {
    s.doc().unwrap().model.iter().filter(|e| matches!(e.kind, EntityKind::Face3d(_))).count()
}

#[test]
fn extrude_selected_closed_profile_creates_side_and_cap_facets() {
    let mut s = Session::new();
    let profile = s.add_entity(EntityKind::LwPolyline(square(2.0, 4.0))).unwrap();
    s.set_selection(vec![profile]);
    let out = s.execute("extrude", &json!({"height": 5.0})).unwrap();
    assert_eq!(out["representation"], "3DFACE facets");
    assert!(faces(&s) >= 8);
    assert!(s.doc().unwrap().model.iter().any(|e| matches!(&e.kind, EntityKind::Face3d(f) if f.corners.iter().any(|p| (p.z - 7.0).abs() < 1e-9))));
}

#[test]
fn revolve_selected_profile_creates_faceted_surface() {
    let mut s = Session::new();
    let profile = s.add_entity(EntityKind::Polyline3d(Polyline3d {
        points: vec![Vec3::new(2.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 3.0)],
        closed: false,
    })).unwrap();
    s.set_selection(vec![profile]);
    let out = s.execute("revolve", &json!({"angle": 360.0, "segments": 12})).unwrap();
    assert!(out["faces"].as_u64().unwrap_or(0) >= 12);
    assert!(faces(&s) >= 12);
}

#[test]
fn sweep_uses_profile_then_path_selection_order() {
    let mut s = Session::new();
    let profile = s.add_entity(EntityKind::LwPolyline(square(0.0, 1.0))).unwrap();
    let path = s.add_entity(EntityKind::Polyline3d(Polyline3d {
        points: vec![Vec3::ZERO, Vec3::new(0.0, 0.0, 3.0), Vec3::new(2.0, 0.0, 5.0)],
        closed: false,
    })).unwrap();
    s.set_selection(vec![profile, path]);
    let out = s.execute("sweep", &json!({})).unwrap();
    assert!(out["faces"].as_u64().unwrap_or(0) >= 8);
}

#[test]
fn loft_resamples_two_closed_profiles_and_connects_them() {
    let mut s = Session::new();
    let a = s.add_entity(EntityKind::LwPolyline(square(0.0, 2.0))).unwrap();
    let b = s.add_entity(EntityKind::LwPolyline(square(5.0, 4.0))).unwrap();
    s.set_selection(vec![a, b]);
    let out = s.execute("loft", &json!({"samples": 16})).unwrap();
    assert_eq!(out["faces"].as_u64(), Some(16));
    assert!(faces(&s) >= 16);
}

#[test]
fn section_plane_creates_persistent_faceted_marker_with_metadata() {
    let mut s = Session::new();
    let out = s
        .execute(
            "sectionplane",
            &json!({
                "origin": [1.0, 2.0, 3.0],
                "normal": [0.0, 0.0, 1.0],
                "width": 10.0,
                "height": 4.0,
                "name": "Section A"
            }),
        )
        .unwrap();
    assert_eq!(out["representation"], "3DFACE section marker");
    assert_eq!(out["clipping"], false);
    let h = cadcraft_engine::doc::Handle::parse_hex(out["handle"].as_str().unwrap()).unwrap();
    let e = s.doc().unwrap().entity(h).unwrap();
    let EntityKind::Face3d(face) = &e.kind else { panic!("section marker must be a 3DFACE") };
    assert!(face.corners.iter().all(|p| (p.z - 3.0).abs() < 1e-9));
    assert!(e.common.xdata.iter().any(|t| t.code == 1001 && t.value == "TBT360_SECTION"));
    assert!(e.common.xdata.iter().any(|t| t.code == 1000 && t.value == "Section A"));
}

#[test]
fn sectionclip_persists_renderer_state() {
    let mut s = Session::new();
    s.execute("sectionplane", &json!({"origin":[0.0,0.0,0.0],"normal":[0.0,0.0,1.0]})).unwrap();
    let out=s.execute("sectionclip", &json!({"on":true,"side":"positive"})).unwrap();
    assert_eq!(out["on"], true);
    assert_eq!(s.doc().unwrap().header.i64("SECTIONCLIP",0),1);
    assert_eq!(s.doc().unwrap().header.str("SECTIONCLIPSIDE",""),"positive");
}

#[test]
fn pointcloud_creates_tagged_3d_points() {
    let mut s=Session::new();
    let out=s.execute("pointcloud", &json!({"name":"Scan A","points":[[0.0,0.0,1.0],[2.0,3.0,4.0]]})).unwrap();
    assert_eq!(out["count"],2);
    let tagged=s.doc().unwrap().model.iter().filter(|e| {
        matches!(e.kind, EntityKind::Point(_)) && e.common.xdata.iter().any(|t| t.code==1001 && t.value=="TBT360_POINTCLOUD")
    }).count();
    assert_eq!(tagged,2);
}
