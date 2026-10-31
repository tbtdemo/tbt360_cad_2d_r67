use cadcraft_engine::{Session, doc::EntityKind};
use serde_json::json;

fn face_count(s: &Session) -> usize {
    s.doc().unwrap().model.iter().filter(|e| matches!(e.kind, EntityKind::Face3d(_))).count()
}

#[test]
fn box_creates_six_real_3dface_facets_with_height() {
    let mut s = Session::new();
    let out = s.execute("box", &json!({"base":[1.0,2.0,3.0],"length":4.0,"width":5.0,"height":6.0})).unwrap();
    assert_eq!(out["representation"], "3DFACE facets");
    assert_eq!(out["acis"], false);
    assert_eq!(face_count(&s), 6);
    assert!(s.doc().unwrap().model.iter().any(|e| matches!(&e.kind, EntityKind::Face3d(f) if f.corners.iter().any(|p| (p.z - 9.0).abs() < 1e-9))));
}

#[test]
fn curved_primitives_generate_facets_and_validate_dimensions() {
    for (cmd, params) in [
        ("cylinder", json!({"radius":2.0,"height":4.0,"segments":8})),
        ("cone", json!({"radius":2.0,"height":4.0,"segments":8})),
        ("sphere", json!({"radius":2.0,"segments":8,"rings":4})),
        ("torus", json!({"majorRadius":4.0,"minorRadius":1.0,"segments":8,"tubeSegments":4})),
    ] {
        let mut s = Session::new();
        let out = s.execute(cmd, &params).unwrap();
        assert!(out["faces"].as_u64().unwrap_or(0) > 0, "{cmd}");
        assert!(face_count(&s) > 0, "{cmd}");
    }
    let mut s = Session::new();
    assert!(s.execute("box", &json!({"length":0.0})).is_err());
    assert!(s.execute("torus", &json!({"majorRadius":2.0,"minorRadius":3.0})).is_err());
}
