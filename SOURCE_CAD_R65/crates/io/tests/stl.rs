use cadcraft_doc::{Common, Drawing, EntityKind, Face3d, Space};
use cadcraft_geom::Vec3;

#[test]
fn binary_stl_round_trips_face_facets() {
    let mut d = Drawing::new_metric();
    d.add(&Space::Model, Common::default(), EntityKind::Face3d(Face3d {
        corners: [Vec3::new(0.0, 0.0, 1.0), Vec3::new(10.0, 0.0, 1.0), Vec3::new(0.0, 10.0, 2.0), Vec3::new(0.0, 10.0, 2.0)],
        hidden_edges: 0,
    })).unwrap();
    let bytes = cadcraft_io::write_stl(&d).unwrap();
    assert_eq!(bytes.len(), 134);
    let read = cadcraft_io::read_stl(&bytes).unwrap();
    let e = read.model.iter().next().unwrap();
    let EntityKind::Face3d(face) = &e.kind else { panic!("expected imported facet") };
    assert!((face.corners[0].z - 1.0).abs() < 1e-6);
    assert_eq!(read.model.len(), 1);
}

#[test]
fn ascii_stl_is_imported_as_facets() {
    let ascii = b"solid test\n facet normal 0 0 1\n  outer loop\n   vertex 0 0 0\n   vertex 1 0 0\n   vertex 0 1 0\n  endloop\n endfacet\nendsolid test\n";
    let d = cadcraft_io::read_stl(ascii).unwrap();
    assert_eq!(d.model.len(), 1);
}
