use cadcraft_engine::{Session, doc::EntityKind};
use serde_json::json;

#[test]
fn mesh_builds_dxf_compatible_facets_from_indexed_vertices() {
    let mut s = Session::new();
    let result = s.execute("mesh", &json!({
        "vertices": [[0, 0, 0], [10, 0, 0], [10, 10, 5], [0, 10, 5]],
        "faces": [[0, 1, 2], [0, 2, 3]],
        "hiddenEdges": 2
    })).unwrap();
    assert_eq!(result["vertices"], 4);
    assert_eq!(result["faces"], 2);
    assert_eq!(result["handles"].as_array().unwrap().len(), 2);
    let entities: Vec<_> = s.doc().unwrap().model.iter().collect();
    assert!(entities.iter().all(|e| matches!(e.kind, EntityKind::Face3d(_))));
    let EntityKind::Face3d(face) = &entities[1].kind else { panic!("expected face") };
    assert_eq!(face.corners[2].z, 5.0);
}

#[test]
fn mesh_rejects_invalid_face_indices_before_creating_entities() {
    let mut s = Session::new();
    assert!(s.execute("mesh", &json!({
        "vertices": [[0, 0, 0], [1, 0, 0], [0, 1, 0]],
        "faces": [[0, 1, 4]]
    })).is_err());
    assert_eq!(s.doc().unwrap().model.len(), 0);
}
