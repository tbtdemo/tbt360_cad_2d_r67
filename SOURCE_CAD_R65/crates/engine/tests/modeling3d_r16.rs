use cadcraft_engine::Session;
use serde_json::json;

#[test]
fn perspective_orbit_persists_projection_and_distance() {
    let mut s=Session::new();
    s.new_drawing();
    let out=s.run("3dorbit", &json!({"yaw":25.0,"pitch":30.0,"projection":"perspective","distance":250.0})).unwrap();
    assert_eq!(out["projection"], "perspective");
    let d=s.doc().unwrap();
    assert_eq!(d.header.str("VIEWPROJ",""), "perspective");
    assert!((d.header.f64("VIEWDIST",0.0)-250.0).abs()<1e-9);
}

#[test]
fn text3d_creates_persistent_tagged_text() {
    let mut s=Session::new();
    s.new_drawing();
    let out=s.run("text3d", &json!({"at":[1.0,2.0,3.0],"text":"Level +3.000","height":0.25})).unwrap();
    let h=out["handle"].as_str().unwrap();
    let e=s.doc().unwrap().model.iter().find(|e| e.handle.hex()==h).unwrap();
    assert!(e.common.xdata.iter().any(|t| t.code==1001 && t.value=="TBT360_3DANNOTATION"));
}
