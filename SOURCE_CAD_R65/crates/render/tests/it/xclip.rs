use std::sync::Arc;

use cadcraft_doc::{Block, Common, Drawing, Entity, EntityKind, Insert, Line, RawTag, Space};
use cadcraft_geom::Vec3;

#[test]
fn rectangular_xclip_clips_xref_geometry_in_display_list() {
    let mut d = Drawing::new_metric();
    let mut b = Block::new("SITE");
    b.xref_path = Some("site.dwg".into());
    b.entities.push(Entity::new(
        cadcraft_doc::Handle(2),
        EntityKind::Line(Line { a: Vec3::new(-10.0, 0.0, 0.0), b: Vec3::new(10.0, 0.0, 0.0) }),
    ));
    d.blocks.insert("SITE".into(), Arc::new(b));
    let mut e = Entity::new(
        cadcraft_doc::Handle(1),
        EntityKind::Insert(Insert {
            block: "SITE".into(),
            insert: Vec3::ZERO,
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: 0.0,
            attribs: Vec::new(),
            cols: 1,
            rows: 1,
            col_spacing: 0.0,
            row_spacing: 0.0,
        }),
    );
    e.common = Common { xdata: vec![
        RawTag { code: 1001, value: "TBT360_XCLIP".into() },
        RawTag { code: 1070, value: "1".into() },
        RawTag { code: 1000, value: "-2,-2;2,2".into() },
    ], ..Common::default() };
    d.model.push(e);
    let list = cadcraft_render::build(&d, &Space::Model, &cadcraft_render::Options::default());
    let rect = cadcraft_geom::Bounds2::new(cadcraft_geom::Vec2::new(-2.0, -2.0), cadcraft_geom::Vec2::new(2.0, 2.0));
    assert!(!list.prims.is_empty());
    assert!(list.prims.iter().flat_map(|p| list.points(p)).all(|p| rect.contains(*p)));
}

#[test]
fn concave_polygon_xclip_removes_notch_and_retains_both_arms() {
    let mut d = Drawing::new_metric();
    let mut b = Block::new("CONCAVE_SITE");
    b.xref_path = Some("site.dxf".into());
    b.entities.push(Entity::new(cadcraft_doc::Handle(20), EntityKind::Line(Line {
        a: Vec3::new(-1.0, 2.0, 0.0), b: Vec3::new(7.0, 2.0, 0.0),
    })));
    d.blocks.insert("CONCAVE_SITE".into(), Arc::new(b));
    let mut e = Entity::new(cadcraft_doc::Handle(10), EntityKind::Insert(Insert {
        block: "CONCAVE_SITE".into(), insert: Vec3::ZERO,
        scale: Vec3::new(1.0, 1.0, 1.0), rotation: 0.0,
        attribs: Vec::new(), cols: 1, rows: 1,
        col_spacing: 0.0, row_spacing: 0.0,
    }));
    // U-shaped concave polygon: the segment through y=2 must leave a gap x=2..4.
    e.common.xdata = vec![
        RawTag { code: 1001, value: "TBT360_XCLIP".into() },
        RawTag { code: 1070, value: "2".into() },
        RawTag { code: 1000, value: "0,0;6,0;6,4;4,4;4,1;2,1;2,4;0,4".into() },
    ];
    d.model.push(e);
    let list = cadcraft_render::build(&d, &Space::Model, &cadcraft_render::Options::default());
    let pieces: Vec<_> = list.prims.iter().filter(|p| {
        matches!(p.kind, cadcraft_render::Kind::Polyline)
    }).map(|p| list.points(p)).collect();
    assert!(pieces.len() >= 2, "concave clip should split the line");
    for piece in pieces {
        for v in piece {
            assert!((0.0..=6.0).contains(&v.x));
            assert!((v.x <= 2.0 + 1e-8) || (v.x >= 4.0 - 1e-8),
                "notch must be clipped out: {:?}", v);
        }
    }
}

#[test]
fn nested_polygon_xclip_intersects_parent_and_child_boundaries() {
    let mut d = Drawing::new_metric();
    let mut inner = Block::new("NESTED_INNER");
    inner.entities.push(Entity::new(cadcraft_doc::Handle(31), EntityKind::Line(Line {
        a: Vec3::new(-2.0, 0.0, 0.0), b: Vec3::new(10.0, 0.0, 0.0),
    })));
    d.blocks.insert("NESTED_INNER".into(), Arc::new(inner));
    let mut outer = Block::new("NESTED_OUTER");
    let mut child = Entity::new(cadcraft_doc::Handle(32), EntityKind::Insert(Insert {
        block: "NESTED_INNER".into(), insert: Vec3::ZERO,
        scale: Vec3::new(1.0,1.0,1.0), rotation: 0.0,
        attribs: Vec::new(), cols: 1, rows: 1, col_spacing: 0.0, row_spacing: 0.0,
    }));
    child.common.xdata = vec![
        RawTag { code: 1001, value: "TBT360_XCLIP".into() },
        RawTag { code: 1070, value: "2".into() },
        RawTag { code: 1000, value: "2,-1;8,-1;8,1;2,1".into() },
    ];
    outer.entities.push(child);
    d.blocks.insert("NESTED_OUTER".into(), Arc::new(outer));
    let mut parent = Entity::new(cadcraft_doc::Handle(33), EntityKind::Insert(Insert {
        block: "NESTED_OUTER".into(), insert: Vec3::ZERO,
        scale: Vec3::new(1.0,1.0,1.0), rotation: 0.0,
        attribs: Vec::new(), cols: 1, rows: 1, col_spacing: 0.0, row_spacing: 0.0,
    }));
    parent.common.xdata = vec![
        RawTag { code: 1001, value: "TBT360_XCLIP".into() },
        RawTag { code: 1070, value: "2".into() },
        RawTag { code: 1000, value: "0,-1;4,-1;4,1;0,1".into() },
    ];
    d.model.push(parent);
    let list = cadcraft_render::build(&d, &Space::Model, &cadcraft_render::Options::default());
    let points: Vec<_> = list.prims.iter().filter(|p| matches!(p.kind, cadcraft_render::Kind::Polyline))
        .flat_map(|p| list.points(p)).collect();
    assert!(points.len() >= 2, "nested clipping must preserve intersection");
    assert!(points.iter().all(|p| p.x >= 2.0 - 1e-8 && p.x <= 4.0 + 1e-8),
        "nested clip must intersect both polygons, not replace the parent");
}
