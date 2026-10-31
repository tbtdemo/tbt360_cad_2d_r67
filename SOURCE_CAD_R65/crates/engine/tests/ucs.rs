use cadcraft_geom::Vec3;
use serde_json::json;

use cadcraft_engine::Session;

#[test]
fn ucs_save_set_list_and_delete_persist_model_definition() {
    let mut s = Session::new();
    let saved = s.execute("ucs", &json!({
        "mode": "save",
        "name": "Survey",
        "origin": [100.0, 200.0, 30.0],
        "xAxis": [0.0, 2.0, 0.0],
        "yAxis": [-3.0, 0.0, 0.0]
    })).unwrap();
    assert_eq!(saved["current"], "Survey");
    assert_eq!(s.doc().unwrap().header.str("UCSNAME", ""), "Survey");
    let u = s.doc().unwrap().ucss.iter().find(|u| u.name == "Survey").unwrap();
    assert_eq!(u.origin, Vec3::new(100.0, 200.0, 30.0));
    assert!((u.x_axis.len() - 1.0).abs() < 1e-12);
    assert!((u.y_axis.len() - 1.0).abs() < 1e-12);
    assert!(u.x_axis.dot(u.y_axis).abs() < 1e-12);

    s.execute("ucs", &json!({"mode": "set", "name": "Survey"})).unwrap();
    let list = s.execute("ucs", &json!({"mode": "list"})).unwrap();
    assert_eq!(list["current"], "Survey");
    assert_eq!(list["ucss"][0]["name"], "Survey");

    s.execute("ucs", &json!({"mode": "delete", "name": "Survey"})).unwrap();
    assert!(s.doc().unwrap().ucss.iter().all(|u| u.name != "Survey"));
}

#[test]
fn ucs_rejects_degenerate_basis() {
    let mut s = Session::new();
    assert!(s.execute("ucs", &json!({
        "mode": "save", "name": "Bad", "xAxis": [1.0, 0.0, 0.0], "yAxis": [2.0, 0.0, 0.0]
    })).is_err());
}
