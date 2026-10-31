use cadcraft_engine::{Session, doc::EntityKind};
use serde_json::json;

#[test]
fn three_dface_creates_real_face_with_z_and_hidden_edges() {
    let mut s = Session::new();
    let result = s.execute("3dface", &json!({
        "points": [[0.0, 0.0, 10.0], [10.0, 0.0, 10.0], [10.0, 10.0, 12.0], [0.0, 10.0, 12.0]],
        "hiddenEdges": 5
    })).unwrap();
    let handle = result["handle"].as_str().unwrap();
    let entity = s.doc().unwrap().model.iter().find(|e| e.handle.hex() == handle).unwrap();
    let EntityKind::Face3d(face) = &entity.kind else { panic!("expected 3DFACE") };
    assert_eq!(face.hidden_edges, 5);
    assert_eq!(face.corners[0].z, 10.0);
    assert_eq!(face.corners[2].z, 12.0);
}

#[test]
fn three_dface_triangle_repeats_last_corner_and_rejects_bad_count() {
    let mut s = Session::new();
    s.execute("3dface", &json!({"points": [[0, 0, 1], [1, 0, 1], [0, 1, 1]]})).unwrap();
    let face = s.doc().unwrap().model.iter().next().unwrap();
    let EntityKind::Face3d(face) = &face.kind else { panic!("expected 3DFACE") };
    assert_eq!(face.corners[2], face.corners[3]);
    assert!(s.execute("3dface", &json!({"points": [[0, 0], [1, 0]]})).is_err());
}
