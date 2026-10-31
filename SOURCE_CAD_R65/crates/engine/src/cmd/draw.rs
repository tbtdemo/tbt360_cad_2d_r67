//! Draw commands: LINE, PLINE, CIRCLE, ARC, RECTANG, POLYGON, ELLIPSE, POINT, XLINE, RAY,
//! SPLINE, DONUT, TEXT, MTEXT. Each has a JSON form and an interactive prompt sequence.

use cadcraft_color::Color;
use cadcraft_doc::{EntityKind, Face3d, HAlign, Line, MText, Point, RawTag, RayLine, Text, VAlign};
use cadcraft_geom::{Arc, Circle, Ellipse, PolyVertex, Polyline, Spline, Vec2, Vec3, arc_to_bulge, bulge_to_arc};
use serde_json::{Value, json};

use super::helpers::{arc, circle, line, lwpoly, polygon_from_edge, polygon_typed_radius_point, polygon_vertices, v3};

use super::machines::number;
use super::pline_opts::{Outcome, PlineAsk};
use super::{
    CommandSpec, bad, bool_or, distinct_points, f64_or, f64_req, find_command, point_param, point_req, point_value, points_param, size_param,
    str_param,
};

use crate::{Accept, Input, Interactive, Prompt, Result, Session, Step, snap};

mod circle;
mod ellipse;
mod line_pline;
mod point;
mod polygon;
mod spline;
mod text;

use circle::{CircleM, run_arc, run_circle};
pub(crate) use ellipse::ellipse_machine;
use ellipse::{EllipseM, run_ellipse};
use line_pline::{LineM, PlineM, run_line, run_pline};
use point::{DonutM, PointM, RayM, run_donut, run_point, run_ray};
use polygon::{PolygonM, run_polygon};
use spline::{SplineM, run_spline};
use text::{MTextM, TextM, run_mtext, run_text};

/// Every draw command's spec: JSON parameters, menu path, aliases and the interactive prompt
/// machine each command starts.
pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("line", "Line", run_line)
            .menu(&["Draw", "Line"])
            .alias(&["l"])
            .params("{points: [[x,y],...], closed?: bool}")
            .interactive(|_| Ok(Box::new(LineM::default()))),
        CommandSpec::new("pline", "Polyline", run_pline)
            .menu(&["Draw", "Polyline"])
            .alias(&["pl"])
            .params("{vertices: [[x,y] | {p:[x,y], bulge, startWidth, endWidth}], closed?, width?}")
            .interactive(|_| Ok(Box::new(PlineM::default()))),
        CommandSpec::new("circle", "Circle", run_circle)
            .menu(&["Draw", "Circle", "Center, Radius"])
            .alias(&["c"])
            .params("{center, radius} | {center, diameter} | {p1, p2} | {p1, p2, p3}")
            .interactive(|_| Ok(Box::new(CircleM::default()))),
        CommandSpec::new("arc", "Arc", run_arc)
            .menu(&["Draw", "Arc", "3 Points"])
            .alias(&["a"])
            .params("{p1, p2, p3} | {center, radius, start, end (degrees)} | {start, center, end}")
            .interactive(|_| Ok(super::draw2::arc_machine())),
        CommandSpec::new("rectang", "Rectangle", super::draw2::run_rectang2)
            .menu(&["Draw", "Rectangle"])
            .alias(&["rec", "rectangle"])
            .params("{p1, p2 | dimensions: [l, w] (p2 picks the quadrant) | area + length|breadth, rotation? (deg), fillet?, chamfer?: [d1, d2], width?, elevation?, thickness?, center?: bool (p1 is the centre, not a corner; the size spans it evenly)}")
            .interactive(|_| Ok(Box::new(super::draw2::RectM2::default()))),
        CommandSpec::new("polygon", "Polygon", run_polygon)
            .menu(&["Draw", "Polygon"])
            .alias(&["pol"])
            .params("{sides, center, radius, inscribed?: bool, angle?: deg to a vertex (inscribed) or edge midpoint (default: bottom edge horizontal)} | {sides, edge: [[x,y],[x,y]]}")
            .interactive(|_| Ok(Box::new(PolygonM::default()))),
        CommandSpec::new("ellipse", "Ellipse", run_ellipse)
            .menu(&["Draw", "Ellipse", "Center"])
            .alias(&["el"])
            .params("{center, major: [dx,dy], ratio, start?, end? (degrees)} | {isocircle: {center, radius | diameter, plane?}}")
            .interactive(|_| Ok(Box::new(EllipseM::default()))),
        CommandSpec::new("point", "Point", run_point)
            .menu(&["Draw", "Point", "Single Point"])
            .alias(&["po"])
            .params("{at: [x,y]} | {points: [...]}")
            .interactive(|_| Ok(Box::new(PointM { multiple: false }))),
        CommandSpec::new("point.multiple", "Multiple Point", run_point)
            .menu(&["Draw", "Point", "Multiple Point"])
            .alias(&["multipoint", "mpoint"])
            .params("{points: [...]}")
            .interactive(|_| Ok(Box::new(PointM { multiple: true }))),
        CommandSpec::new("xline", "Construction Line", super::draw2::run_xline2)
            .menu(&["Draw", "Construction Line"])
            .alias(&["xl"])
            .params("{base, through} | {base, angle (degrees)} | {base, hor|ver: true} | {vertex, start, end} (bisect) | {handle, distance, side} | {handle, through} (offset)")
            .interactive(|_| Ok(Box::new(super::draw2::XlineM2::default()))),
        CommandSpec::new("ray", "Ray", run_ray)
            .menu(&["Draw", "Ray"])
            .params("{base, through}")
            .interactive(|_| Ok(Box::new(RayM::default()))),
        CommandSpec::new("spline", "Spline", run_spline)
            .menu(&["Draw", "Spline", "Fit Points"])
            .alias(&["spl"])
            .params("{fit: [[x,y],...], closed?, knots?: chord|sqrt|uniform, startTangent?, endTangent? ([dx,dy]), tolerance?} | {control: [...], degree?} | {object: handle|true, handles?} (polyline to spline)")
            .interactive(|_| Ok(Box::new(SplineM::default()))),
        CommandSpec::new("donut", "Donut", run_donut)
            .menu(&["Draw", "Donut"])
            .alias(&["do", "doughnut"])
            .params("{center, inside, outside}")
            .interactive(|_| Ok(Box::new(DonutM::default()))),
        CommandSpec::new("3dface", "3D Face", run_3dface)
            .menu(&["Draw", "3D Modeling", "3D Face"])
            .alias(&["3face"])
            .params("{points: [[x,y,z?], [x,y,z?], [x,y,z?], [x,y,z?]], hiddenEdges?: 0..15} (three points create a triangle)"),
        CommandSpec::new("mesh", "Facet Mesh", run_mesh)
            .menu(&["Draw", "3D Modeling", "Meshes"])
            .params("{vertices: [[x,y,z],...], faces: [[i,j,k] | [i,j,k,l],...] (zero-based vertex indices; stored as DXF-compatible 3DFACE facets)}"),
        CommandSpec::new("box", "Box", run_box)
            .menu(&["Draw", "3D Modeling", "Primitives", "Box"])
            .params("{base?: [x,y,z], length?, width?, height?, useUcs?: bool} -> 6 3DFACE facets"),
        CommandSpec::new("pyramid", "Pyramid", run_pyramid)
            .menu(&["Draw", "3D Modeling", "Primitives", "Pyramid"])
            .params("{base?: [x,y,z], length?, width?, height?, useUcs?: bool} -> 5 3DFACE facets"),
        CommandSpec::new("cylinder", "Cylinder", run_cylinder)
            .menu(&["Draw", "3D Modeling", "Primitives", "Cylinder"])
            .params("{center?: [x,y,z], radius?, height?, segments?: 8..128, useUcs?: bool} -> 3DFACE facets"),
        CommandSpec::new("cone", "Cone", run_cone)
            .menu(&["Draw", "3D Modeling", "Primitives", "Cone"])
            .params("{center?: [x,y,z], radius?, height?, segments?: 8..128, useUcs?: bool} -> 3DFACE facets"),
        CommandSpec::new("sphere", "Sphere", run_sphere)
            .menu(&["Draw", "3D Modeling", "Primitives", "Sphere"])
            .params("{center?: [x,y,z], radius?, segments?: 8..128, rings?: 4..64, useUcs?: bool} -> 3DFACE facets"),
        CommandSpec::new("torus", "Torus", run_torus)
            .menu(&["Draw", "3D Modeling", "Primitives", "Torus"])
            .params("{center?: [x,y,z], majorRadius?, minorRadius?, segments?: 8..128, tubeSegments?: 4..64, useUcs?: bool} -> 3DFACE facets"),
        CommandSpec::new("extrude", "Extrude", run_extrude)
            .menu(&["Draw", "3D Modeling", "Solid", "Extrude"])
            .params("{handle?: hex, height?: positive} (default: first selected closed polyline/circle) -> 3DFACE facets"),
        CommandSpec::new("revolve", "Revolve", run_revolve)
            .menu(&["Draw", "3D Modeling", "Solid", "Revolve"])
            .params("{handle?: hex, angle?: degrees, segments?: 8..128, axisOrigin?: [x,y,z]} (default: first selected profile; Z axis) -> 3DFACE facets"),
        CommandSpec::new("sweep", "Sweep", run_sweep)
            .menu(&["Draw", "3D Modeling", "Solid", "Sweep"])
            .params("{profile?: hex, path?: hex} (default: first two selected; section is translated along path without twist) -> 3DFACE facets"),
        CommandSpec::new("loft", "Loft", run_loft)
            .menu(&["Draw", "3D Modeling", "Solid", "Loft"])
            .params("{profiles?: [hex,hex,...], samples?: 8..128} (default: selected closed profiles; resampled and connected) -> 3DFACE facets"),
        CommandSpec::new("sectionplane", "Section Plane", run_sectionplane)
            .menu(&["Draw", "3D Modeling", "Section Plane"])
            .alias(&["section"])
            .params("{origin?: [x,y,z], normal?: [x,y,z], width?, height?, name?} -> persistent 3DFACE section marker with TBT360_SECTION xdata"),
        CommandSpec::new("sectionclip", "Section Clip", run_sectionclip)
            .menu(&["Draw", "3D Modeling", "Section Clip"])
            .params("{on?: bool, side?: positive|negative} (none: toggle) -> renderer section clipping state"),
        CommandSpec::new("sectiongeometry3d", "Generate Section Lines", run_sectiongeometry3d)
            .menu(&["Draw", "3D Modeling", "Section Lines"])
            .alias(&["sectionlines3d", "sectiongeometry"])
            .params("{planeHandle?:hex, handles?:[3DFACE handles], tolerance?:positive} -> native 3D LINE intersections"),
        CommandSpec::new("pointcloud", "Point Cloud", run_pointcloud)
            .menu(&["Draw", "3D Modeling", "Point Cloud"])
            .params("{points: [[x,y,z],...], name?: string, maxPoints?: 1..200000} -> native POINT entities tagged TBT360_POINTCLOUD"),
        CommandSpec::new("pointcloud.list", "List Point Clouds", run_pointcloud_list)
            .params("{} -> point-cloud groups with count and visibility"),
        CommandSpec::new("pointcloud.visible", "Set Point Cloud Visibility", run_pointcloud_visible)
            .params("{name: string, visible: bool}"),
        CommandSpec::new("pointcloud.delete", "Delete Point Cloud", run_pointcloud_delete)
            .params("{name: string} -> removes all native POINT entities in the named cloud"),
        CommandSpec::new("pointcloud.isolate", "Isolate Point Cloud", run_pointcloud_isolate)
            .params("{name: string} -> shows the named cloud and hides other TBT360 point-cloud groups"),
        CommandSpec::new("pointcloud.showall", "Show All Point Clouds", run_pointcloud_showall)
            .params("{} -> shows every TBT360 point-cloud group"),
        CommandSpec::new("pointcloud.selectvisible", "Select Visible Point Cloud", run_pointcloud_selectvisible)
            .params("{name:string, maxPoints?:1..200000} -> replaces selection with currently visible points from the named cloud"),
        CommandSpec::new("pointcloud.sample", "Sample Point Cloud", run_pointcloud_sample)
            .params("{name: string, keepEvery?: 1..1000} -> non-destructive display sampling using entity visibility"),
        CommandSpec::new("pointcloud.style", "Point Cloud Display Style", run_pointcloud_style)
            .params("{size?: 0.1..1000, mode?: 0..99} -> persisted point-cloud viewport point style"),
        CommandSpec::new("pointcloud.crop", "Crop Point Cloud by 3D Box", run_pointcloud_crop)
            .params("{name: string, min:[x,y,z], max:[x,y,z], invert?: bool} -> non-destructive visibility crop; point data is preserved"),
        CommandSpec::new("pointcloud.translate", "Translate Point Cloud", run_pointcloud_translate)
            .params("{name: string, delta:[dx,dy,dz]} -> moves every native POINT in the named cloud; undoable and persistent"),
        CommandSpec::new("pointcloud.scale", "Scale Point Cloud", run_pointcloud_scale)
            .params("{name: string, factor: positive, origin?:[x,y,z]} -> uniform scale; default origin is cloud centroid"),
        CommandSpec::new("pointcloud.rotate", "Rotate Point Cloud", run_pointcloud_rotate)
            .params("{name: string, angle: degrees, origin?:[x,y,z]} -> rotates the named cloud about the world Z axis; default origin is cloud centroid"),
        CommandSpec::new("pointcloud.recenter", "Recenter Point Cloud", run_pointcloud_recenter)
            .params("{name: string, target?:[x,y,z]} -> moves cloud centroid to target; default target is world origin"),
        CommandSpec::new("pointcloud.rename", "Rename Point Cloud", run_pointcloud_rename)
            .params("{name: string, newName: string} -> renames the TBT360_POINTCLOUD group without changing XYZ data"),
        CommandSpec::new("pointcloud.merge", "Merge Point Clouds", run_pointcloud_merge)
            .params("{source: string, target: string} -> retags source POINT entities into target group; XYZ data is preserved"),
        CommandSpec::new("pointcloud.bounds", "Point Cloud Bounds", run_pointcloud_bounds)
            .params("{name: string} -> count, visibleCount, centroid, min/max bounds and diagonal length"),
        CommandSpec::new("pointcloud.box3d", "Create Point Cloud 3D Bounds", run_pointcloud_box3d)
            .params("{name:string} -> creates 12 native 3D LINE edges around the cloud bounds"),
        CommandSpec::new("pointcloud.color", "Point Cloud Color", run_pointcloud_color)
            .params("{name: string, color: string|ACI} -> applies entity color to the named cloud; XYZ data is preserved"),
        CommandSpec::new("pointcloud.exportxyz", "Export Point Cloud XYZ", run_pointcloud_exportxyz)
            .params("{name: string, visibleOnly?: bool, delimiter?: comma|space|tab} -> bounded XYZ text export without modifying the drawing"),
        CommandSpec::new("pointcloud.rotate3d", "Rotate Point Cloud 3D", run_pointcloud_rotate3d)
            .params("{name: string, axis:[x,y,z], angle: degrees, origin?:[x,y,z]} -> Rodrigues rotation about an arbitrary 3D axis; default origin is cloud centroid"),
        CommandSpec::new("pointcloud.filterz", "Filter Point Cloud Elevation", run_pointcloud_filterz)
            .params("{name: string, minZ: number, maxZ: number, invert?: bool} -> non-destructive visibility filter by world Z"),
        CommandSpec::new("pointcloud.nearest", "Nearest Point Cloud Points", run_pointcloud_nearest)
            .params("{name: string, at:[x,y,z], maxResults?:1..64, visibleOnly?:bool} -> nearest native POINTs with distance; read-only")
            .noundo(),
        CommandSpec::new("pointcloud.radius", "Point Cloud Radius Query", run_pointcloud_radius)
            .params("{name: string, at:[x,y,z], radius: positive, maxResults?:1..4096, visibleOnly?:bool} -> native POINTs within radius sorted by distance; read-only")
            .noundo(),
        CommandSpec::new("pointcloud.voxel", "Point Cloud Voxel Display Filter", run_pointcloud_voxel)
            .params("{name:string, size:positive, visibleOnlySource?:bool} -> non-destructive one-point-per-voxel display filter; XYZ data is preserved"),
        CommandSpec::new("pointcloud.clipplane", "Point Cloud Clip Plane", run_pointcloud_clipplane)
            .params("{name:string, origin:[x,y,z], normal:[x,y,z], keepPositive?:bool} -> non-destructive arbitrary-plane visibility clip"),
        CommandSpec::new("pointcloud.sliceplane", "Point Cloud Slice Plane", run_pointcloud_sliceplane)
            .params("{name:string, origin:[x,y,z], normal:[x,y,z], halfThickness:positive, invert?:bool} -> non-destructive visibility band around an arbitrary plane"),
        CommandSpec::new("pointcloud.zstats", "Point Cloud Elevation Statistics", run_pointcloud_zstats)
            .params("{name:string, bins?:2..100, visibleOnly?:bool} -> Z min/max/mean and histogram; read-only")
            .noundo(),
        CommandSpec::new("pointcloud.exportply", "Export Point Cloud ASCII PLY", run_pointcloud_exportply)
            .params("{name:string, visibleOnly?:bool, maxPoints?:1..200000} -> ASCII PLY vertex text; read-only and dependency-free")
            .noundo(),
        CommandSpec::new("pointcloud.dedup", "Point Cloud Duplicate Check", run_pointcloud_dedup)
            .params("{name:string, tolerance?:positive, delete?:bool} -> reports duplicate native POINTs by tolerance cell; delete=true removes duplicates and preserves one representative"),
        CommandSpec::new("pointcloud.density", "Point Cloud Density Statistics", run_pointcloud_density)
            .params("{name:string, visibleOnly?:bool} -> count, bounds volume/area and approximate point density; read-only")
            .noundo(),
        CommandSpec::new("pointcloud.planefit", "Point Cloud Best-Fit Plane", run_pointcloud_planefit)
            .params("{name:string, visibleOnly?:bool} -> centroid, unit normal, RMS/max deviation and projected extents; read-only")
            .noundo(),
        CommandSpec::new("pointcloud.plane3d", "Create Point Cloud Best-Fit Plane", run_pointcloud_plane3d)
            .params("{name:string, visibleOnly?:bool, margin?:non-negative} -> native 3DFACE rectangle on the least-squares best-fit plane"),
        CommandSpec::new("pointcloud.copyvisible", "Copy Visible Point Cloud", run_pointcloud_copyvisible)
            .params("{name:string, newName:string} -> copies currently visible native POINTs into a new TBT360 point-cloud group without changing the source"),
        CommandSpec::new("pointcloud.gridindex", "Point Cloud Grid Index", run_pointcloud_gridindex)
            .params("{name:string, cellSize:positive, visibleOnly?:bool} -> occupied-cell count, max/mean occupancy and bounded cell summary; read-only")
            .noundo(),
        CommandSpec::new("pointcloud.classifyz", "Classify Point Cloud by Elevation", run_pointcloud_classifyz)
            .params("{name:string, breaks:[z1,z2,...], labels?:[string,...], visibleOnly?:bool} -> persists TBT360_POINTCLASS XData on POINT entities"),
        CommandSpec::new("pointcloud.icptranslate", "Register Point Clouds (ICP Translate)", run_pointcloud_icptranslate)
            .params("{source:string, target:string, iterations?:1..50, maxPairs?:100..20000, visibleOnly?:bool, apply?:bool} -> bounded translation-only ICP registration with RMS before/after; apply=true moves source"),
        CommandSpec::new("pointcloud.octreestats", "Point Cloud Octree Statistics", run_pointcloud_octreestats)
            .params("{name:string, maxDepth?:1..12, leafSize?:8..4096, visibleOnly?:bool} -> bounded octree node/leaf/depth occupancy statistics; read-only")
            .noundo(),
        CommandSpec::new("pointcloud.icprigid", "Register Point Clouds (Rigid ICP)", run_pointcloud_icprigid)
            .params("{source:string,target:string,iterations?:1..30,maxPairs?:100..12000,visibleOnly?:bool,apply?:bool} -> bounded rigid 6DoF ICP using Horn quaternion fit"),
        CommandSpec::new("pointcloud.octreebuild", "Build Persistent Point Cloud Octree", run_pointcloud_octreebuild)
            .params("{name:string,maxDepth?:1..12,leafSize?:8..4096,visibleOnly?:bool} -> persists octree leaf ids in POINT XData and document metadata"),
        CommandSpec::new("hiddenline3d", "3D Hidden-Line Documentation", run_hiddenline3d)
            .menu(&["Draw","3D Modeling","Documentation","Hidden-Line Projection"])
            .alias(&["hlr3d"])
            .params("{handles?:[3DFACE],view?:[x,y,z],origin?:[x,y,z],scale?:positive} -> native 2D LINE silhouette/crease projection; approximate, no z-buffer occlusion"),
        CommandSpec::new("surfacepatch", "Surface Patch (Faceted)", run_surfacepatch)
            .menu(&["Draw","3D Modeling","Surface","Patch"])
            .params("{corners:[[x,y,z];4],u?:1..64,v?:1..64} -> faceted bilinear surface as native 3DFACE grid"),
        CommandSpec::new("solidaudit3d", "Solid / Shell Audit", run_solidaudit3d)
            .menu(&["Draw","3D Modeling","Solid","Audit Faceted Shell"])
            .params("{handles?:[3DFACE],tolerance?:positive} -> manifold/boundary/nonmanifold counts and signed-volume estimate; read-only")
            .noundo(),
        CommandSpec::new("pbrmaterial", "PBR Material Metadata", run_pbrmaterial)
            .menu(&["Draw","3D Modeling","Materials","PBR Material Metadata"])
            .params("{handles?:[hex],baseColor?:string,metallic?:0..1,roughness?:0..1,opacity?:0..1,texture?:string,normalMap?:string} -> persistent TBT360_PBR XData metadata"),
        CommandSpec::new("facetstats3d", "3D Facet Statistics", run_facetstats3d)
            .menu(&["Draw", "3D Modeling", "Inspect", "Facet Statistics"])
            .alias(&["3dstats", "facetstats"])
            .params("{handles?:[3DFACE handles]} -> face/triangle/quad counts, area and edge-length statistics; read-only")
            .noundo(),
        CommandSpec::new("massprops3d", "3D Facet Mass Properties", run_massprops3d)
            .menu(&["Tools", "Inquiry", "3D Facet Mass Properties"])
            .alias(&["3dprops", "facetprops"])
            .params("{handles?: [hex,...]} (default: selection) -> faceCount, surfaceArea, signedVolume, volume, bounds; volume is meaningful only for a closed consistently oriented shell"),
        CommandSpec::new("facetcheck3d", "3D Facet Shell Check", run_facetcheck3d)
            .menu(&["Tools", "Inquiry", "3D Facet Shell Check"])
            .alias(&["3dcheck", "facetcheck"])
            .params("{handles?: [hex,...], tolerance?: positive} (default: selection) -> boundary/manifold/non-manifold edge counts and degenerate faces"),
        CommandSpec::new("facetflip3d", "Flip 3D Facet Orientation", run_facetflip3d)
            .menu(&["Modify", "3D", "Flip Facet Orientation"])
            .alias(&["3dflip", "facetflip"])
            .params("{handles?: [hex,...]} (default: selection) -> reverses 3DFACE winding for shell/back-face repair"),
        CommandSpec::new("facetnormals3d", "3D Facet Normal Report", run_facetnormals3d)
            .menu(&["Tools", "Inquiry", "3D Facet Normal Report"])
            .alias(&["3dnormals", "facetnormals"])
            .params("{handles?: [hex,...], maxFaces?: 1..5000} (default: selection) -> per-face centroid, unit normal and area"),
        CommandSpec::new("facetorientout3d", "Orient 3D Facets Outward", run_facetorientout3d)
            .menu(&["Modify", "3D", "Orient Facets Outward"])
            .alias(&["3dorientout", "facetorientout"])
            .params("{handles?: [hex,...]} (default: selection) -> flips selected 3DFACE winding whose normal points toward the selection centroid"),
        CommandSpec::new("facetcleanup3d", "Clean Degenerate 3D Facets", run_facetcleanup3d)
            .menu(&["Modify", "3D", "Clean Degenerate Facets"])
            .alias(&["3dcleanfacets", "facetcleanup"])
            .params("{handles?: [hex,...], tolerance?: positive, delete?: bool} (default: selection) -> reports and optionally deletes degenerate/tiny 3DFACE entities"),
        CommandSpec::new("facettransform3d", "Transform 3D Facets", run_facettransform3d)
            .menu(&["Modify", "3D", "Transform Facets"])
            .alias(&["3dtransform", "facettransform"])
            .params("{mode: move|scale|rotate, handles?: [hex,...], delta?:[x,y,z], factor?:positive, origin?:[x,y,z], axis?:[x,y,z], angle?:degrees} (default: selection) -> transforms native 3DFACE geometry"),
        CommandSpec::new("facetmirror3d", "Mirror 3D Facets", run_facetmirror3d)
            .menu(&["Modify", "3D", "Mirror Facets"])
            .alias(&["3dmirror", "facetmirror"])
            .params("{handles?: [hex,...], origin:[x,y,z], normal:[x,y,z]} (default: selection) -> mirrors native 3DFACE vertices across an arbitrary 3D plane"),
        CommandSpec::new("facetoffset3d", "Offset 3D Facets Along Normal", run_facetoffset3d)
            .menu(&["Modify", "3D", "Offset Facets Along Normal"])
            .alias(&["3doffsetfacet", "facetoffset"])
            .params("{handles?: [hex,...], distance:number} (default: selection) -> offsets each native 3DFACE along its own unit normal; faceted repair/modeling helper"),
        CommandSpec::new("facetdedup3d", "Remove Duplicate 3D Facets", run_facetdedup3d)
            .menu(&["Modify", "3D", "Remove Duplicate Facets"])
            .alias(&["3ddedup", "facetdedup"])
            .params("{handles?:[hex,...], tolerance?:positive, delete?:bool} (default: selection) -> detects geometrically duplicate 3DFACE entities and optionally deletes duplicates"),
        CommandSpec::new("facettriangulate3d", "Triangulate 3D Facets", run_facettriangulate3d)
            .menu(&["Modify", "3D", "Triangulate Facets"])
            .alias(&["3dtriangulate", "facettriangulate"])
            .params("{handles?:[hex,...]} (default: selection) -> splits quad 3DFACE entities into native triangle 3DFACEs; triangles remain unchanged"),
        CommandSpec::new("facetweld3d", "Weld 3D Facet Vertices", run_facetweld3d)
            .menu(&["Modify", "3D", "Weld Facet Vertices"])
            .alias(&["3dweld", "facetweld"])
            .params("{handles?:[hex,...], tolerance?:positive} (default: selection) -> snaps near-coincident selected 3DFACE vertices to a shared averaged position"),
        CommandSpec::new("facetboundary3d", "Extract 3D Facet Boundary", run_facetboundary3d)
            .menu(&["Modify", "3D", "Extract Facet Boundary"])
            .alias(&["3dboundary", "facetboundary"])
            .params("{handles?:[hex,...], tolerance?:positive} (default: selection) -> creates native 3D LINE entities for open boundary edges of selected 3DFACE shell"),
        CommandSpec::new("facetcrease3d", "Extract 3D Crease Edges", run_facetcrease3d)
            .menu(&["Modify", "3D", "Extract Crease Edges"])
            .alias(&["3dcrease", "facetcrease"])
            .params("{handles?:[hex,...], angle?:0..180, tolerance?:positive} -> creates native 3D LINE entities on manifold edges whose face-normal angle meets the threshold"),
        CommandSpec::new("facetsubdivide3d", "Subdivide 3D Facets", run_facetsubdivide3d)
            .menu(&["Modify", "3D", "Subdivide Facets"])
            .alias(&["3dsubdivide", "facetsubdivide"])
            .params("{handles?:[hex,...]} (default: selection) -> subdivides triangle/quad 3DFACE geometry into four native child facets while preserving source common properties"),
        CommandSpec::new("sectioncutstyle", "Section Cut Display", run_sectioncutstyle)
            .params("{lines?: bool} -> show/hide exact section-plane intersection edges on 3DFACE geometry"),
        CommandSpec::new("text3d", "3D Text Label", run_text3d)
            .menu(&["Draw", "3D Modeling", "3D Annotation", "Text Label"])
            .alias(&["3dtext"])
            .params("{at: [x,y,z], text: string, height?: positive} -> screen-facing TEXT anchored in 3D and tagged TBT360_3DANNOTATION"),
        CommandSpec::new("dim3d", "3D Distance Dimension", run_dim3d)
            .menu(&["Draw", "3D Modeling", "3D Annotation", "Distance Dimension"])
            .alias(&["3ddim", "dimension3d"])
            .params("{a: [x,y,z], b: [x,y,z], offset?: [dx,dy,dz], textHeight?: positive, precision?: 0..8} -> projected 3D witness/dimension LINEs + TEXT"),
        CommandSpec::new("dim3d.assoc", "Associative 3D Line Dimension", run_dim3d_assoc)
            .menu(&["Draw", "3D Modeling", "Annotation", "Associative 3D Line Dimension"])
            .alias(&["3ddimassoc", "dim3dassoc"])
            .params("{handle: LINE handle, offset?:[x,y,z], textHeight?:positive, precision?:0..8} -> creates native LINE/TEXT dimension linked to the source LINE by XData"),
        CommandSpec::new("dim3d.faceassoc", "Associative 3D Face Edge Dimension", run_dim3d_faceassoc)
            .menu(&["Draw", "3D Modeling", "Annotation", "Associative Face Edge Dimension"])
            .alias(&["3ddimface", "dim3dface"])
            .params("{handle:3DFACE handle, edge?:0..3, offset?:[x,y,z]} -> native LINE/TEXT dimension linked to a source face edge"),
        CommandSpec::new("dim3d.facerefresh", "Refresh 3D Face Edge Dimensions", run_dim3d_facerefresh)
            .menu(&["Draw", "3D Modeling", "Annotation", "Refresh Face Edge Dimensions"])
            .alias(&["3ddimfacerefresh"])
            .params("{} -> rebuilds all TBT360_DIM3D_FACE dimensions from current 3DFACE edges"),
        CommandSpec::new("dim3d.refresh", "Refresh Associative 3D Dimensions", run_dim3d_refresh)
            .menu(&["Draw", "3D Modeling", "Annotation", "Refresh Associative 3D Dimensions"])
            .alias(&["3ddimrefresh", "dim3drefresh"])
            .params("{} -> rebuilds all TBT360 associative 3D line dimensions from current source LINE geometry"),
        CommandSpec::new("leader3d", "3D Leader", run_leader3d)
            .menu(&["Draw", "3D Modeling", "3D Annotation", "Leader"])
            .alias(&["3dleader"])
            .params("{arrow: [x,y,z], landing: [x,y,z], dogleg?: [dx,dy,dz], text: string, textHeight?: positive} -> projected 3D LINE(s) + TEXT"),
        CommandSpec::new("dim3d.angle", "3D Angular Dimension", run_dim3d_angle)
            .menu(&["Draw", "3D Modeling", "3D Annotation", "Angular Dimension"])
            .alias(&["3dangle"])
            .params("{vertex:[x,y,z], a:[x,y,z], b:[x,y,z], offset?: positive, textHeight?: positive, precision?:0..8}"),
        CommandSpec::new("dim3d.radius", "3D Radius Dimension", run_dim3d_radius)
            .menu(&["Draw", "3D Modeling", "3D Annotation", "Radius Dimension"])
            .alias(&["3dradius"])
            .params("{center:[x,y,z], point:[x,y,z], textHeight?: positive, precision?:0..8}"),
        CommandSpec::new("text", "Single Line Text", run_text)
            .menu(&["Draw", "Text", "Single Line Text"])
            .alias(&["dt", "dtext", "-text"])
            .params("{at, text, height?, rotation? (degrees), justify?: L|C|R|M|TL|TC|TR|ML|MC|MR|BL|BC|BR}")
            .interactive(|s| Ok(Box::new(TextM::new(s)))),
        CommandSpec::new("mtext", "Multiline Text", run_mtext)
            .menu(&["Draw", "Text", "Multiline Text..."])
            .alias(&["t", "mt", "-mtext"])
            .params(
                "{at, text, height?, width?, attach?: 1..9 | justify?: TL..BR, rotation?, style?, lineSpacing?, lineSpacingExact?, \
                 list?: bullet|number|lower|upper, indent?, firstIndent?, rightIndent?, tabs?: [..], align?: left|center|right|justify|distributed, \
                 columns?: {type: static|dynamic, count?, gutter?, height, heights?, reversed?}} (indents and tabs in text heights; width = one column)",
            )
            .interactive(|_| Ok(Box::new(MTextM::default()))),
    ]
}

fn run_3dface(s: &mut Session, p: &Value) -> Result<Value> {
    let raw = p.get("points").and_then(Value::as_array).ok_or_else(|| bad("3dface", "`points` must contain 3 or 4 points"))?;
    if !(3..=4).contains(&raw.len()) { return Err(bad("3dface", "`points` must contain 3 or 4 points")); }
    let mut pts = Vec::with_capacity(4);
    for value in raw {
        let a = value.as_array().ok_or_else(|| bad("3dface", "each point must be [x,y,z?]"))?;
        if a.len() < 2 || a.len() > 3 { return Err(bad("3dface", "each point must be [x,y,z?]")); }
        let x = a[0].as_f64().ok_or_else(|| bad("3dface", "point coordinates must be numbers"))?;
        let y = a[1].as_f64().ok_or_else(|| bad("3dface", "point coordinates must be numbers"))?;
        let z = a.get(2).and_then(Value::as_f64).unwrap_or(0.0);
        let q = Vec3::new(x, y, z);
        if !q.is_finite() { return Err(bad("3dface", "point coordinates must be finite")); }
        pts.push(q);
    }
    if pts.len() == 3 { pts.push(*pts.last().unwrap_or(&Vec3::ZERO)); }
    let hidden_edges = p.get("hiddenEdges").and_then(Value::as_u64).unwrap_or(0);
    if hidden_edges > 15 { return Err(bad("3dface", "hiddenEdges must be between 0 and 15")); }
    let h = s.add_entity(EntityKind::Face3d(Face3d { corners: [pts[0], pts[1], pts[2], pts[3]], hidden_edges: hidden_edges as u8 }))?;
    added(h)
}

fn run_mesh(s: &mut Session, p: &Value) -> Result<Value> {
    let vertices = p.get("vertices").and_then(Value::as_array).ok_or_else(|| bad("mesh", "`vertices` is required"))?;
    let faces = p.get("faces").and_then(Value::as_array).ok_or_else(|| bad("mesh", "`faces` is required"))?;
    if vertices.is_empty() || faces.is_empty() { return Err(bad("mesh", "vertices and faces cannot be empty")); }
    let mut vs = Vec::with_capacity(vertices.len());
    for value in vertices {
        let a = value.as_array().ok_or_else(|| bad("mesh", "each vertex must be [x,y,z]"))?;
        if a.len() != 3 { return Err(bad("mesh", "each vertex must contain x,y,z")); }
        let q = Vec3::new(a[0].as_f64().ok_or_else(|| bad("mesh", "vertex coordinates must be numbers"))?, a[1].as_f64().ok_or_else(|| bad("mesh", "vertex coordinates must be numbers"))?, a[2].as_f64().ok_or_else(|| bad("mesh", "vertex coordinates must be numbers"))?);
        if !q.is_finite() { return Err(bad("mesh", "vertex coordinates must be finite")); }
        vs.push(q);
    }
    let mut parsed = Vec::with_capacity(faces.len());
    for face in faces {
        let f = face.as_array().ok_or_else(|| bad("mesh", "each face must contain 3 or 4 vertex indices"))?;
        if !(3..=4).contains(&f.len()) { return Err(bad("mesh", "each face must contain 3 or 4 vertex indices")); }
        let mut ids = [0usize; 4];
        for (i, value) in f.iter().enumerate() {
            let index = value.as_u64().ok_or_else(|| bad("mesh", "face indices must be non-negative integers"))? as usize;
            if index >= vs.len() { return Err(bad("mesh", "face index is outside vertices")); }
            ids[i] = index;
        }
        if f.len() == 3 { ids[3] = ids[2]; }
        parsed.push(ids);
    }
    let mut handles = Vec::with_capacity(parsed.len());
    let hidden_edges = p.get("hiddenEdges").and_then(Value::as_u64).unwrap_or(0);
    if hidden_edges > 15 { return Err(bad("mesh", "hiddenEdges must be between 0 and 15")); }
    for [a, b, c, d] in parsed {
        handles.push(s.add_entity(EntityKind::Face3d(Face3d { corners: [vs[a], vs[b], vs[c], vs[d]], hidden_edges: hidden_edges as u8 }))?.hex());
    }
    let face_count = handles.len();
    Ok(json!({"handles": handles, "vertices": vs.len(), "faces": face_count, "representation": "3DFACE facets"}))
}



fn vec3_value(v: &Value) -> Option<Vec3> {
    let a = v.as_array()?;
    if a.len() != 3 { return None; }
    let q = Vec3::new(a[0].as_f64()?, a[1].as_f64()?, a[2].as_f64()?);
    q.is_finite().then_some(q)
}

fn vec3_param_or(p: &Value, name: &str, default: Vec3) -> Result<Vec3> {
    let Some(v) = p.get(name) else { return Ok(default) };
    let a = v.as_array().ok_or_else(|| bad(name, "expected [x,y,z]"))?;
    if a.len() != 3 { return Err(bad(name, "expected [x,y,z]")); }
    let q = Vec3::new(
        a[0].as_f64().ok_or_else(|| bad(name, "coordinates must be numbers"))?,
        a[1].as_f64().ok_or_else(|| bad(name, "coordinates must be numbers"))?,
        a[2].as_f64().ok_or_else(|| bad(name, "coordinates must be numbers"))?,
    );
    if !q.is_finite() { return Err(bad(name, "coordinates must be finite")); }
    Ok(q)
}

fn positive_or(p: &Value, name: &str, default: f64) -> Result<f64> {
    let v = p.get(name).and_then(Value::as_f64).unwrap_or(default);
    if !v.is_finite() || v <= 0.0 { return Err(bad(name, "must be a positive finite number")); }
    Ok(v)
}

fn count_or(p: &Value, name: &str, default: usize, min: usize, max: usize) -> Result<usize> {
    let v = p.get(name).and_then(Value::as_u64).map(|n| n as usize).unwrap_or(default);
    if !(min..=max).contains(&v) { return Err(bad(name, format!("must be between {min} and {max}"))); }
    Ok(v)
}

fn apply_current_ucs(s: &Session, p: &Value, vertices: &mut [Vec3]) -> Result<()> {
    if !p.get("useUcs").and_then(Value::as_bool).unwrap_or(false) { return Ok(()); }
    let d=s.doc()?; let name=d.header.str("UCSNAME","World");
    if name.eq_ignore_ascii_case("World") { return Ok(()); }
    let Some(u)=d.ucss.iter().find(|u|u.name.eq_ignore_ascii_case(&name)) else { return Err(bad("useUcs",format!("current UCS `{name}` is missing"))); };
    let x=u.x_axis.normalized(); let y=u.y_axis.normalized(); let z=x.cross(y).normalized();
    if z==Vec3::ZERO { return Err(bad("useUcs","current UCS axes are degenerate")); }
    for q in vertices { *q=u.origin + x*q.x + y*q.y + z*q.z; }
    Ok(())
}

fn add_facets(s: &mut Session, vertices: &[Vec3], faces: &[[usize; 4]], primitive: &str) -> Result<Value> {
    let mut handles = Vec::with_capacity(faces.len());
    for [a, b, c, d] in faces {
        let h = s.add_entity(EntityKind::Face3d(Face3d {
            corners: [vertices[*a], vertices[*b], vertices[*c], vertices[*d]],
            hidden_edges: 0,
        }))?;
        handles.push(h.hex());
    }
    Ok(json!({
        "primitive": primitive,
        "representation": "3DFACE facets",
        "faces": handles.len(),
        "handles": handles,
        "acis": false
    }))
}

fn run_box(s: &mut Session, p: &Value) -> Result<Value> {
    let o = vec3_param_or(p, "base", Vec3::ZERO)?;
    let lx = positive_or(p, "length", 10.0)?;
    let ly = positive_or(p, "width", 10.0)?;
    let lz = positive_or(p, "height", 10.0)?;
    let mut v = vec![
        o,
        Vec3::new(o.x + lx, o.y, o.z),
        Vec3::new(o.x + lx, o.y + ly, o.z),
        Vec3::new(o.x, o.y + ly, o.z),
        Vec3::new(o.x, o.y, o.z + lz),
        Vec3::new(o.x + lx, o.y, o.z + lz),
        Vec3::new(o.x + lx, o.y + ly, o.z + lz),
        Vec3::new(o.x, o.y + ly, o.z + lz),
    ];
    let f = [[0, 1, 2, 3], [4, 7, 6, 5], [0, 4, 5, 1], [1, 5, 6, 2], [2, 6, 7, 3], [3, 7, 4, 0]];
    apply_current_ucs(s,p,&mut v)?;
    add_facets(s, &v, &f, "Box")
}

fn run_pyramid(s: &mut Session, p: &Value) -> Result<Value> {
    let o = vec3_param_or(p, "base", Vec3::ZERO)?;
    let lx = positive_or(p, "length", 10.0)?;
    let ly = positive_or(p, "width", 10.0)?;
    let lz = positive_or(p, "height", 10.0)?;
    let mut v = vec![
        o,
        Vec3::new(o.x + lx, o.y, o.z),
        Vec3::new(o.x + lx, o.y + ly, o.z),
        Vec3::new(o.x, o.y + ly, o.z),
        Vec3::new(o.x + lx * 0.5, o.y + ly * 0.5, o.z + lz),
    ];
    let f = [[0, 1, 2, 3], [0, 4, 1, 1], [1, 4, 2, 2], [2, 4, 3, 3], [3, 4, 0, 0]];
    apply_current_ucs(s,p,&mut v)?;
    add_facets(s, &v, &f, "Pyramid")
}

fn ring(center: Vec3, radius: f64, z: f64, count: usize) -> Vec<Vec3> {
    (0..count)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / count as f64;
            Vec3::new(center.x + radius * a.cos(), center.y + radius * a.sin(), center.z + z)
        })
        .collect()
}

fn run_cylinder(s: &mut Session, p: &Value) -> Result<Value> {
    let c = vec3_param_or(p, "center", Vec3::ZERO)?;
    let r = positive_or(p, "radius", 5.0)?;
    let h = positive_or(p, "height", 10.0)?;
    let n = count_or(p, "segments", 24, 8, 128)?;
    let mut v = ring(c, r, 0.0, n);
    v.extend(ring(c, r, h, n));
    let cb = v.len(); v.push(c);
    let ct = v.len(); v.push(Vec3::new(c.x, c.y, c.z + h));
    let mut f = Vec::with_capacity(n * 3);
    for i in 0..n {
        let j = (i + 1) % n;
        f.push([i, j, n + j, n + i]);
        f.push([cb, j, i, i]);
        f.push([ct, n + i, n + j, n + j]);
    }
    apply_current_ucs(s,p,&mut v)?;
    add_facets(s, &v, &f, "Cylinder")
}

fn run_cone(s: &mut Session, p: &Value) -> Result<Value> {
    let c = vec3_param_or(p, "center", Vec3::ZERO)?;
    let r = positive_or(p, "radius", 5.0)?;
    let h = positive_or(p, "height", 10.0)?;
    let n = count_or(p, "segments", 24, 8, 128)?;
    let mut v = ring(c, r, 0.0, n);
    let cb = v.len(); v.push(c);
    let apex = v.len(); v.push(Vec3::new(c.x, c.y, c.z + h));
    let mut f = Vec::with_capacity(n * 2);
    for i in 0..n {
        let j = (i + 1) % n;
        f.push([cb, j, i, i]);
        f.push([i, j, apex, apex]);
    }
    apply_current_ucs(s,p,&mut v)?;
    add_facets(s, &v, &f, "Cone")
}

fn run_sphere(s: &mut Session, p: &Value) -> Result<Value> {
    let c = vec3_param_or(p, "center", Vec3::ZERO)?;
    let r = positive_or(p, "radius", 5.0)?;
    let seg = count_or(p, "segments", 24, 8, 128)?;
    let rings = count_or(p, "rings", 12, 4, 64)?;
    let mut v = Vec::with_capacity((rings + 1) * seg);
    for j in 0..=rings {
        let phi = -std::f64::consts::FRAC_PI_2 + std::f64::consts::PI * j as f64 / rings as f64;
        let cr = r * phi.cos();
        let z = c.z + r * phi.sin();
        for i in 0..seg {
            let a = std::f64::consts::TAU * i as f64 / seg as f64;
            v.push(Vec3::new(c.x + cr * a.cos(), c.y + cr * a.sin(), z));
        }
    }
    let mut f = Vec::with_capacity(rings * seg);
    for j in 0..rings {
        for i in 0..seg {
            let k = (i + 1) % seg;
            f.push([j * seg + i, j * seg + k, (j + 1) * seg + k, (j + 1) * seg + i]);
        }
    }
    apply_current_ucs(s,p,&mut v)?;
    add_facets(s, &v, &f, "Sphere")
}

fn run_torus(s: &mut Session, p: &Value) -> Result<Value> {
    let c = vec3_param_or(p, "center", Vec3::ZERO)?;
    let major = positive_or(p, "majorRadius", 7.0)?;
    let minor = positive_or(p, "minorRadius", 2.0)?;
    if minor >= major { return Err(bad("minorRadius", "must be smaller than majorRadius")); }
    let seg = count_or(p, "segments", 24, 8, 128)?;
    let tube = count_or(p, "tubeSegments", 12, 4, 64)?;
    let mut v = Vec::with_capacity(seg * tube);
    for i in 0..seg {
        let a = std::f64::consts::TAU * i as f64 / seg as f64;
        let (ca, sa) = (a.cos(), a.sin());
        for j in 0..tube {
            let b = std::f64::consts::TAU * j as f64 / tube as f64;
            let rr = major + minor * b.cos();
            v.push(Vec3::new(c.x + rr * ca, c.y + rr * sa, c.z + minor * b.sin()));
        }
    }
    let mut f = Vec::with_capacity(seg * tube);
    for i in 0..seg {
        let ni = (i + 1) % seg;
        for j in 0..tube {
            let nj = (j + 1) % tube;
            f.push([i * tube + j, ni * tube + j, ni * tube + nj, i * tube + nj]);
        }
    }
    apply_current_ucs(s,p,&mut v)?;
    add_facets(s, &v, &f, "Torus")
}


fn selected_or_handle(s: &Session, p: &Value, cmd: &str) -> Result<cadcraft_doc::Handle> {
    if let Some(text) = p.get("handle").and_then(Value::as_str) {
        return cadcraft_doc::Handle::parse_hex(text).ok_or_else(|| bad(cmd, "invalid handle"));
    }
    s.selection().first().copied().ok_or_else(|| bad(cmd, "select a supported profile first"))
}

fn profile_points(s: &Session, h: cadcraft_doc::Handle, closed_required: bool, cmd: &str) -> Result<(Vec<Vec3>, bool)> {
    let e = s.doc()?.entity(h).ok_or_else(|| bad(cmd, "profile entity was not found"))?;
    match &e.kind {
        EntityKind::LwPolyline(pl) => {
            if closed_required && !pl.closed { return Err(bad(cmd, "profile must be a closed polyline or circle")); }
            let pts2 = Polyline { vertices: pl.vertices.clone(), closed: pl.closed }.tessellate(0.05);
            if pts2.len() < 2 { return Err(bad(cmd, "profile has too few points")); }
            Ok((pts2.into_iter().map(|q| Vec3::new(q.x, q.y, pl.elevation)).collect(), pl.closed))
        }
        EntityKind::Circle(c) => {
            let n = 48usize;
            let pts = (0..n).map(|i| {
                let a = std::f64::consts::TAU * i as f64 / n as f64;
                Vec3::new(c.center.x + c.radius * a.cos(), c.center.y + c.radius * a.sin(), c.center.z)
            }).collect();
            Ok((pts, true))
        }
        EntityKind::Polyline3d(pl) if !closed_required || pl.closed => {
            if pl.points.len() < 2 { return Err(bad(cmd, "profile has too few points")); }
            Ok((pl.points.clone(), pl.closed))
        }
        _ => Err(bad(cmd, "supported profiles are closed 2D polylines/circles; REVOLVE also accepts 3D polylines")),
    }
}

fn run_extrude(s: &mut Session, p: &Value) -> Result<Value> {
    let h = selected_or_handle(s, p, "extrude")?;
    let height = positive_or(p, "height", 10.0)?;
    let (base, closed) = profile_points(s, h, true, "extrude")?;
    if !closed || base.len() < 3 { return Err(bad("extrude", "profile must be closed")); }
    let n = base.len();
    let mut vertices = base.clone();
    vertices.extend(base.iter().map(|q| Vec3::new(q.x, q.y, q.z + height)));
    let mut faces = Vec::with_capacity(n + 2);
    for i in 0..n {
        let j = (i + 1) % n;
        faces.push([i, j, n + j, n + i]);
    }
    // Fan caps. Triangles are represented by repeating the last corner, matching 3DFACE semantics.
    for i in 1..n.saturating_sub(1) {
        faces.push([0, i + 1, i, i]);
        faces.push([n, n + i, n + i + 1, n + i + 1]);
    }
    add_facets(s, &vertices, &faces, "Extrude")
}

fn run_revolve(s: &mut Session, p: &Value) -> Result<Value> {
    let h = selected_or_handle(s, p, "revolve")?;
    let angle_deg = p.get("angle").and_then(Value::as_f64).unwrap_or(360.0);
    if !angle_deg.is_finite() || angle_deg.abs() < 1e-6 || angle_deg.abs() > 360.0 {
        return Err(bad("angle", "must be finite and between -360 and 360 degrees, excluding zero"));
    }
    let seg = count_or(p, "segments", 32, 8, 128)?;
    let axis = vec3_param_or(p, "axisOrigin", Vec3::ZERO)?;
    let (profile, closed_profile) = profile_points(s, h, false, "revolve")?;
    if profile.len() < 2 { return Err(bad("revolve", "profile has too few points")); }
    let full = (angle_deg.abs() - 360.0).abs() < 1e-9;
    let rings = if full { seg } else { seg + 1 };
    let mut vertices = Vec::with_capacity(rings * profile.len());
    for k in 0..rings {
        let t = if full { k as f64 / seg as f64 } else { k as f64 / seg as f64 };
        let a = angle_deg.to_radians() * t;
        let (ca, sa) = (a.cos(), a.sin());
        for q in &profile {
            let x = q.x - axis.x;
            let y = q.y - axis.y;
            vertices.push(Vec3::new(axis.x + x * ca - y * sa, axis.y + x * sa + y * ca, q.z));
        }
    }
    let m = profile.len();
    let strips = if full { rings } else { rings - 1 };
    let spans = if closed_profile { m } else { m - 1 };
    let mut faces = Vec::with_capacity(strips * spans);
    for k in 0..strips {
        let nk = if full { (k + 1) % rings } else { k + 1 };
        for i in 0..spans {
            let j = if closed_profile { (i + 1) % m } else { i + 1 };
            faces.push([k * m + i, k * m + j, nk * m + j, nk * m + i]);
        }
    }
    add_facets(s, &vertices, &faces, "Revolve")
}


fn path_points(s: &Session, h: cadcraft_doc::Handle, cmd: &str) -> Result<Vec<Vec3>> {
    let e = s.doc()?.entity(h).ok_or_else(|| bad(cmd, "path entity was not found"))?;
    match &e.kind {
        EntityKind::Polyline3d(pl) if pl.points.len() >= 2 => Ok(pl.points.clone()),
        EntityKind::LwPolyline(pl) if pl.vertices.len() >= 2 => {
            let pts2 = Polyline { vertices: pl.vertices.clone(), closed: pl.closed }.tessellate(0.05);
            Ok(pts2.into_iter().map(|q| Vec3::new(q.x, q.y, pl.elevation)).collect())
        }
        EntityKind::Line(l) => Ok(vec![l.a, l.b]),
        _ => Err(bad(cmd, "path must be a line, 2D polyline, or 3D polyline")),
    }
}

fn run_sweep(s: &mut Session, p: &Value) -> Result<Value> {
    let sel = s.selection();
    let parse_h = |name: &str| -> Result<Option<cadcraft_doc::Handle>> {
        match p.get(name).and_then(Value::as_str) {
            Some(v) => Ok(Some(cadcraft_doc::Handle::parse_hex(v).ok_or_else(|| bad("sweep", "invalid handle"))?)),
            None => Ok(None),
        }
    };
    let profile_h = parse_h("profile")?.or_else(|| sel.first().copied()).ok_or_else(|| bad("sweep", "select a closed profile then a path"))?;
    let path_h = parse_h("path")?.or_else(|| sel.get(1).copied()).ok_or_else(|| bad("sweep", "select a closed profile then a path"))?;
    if profile_h == path_h { return Err(bad("sweep", "profile and path must be different entities")); }
    let (profile, closed) = profile_points(s, profile_h, true, "sweep")?;
    if !closed || profile.len() < 3 { return Err(bad("sweep", "profile must be closed")); }
    let path = path_points(s, path_h, "sweep")?;
    let origin = path[0];
    let m = profile.len();
    let mut vertices = Vec::with_capacity(path.len() * m);
    for q in &path {
        let dx = q.x - origin.x; let dy = q.y - origin.y; let dz = q.z - origin.z;
        vertices.extend(profile.iter().map(|v| Vec3::new(v.x + dx, v.y + dy, v.z + dz)));
    }
    let mut faces = Vec::with_capacity((path.len() - 1) * m);
    for k in 0..path.len() - 1 {
        for i in 0..m {
            let j = (i + 1) % m;
            faces.push([k*m+i, k*m+j, (k+1)*m+j, (k+1)*m+i]);
        }
    }
    add_facets(s, &vertices, &faces, "Sweep")
}

fn loop_length(points: &[Vec3]) -> f64 {
    if points.len() < 2 { return 0.0; }
    (0..points.len()).map(|i| {
        let a=points[i]; let b=points[(i+1)%points.len()];
        ((b.x-a.x).powi(2)+(b.y-a.y).powi(2)+(b.z-a.z).powi(2)).sqrt()
    }).sum()
}

fn resample_loop(points: &[Vec3], n: usize) -> Vec<Vec3> {
    if points.is_empty() || n == 0 { return Vec::new(); }
    let total = loop_length(points);
    if total <= 1e-12 { return vec![points[0]; n]; }
    let mut out=Vec::with_capacity(n);
    let mut edge=0usize; let mut edge_start=0.0;
    let mut a=points[0]; let mut b=points[1%points.len()];
    let mut edge_len=((b.x-a.x).powi(2)+(b.y-a.y).powi(2)+(b.z-a.z).powi(2)).sqrt();
    for k in 0..n {
        let target=total*k as f64/n as f64;
        while target > edge_start + edge_len && edge + 1 < points.len() {
            edge_start += edge_len; edge += 1; a=points[edge]; b=points[(edge+1)%points.len()];
            edge_len=((b.x-a.x).powi(2)+(b.y-a.y).powi(2)+(b.z-a.z).powi(2)).sqrt();
        }
        let t=if edge_len<=1e-12 {0.0} else {(target-edge_start)/edge_len};
        out.push(Vec3::new(a.x+(b.x-a.x)*t, a.y+(b.y-a.y)*t, a.z+(b.z-a.z)*t));
    }
    out
}

fn run_loft(s: &mut Session, p: &Value) -> Result<Value> {
    let mut handles: Vec<cadcraft_doc::Handle> = if let Some(arr)=p.get("profiles").and_then(Value::as_array) {
        let mut out=Vec::new();
        for v in arr {
            let text=v.as_str().ok_or_else(|| bad("loft", "profiles must be handle strings"))?;
            out.push(cadcraft_doc::Handle::parse_hex(text).ok_or_else(|| bad("loft", "invalid profile handle"))?);
        }
        out
    } else { s.selection() };
    handles.dedup();
    if handles.len() < 2 { return Err(bad("loft", "select at least two closed profiles")); }
    if handles.len() > 64 { return Err(bad("loft", "at most 64 profiles are supported")); }
    let samples=count_or(p, "samples", 32, 8, 128)?;
    let mut rings=Vec::with_capacity(handles.len());
    for h in handles {
        let (pts, closed)=profile_points(s,h,true,"loft")?;
        if !closed || pts.len()<3 { return Err(bad("loft", "all profiles must be closed")); }
        rings.push(resample_loop(&pts,samples));
    }
    let mut vertices=Vec::with_capacity(rings.len()*samples);
    for ring in &rings { vertices.extend(ring.iter().copied()); }
    let mut faces=Vec::with_capacity((rings.len()-1)*samples);
    for k in 0..rings.len()-1 {
        for i in 0..samples {
            let j=(i+1)%samples;
            faces.push([k*samples+i,k*samples+j,(k+1)*samples+j,(k+1)*samples+i]);
        }
    }
    add_facets(s,&vertices,&faces,"Loft")
}


fn run_sectionplane(s: &mut Session, p: &Value) -> Result<Value> {
    let origin = vec3_param_or(p, "origin", Vec3::ZERO)?;
    let normal = vec3_param_or(p, "normal", Vec3::Z)?.normalized();
    if normal == Vec3::ZERO {
        return Err(bad("sectionplane", "normal must be a non-zero finite vector"));
    }
    let width = positive_or(p, "width", 20.0)?;
    let height = positive_or(p, "height", 12.0)?;
    let name = p.get("name").and_then(Value::as_str).unwrap_or("Section").trim();
    if name.is_empty() {
        return Err(bad("sectionplane", "name must not be empty"));
    }

    // Build a stable orthonormal basis on the plane. Pick the reference axis least parallel
    // to the plane normal to avoid a nearly-zero cross product.
    let reference = if normal.z.abs() < 0.9 { Vec3::Z } else { Vec3::new(0.0, 1.0, 0.0) };
    let u = reference.cross(normal).normalized();
    let v = normal.cross(u).normalized();
    if u == Vec3::ZERO || v == Vec3::ZERO {
        return Err(bad("sectionplane", "could not construct a section-plane basis"));
    }
    let hu = u * (width * 0.5);
    let hv = v * (height * 0.5);
    let corners = [origin - hu - hv, origin + hu - hv, origin + hu + hv, origin - hu + hv];
    let h = s.add_entity(EntityKind::Face3d(Face3d { corners, hidden_edges: 0 }))?;
    s.doc_mut()?.modify_entity(h, |e| {
        e.common.xdata.extend([
            RawTag { code: 1001, value: "TBT360_SECTION".into() },
            RawTag { code: 1000, value: name.to_string() },
            RawTag { code: 1010, value: origin.x.to_string() },
            RawTag { code: 1020, value: origin.y.to_string() },
            RawTag { code: 1030, value: origin.z.to_string() },
            RawTag { code: 1011, value: normal.x.to_string() },
            RawTag { code: 1021, value: normal.y.to_string() },
            RawTag { code: 1031, value: normal.z.to_string() },
            RawTag { code: 1040, value: width.to_string() },
            RawTag { code: 1041, value: height.to_string() },
        ]);
    })?;
    Ok(json!({
        "handle": h.hex(),
        "name": name,
        "origin": [origin.x, origin.y, origin.z],
        "normal": [normal.x, normal.y, normal.z],
        "width": width,
        "height": height,
        "representation": "3DFACE section marker",
        "clipping": false
    }))
}

fn section_xdata(e: &cadcraft_doc::Entity) -> Option<(Vec3, Vec3)> {
    let mut tagged = false;
    let mut ox=None; let mut oy=None; let mut oz=None;
    let mut nx=None; let mut ny=None; let mut nz=None;
    for t in &e.common.xdata {
        if t.code == 1001 && t.value == "TBT360_SECTION" { tagged=true; continue; }
        if !tagged { continue; }
        let v=t.value.parse::<f64>().ok();
        match t.code {1010=>ox=v,1020=>oy=v,1030=>oz=v,1011=>nx=v,1021=>ny=v,1031=>nz=v,_=>{}}
    }
    let o=Vec3::new(ox?,oy?,oz?);
    let n=Vec3::new(nx?,ny?,nz?).normalized();
    (n!=Vec3::ZERO).then_some((o,n))
}

fn run_sectiongeometry3d(s: &mut Session, p: &Value) -> Result<Value> {
    let tol=p.get("tolerance").and_then(Value::as_f64).unwrap_or(1e-8);
    if !tol.is_finite() || tol<=0.0 { return Err(bad("sectiongeometry3d","tolerance must be positive and finite")); }
    let explicit_plane=p.get("planeHandle").and_then(Value::as_str).and_then(cadcraft_doc::Handle::parse_hex);
    let (plane_handle,origin,normal)={
        let d=s.doc()?;
        let candidate = if let Some(h)=explicit_plane {
            d.entity(h).and_then(|e| section_xdata(e).map(|v|(h,v)))
        } else {
            d.model.iter().collect::<Vec<_>>().into_iter().rev().find_map(|e|section_xdata(e).map(|v|(e.handle,v)))
        }.ok_or_else(||bad("sectiongeometry3d","no TBT360 section plane found"))?;
        (candidate.0,(candidate.1).0,(candidate.1).1)
    };
    let handles=facet_handles(s,p,"sectiongeometry3d")?;
    let mut segments=Vec::<(Vec3,Vec3)>::new();
    {
        let d=s.doc()?;
        for h in handles.iter().copied().filter(|h|*h!=plane_handle) {
            let Some(e)=d.entity(h) else {continue;};
            let EntityKind::Face3d(f)=&e.kind else {continue;};
            let tri=(f.corners[3]-f.corners[2]).len()<=tol;
            let pts=if tri{vec![f.corners[0],f.corners[1],f.corners[2]]}else{f.corners.to_vec()};
            let mut hits=Vec::<Vec3>::new();
            for i in 0..pts.len() {
                let a=pts[i]; let b=pts[(i+1)%pts.len()];
                let da=(a-origin).dot(normal); let db=(b-origin).dot(normal);
                if da.abs()<=tol { hits.push(a); }
                if da*db < -tol*tol {
                    let t=da/(da-db);
                    hits.push(a+(b-a)*t);
                } else if db.abs()<=tol {
                    hits.push(b);
                }
            }
            let mut unique=Vec::<Vec3>::new();
            for q in hits {
                if !unique.iter().any(|u|(*u-q).len()<=tol){unique.push(q);}
            }
            if unique.len()>=2 {
                let mut best=(0usize,1usize,0.0);
                for i in 0..unique.len(){for j in i+1..unique.len(){let d=(unique[i]-unique[j]).len();if d>best.2{best=(i,j,d);}}}
                if best.2>tol {segments.push((unique[best.0],unique[best.1]));}
            }
        }
    }
    if segments.is_empty(){return Ok(json!({"planeHandle":plane_handle.hex(),"created":0,"reason":"no selected facets intersect the section plane"}));}
    let mut created=Vec::with_capacity(segments.len());
    for (a,b) in segments {
        let h=s.add_entity(EntityKind::Line(Line{a,b}))?;
        tag_3d_annotation(s,h,"section-line")?;
        created.push(h.hex());
    }
    s.touch();
    Ok(json!({"planeHandle":plane_handle.hex(),"created":created.len(),"createdHandles":created,"origin":[origin.x,origin.y,origin.z],"normal":[normal.x,normal.y,normal.z],"representation":"native 3D LINE section intersections"}))
}

fn run_sectionclip(s: &mut Session, p: &Value) -> Result<Value> {
    let current = s.doc()?.header.i64("SECTIONCLIP", 0) != 0;
    let on = p.get("on").and_then(Value::as_bool).unwrap_or(!current);
    let side_raw = p.get("side").and_then(Value::as_str).map(str::to_string)
        .unwrap_or_else(|| s.doc().map(|d| d.header.str("SECTIONCLIPSIDE", "positive").to_string()).unwrap_or_else(|_| "positive".into()));
    let side = match side_raw.trim().to_ascii_lowercase().as_str() {
        "positive" | "+" => "positive",
        "negative" | "-" => "negative",
        _ => return Err(bad("sectionclip", "side must be positive or negative")),
    };
    let d = s.doc_mut()?;
    d.header.set_i64("SECTIONCLIP", if on { 1 } else { 0 });
    d.header.set_str("SECTIONCLIPSIDE", side);
    s.touch();
    Ok(json!({"on":on,"side":side,"renderer":"3DFACE/3D entity half-space clip"}))
}



fn tag_3d_annotation(s: &mut Session, h: cadcraft_doc::Handle, role: &str) -> Result<()> {
    s.doc_mut()?.modify_entity(h, |e| e.common.xdata.extend([
        RawTag { code: 1001, value: "TBT360_3DANNOTATION".into() },
        RawTag { code: 1000, value: role.into() },
    ]))?;
    Ok(())
}

fn add_text3d_entity(s: &mut Session, at: Vec3, value: String, height: f64, role: &str) -> Result<cadcraft_doc::Handle> {
    let style = s.doc()?.header.str("TEXTSTYLE", "Standard");
    let h = s.add_entity(EntityKind::Text(Text {
        insert: at, align_pt: None, height, value, rotation: 0.0, width_factor: 1.0, oblique: 0.0, style,
        halign: HAlign::Center, valign: VAlign::Baseline,
    }))?;
    tag_3d_annotation(s, h, role)?;
    Ok(h)
}

fn run_dim3d_angle(s: &mut Session, p: &Value) -> Result<Value> {
    let vertex = vec3_param_or(p, "vertex", Vec3::ZERO)?;
    let a = p.get("a").ok_or_else(|| bad("dim3d.angle", "a is required")).and_then(|_| vec3_param_or(p, "a", Vec3::ZERO))?;
    let b = p.get("b").ok_or_else(|| bad("dim3d.angle", "b is required")).and_then(|_| vec3_param_or(p, "b", Vec3::ZERO))?;
    let va = a - vertex; let vb = b - vertex;
    let la=va.len(); let lb=vb.len();
    if la <= 1e-12 || lb <= 1e-12 { return Err(bad("dim3d.angle", "dimension rays must be non-zero")); }
    let ua=va/la; let ub=vb/lb;
    let angle=ua.dot(ub).clamp(-1.0,1.0).acos().to_degrees();
    let precision=p.get("precision").and_then(Value::as_u64).unwrap_or(2).min(8) as usize;
    let offset=positive_or(p,"offset",la.min(lb)*0.65)?;
    let text_height=positive_or(p,"textHeight",s.doc()?.header.f64("TEXTSIZE",0.2).max(0.2))?;
    let bis=(ua+ub).normalized();
    let text_at=if bis==Vec3::ZERO { vertex + ua*offset } else { vertex + bis*offset };
    let h1=s.add_entity(EntityKind::Line(Line{a:vertex,b:a}))?; tag_3d_annotation(s,h1,"dim3d-angle-ray")?;
    let h2=s.add_entity(EntityKind::Line(Line{a:vertex,b:b}))?; tag_3d_annotation(s,h2,"dim3d-angle-ray")?;
    let label=format!("{angle:.precision$}°");
    let ht=add_text3d_entity(s,text_at,label.clone(),text_height,"dim3d-angle-text")?;
    Ok(json!({"handles":[h1.hex(),h2.hex(),ht.hex()],"angle":angle,"text":label,"associative":false}))
}

fn run_dim3d_radius(s: &mut Session, p: &Value) -> Result<Value> {
    let center=vec3_param_or(p,"center",Vec3::ZERO)?;
    let point=p.get("point").ok_or_else(||bad("dim3d.radius","point is required")).and_then(|_|vec3_param_or(p,"point",Vec3::ZERO))?;
    let radius=(point-center).len();
    if radius<=1e-12 || !radius.is_finite(){return Err(bad("dim3d.radius","center and point must be different"));}
    let precision=p.get("precision").and_then(Value::as_u64).unwrap_or(2).min(8) as usize;
    let text_height=positive_or(p,"textHeight",s.doc()?.header.f64("TEXTSIZE",0.2).max(0.2))?;
    let line=s.add_entity(EntityKind::Line(Line{a:center,b:point}))?; tag_3d_annotation(s,line,"dim3d-radius-line")?;
    let label=format!("R {radius:.precision$}");
    let ht=add_text3d_entity(s,point,label.clone(),text_height,"dim3d-radius-text")?;
    Ok(json!({"handles":[line.hex(),ht.hex()],"radius":radius,"text":label,"associative":false}))
}

fn run_dim3d(s: &mut Session, p: &Value) -> Result<Value> {
    let a = vec3_param_or(p, "a", Vec3::ZERO)?;
    let b = p.get("b").ok_or_else(|| bad("dim3d", "b is required")).and_then(|_| vec3_param_or(p, "b", Vec3::ZERO))?;
    let d = b - a;
    let distance = (d.x*d.x + d.y*d.y + d.z*d.z).sqrt();
    if !distance.is_finite() || distance <= 1e-12 { return Err(bad("dim3d", "dimension points must be different")); }
    let offset = match p.get("offset") {
        Some(_) => vec3_param_or(p, "offset", Vec3::ZERO)?,
        None => Vec3::new(0.0, 0.0, (distance * 0.08).max(0.25)),
    };
    let aa = a + offset;
    let bb = b + offset;
    let text_height = positive_or(p, "textHeight", s.doc()?.header.f64("TEXTSIZE", 0.2).max(0.2))?;
    let precision = p.get("precision").and_then(Value::as_u64).unwrap_or_else(|| s.doc().map(|d| d.header.i64("DIMDEC", 2).clamp(0, 8) as u64).unwrap_or(2)).min(8) as usize;
    let label = format!("{distance:.precision$}");
    let mut handles = Vec::new();
    for (u, v, role) in [(a, aa, "dim3d-extension"), (b, bb, "dim3d-extension"), (aa, bb, "dim3d-line")] {
        let h = s.add_entity(EntityKind::Line(Line { a: u, b: v }))?;
        tag_3d_annotation(s, h, role)?;
        handles.push(h.hex());
    }
    let at = (aa + bb) * 0.5;
    let style = s.doc()?.header.str("TEXTSTYLE", "Standard");
    let th = s.add_entity(EntityKind::Text(Text {
        insert: at, align_pt: None, height: text_height, value: label.clone(), rotation: 0.0,
        width_factor: 1.0, oblique: 0.0, style, halign: HAlign::Center, valign: VAlign::Baseline,
    }))?;
    tag_3d_annotation(s, th, "dim3d-text")?;
    handles.push(th.hex());
    Ok(json!({"handles":handles,"distance":distance,"text":label,"a":[a.x,a.y,a.z],"b":[b.x,b.y,b.z],"offset":[offset.x,offset.y,offset.z],"associative":false,"representation":"native LINE/TEXT projected as 3D annotation"}))
}


fn dim3d_assoc_meta(e: &cadcraft_doc::Entity) -> Option<(cadcraft_doc::Handle, Vec3)> {
    let x=&e.common.xdata;
    for i in 0..x.len() {
        if x[i].code==1001 && x[i].value.eq_ignore_ascii_case("TBT360_DIM3D_ASSOC") {
            let source=x.get(i+1).filter(|t|t.code==1000).and_then(|t|cadcraft_doc::Handle::parse_hex(t.value.trim()))?;
            let off=x.get(i+2).filter(|t|t.code==1000)?.value.split(',').filter_map(|v|v.trim().parse::<f64>().ok()).collect::<Vec<_>>();
            if off.len()==3 { return Some((source,Vec3::new(off[0],off[1],off[2]))); }
        }
    }
    None
}

fn tag_dim3d_assoc(s:&mut Session,h:cadcraft_doc::Handle,source:cadcraft_doc::Handle,offset:Vec3)->Result<()> {
    s.doc_mut()?.modify_entity(h,|e|e.common.xdata.extend([
        RawTag{code:1001,value:"TBT360_DIM3D_ASSOC".into()},
        RawTag{code:1000,value:source.hex()},
        RawTag{code:1000,value:format!("{},{},{}",offset.x,offset.y,offset.z)},
    ]))?;
    Ok(())
}

fn source_line_points(s:&Session,h:cadcraft_doc::Handle,cmd:&str)->Result<(Vec3,Vec3)> {
    let e=s.doc()?.entity(h).ok_or_else(||bad(cmd,"source handle does not exist"))?;
    match &e.kind { EntityKind::Line(l)=>Ok((l.a,l.b)), _=>Err(bad(cmd,"source handle must be a LINE")) }
}

fn run_dim3d_assoc(s:&mut Session,p:&Value)->Result<Value>{
    let h=p.get("handle").and_then(Value::as_str).and_then(cadcraft_doc::Handle::parse_hex).or_else(||s.selection().first().copied()).ok_or_else(||bad("dim3d.assoc","handle is required or select one LINE"))?;
    let (a,b)=source_line_points(s,h,"dim3d.assoc")?;
    let distance=(b-a).len(); if distance<=1e-12{return Err(bad("dim3d.assoc","source LINE must have non-zero length"));}
    let offset=match p.get("offset"){Some(_)=>vec3_param_or(p,"offset",Vec3::ZERO)?,None=>Vec3::new(0.0,0.0,(distance*0.08).max(0.25))};
    let mut q=p.clone(); if let Value::Object(ref mut m)=q {m.insert("a".into(),json!([a.x,a.y,a.z]));m.insert("b".into(),json!([b.x,b.y,b.z]));m.insert("offset".into(),json!([offset.x,offset.y,offset.z]));}
    let out=run_dim3d(s,&q)?;
    let hs=out.get("handles").and_then(Value::as_array).cloned().unwrap_or_default();
    for v in hs {if let Some(x)=v.as_str().and_then(cadcraft_doc::Handle::parse_hex){tag_dim3d_assoc(s,x,h,offset)?;}}
    s.touch();
    Ok(json!({"sourceHandle":h.hex(),"distance":distance,"offset":[offset.x,offset.y,offset.z],"associative":true,"xdata":"TBT360_DIM3D_ASSOC","result":out}))
}

fn dim3d_face_meta(e:&cadcraft_doc::Entity)->Option<(cadcraft_doc::Handle,usize,Vec3)>{
    let x=&e.common.xdata;
    for i in 0..x.len(){
        if x[i].code==1001&&x[i].value.eq_ignore_ascii_case("TBT360_DIM3D_FACE"){
            let source=x.get(i+1).filter(|t|t.code==1000).and_then(|t|cadcraft_doc::Handle::parse_hex(t.value.trim()))?;
            let edge=x.get(i+2).filter(|t|t.code==1070).and_then(|t|t.value.parse::<usize>().ok())?;
            let off=x.get(i+3).filter(|t|t.code==1000)?.value.split(',').filter_map(|v|v.trim().parse::<f64>().ok()).collect::<Vec<_>>();
            if off.len()==3{return Some((source,edge,Vec3::new(off[0],off[1],off[2])));}
        }
    }
    None
}

fn tag_dim3d_face(s:&mut Session,h:cadcraft_doc::Handle,source:cadcraft_doc::Handle,edge:usize,offset:Vec3)->Result<()>{
    s.doc_mut()?.modify_entity(h,|e|e.common.xdata.extend([
        RawTag{code:1001,value:"TBT360_DIM3D_FACE".into()},
        RawTag{code:1000,value:source.hex()},
        RawTag{code:1070,value:edge.to_string()},
        RawTag{code:1000,value:format!("{},{},{}",offset.x,offset.y,offset.z)},
    ]))?;
    Ok(())
}

fn face_edge_points(s:&Session,h:cadcraft_doc::Handle,edge:usize,cmd:&str)->Result<(Vec3,Vec3)>{
    let e=s.doc()?.entity(h).ok_or_else(||bad(cmd,"source handle does not exist"))?;
    let EntityKind::Face3d(f)=&e.kind else{return Err(bad(cmd,"source handle must be a 3DFACE"));};
    if edge>3{return Err(bad(cmd,"edge must be 0..3"));}
    Ok((f.corners[edge],f.corners[(edge+1)%4]))
}

fn run_dim3d_faceassoc(s:&mut Session,p:&Value)->Result<Value>{
    let h=p.get("handle").and_then(Value::as_str).and_then(cadcraft_doc::Handle::parse_hex).or_else(||s.selection().first().copied()).ok_or_else(||bad("dim3d.faceassoc","handle is required or select one 3DFACE"))?;
    let edge=p.get("edge").and_then(Value::as_u64).unwrap_or(0).min(3) as usize;
    let (a,b)=face_edge_points(s,h,edge,"dim3d.faceassoc")?; let distance=(b-a).len(); if distance<=1e-12{return Err(bad("dim3d.faceassoc","selected face edge is degenerate"));}
    let offset=match p.get("offset"){Some(_)=>vec3_param_or(p,"offset",Vec3::ZERO)?,None=>Vec3::new(0.0,0.0,(distance*0.08).max(0.25))};
    let mut q=p.clone(); if let Value::Object(ref mut m)=q{m.insert("a".into(),json!([a.x,a.y,a.z]));m.insert("b".into(),json!([b.x,b.y,b.z]));m.insert("offset".into(),json!([offset.x,offset.y,offset.z]));}
    let out=run_dim3d(s,&q)?; for v in out.get("handles").and_then(Value::as_array).cloned().unwrap_or_default(){if let Some(x)=v.as_str().and_then(cadcraft_doc::Handle::parse_hex){tag_dim3d_face(s,x,h,edge,offset)?;}} s.touch();
    Ok(json!({"sourceHandle":h.hex(),"edge":edge,"distance":distance,"associative":true,"metadata":"TBT360_DIM3D_FACE","result":out}))
}

fn run_dim3d_facerefresh(s:&mut Session,_:&Value)->Result<Value>{
    let mut groups=std::collections::BTreeMap::<(cadcraft_doc::Handle,usize),(Vec3,Vec<cadcraft_doc::Handle>)>::new();
    {let d=s.doc()?;for e in d.model.iter(){if let Some((src,edge,off))=dim3d_face_meta(e){groups.entry((src,edge)).or_insert((off,Vec::new())).1.push(e.handle);}}}
    let mut refreshed=0usize; let mut missing=Vec::new();
    for ((src,edge),(off,arts)) in groups{let pts=face_edge_points(s,src,edge,"dim3d.facerefresh");for h in arts{let _=s.doc_mut()?.remove_entity(h);}match pts{Ok((a,b))=>{let out=run_dim3d(s,&json!({"a":[a.x,a.y,a.z],"b":[b.x,b.y,b.z],"offset":[off.x,off.y,off.z]}))?;for v in out.get("handles").and_then(Value::as_array).cloned().unwrap_or_default(){if let Some(x)=v.as_str().and_then(cadcraft_doc::Handle::parse_hex){tag_dim3d_face(s,x,src,edge,off)?;}}refreshed+=1;},Err(_)=>missing.push(json!({"source":src.hex(),"edge":edge}))}}
    if refreshed>0{s.touch();} Ok(json!({"refreshed":refreshed,"missingSources":missing,"association":"3DFACE edge index"}))
}

fn run_dim3d_refresh(s:&mut Session,_p:&Value)->Result<Value>{
    let mut groups=std::collections::BTreeMap::<cadcraft_doc::Handle,Vec3>::new(); let mut artifacts=Vec::new();
    {let d=s.doc()?; for e in d.model.iter() {if let Some((src,off))=dim3d_assoc_meta(e){groups.entry(src).or_insert(off);artifacts.push(e.handle);}}}
    if groups.is_empty(){return Ok(json!({"refreshed":0,"removed":0,"missingSources":[],"message":"No associative 3D dimensions found."}));}
    {let d=s.doc_mut()?;for h in &artifacts{d.remove_entity(*h);}}
    let mut refreshed=0usize;let mut missing=Vec::new();
    for (src,off) in groups {match source_line_points(s,src,"dim3d.refresh") {Ok((a,b))=>{let q=json!({"handle":src.hex(),"offset":[off.x,off.y,off.z]}); let _=run_dim3d_assoc(s,&q)?; let _=(a,b); refreshed+=1;},Err(_)=>missing.push(src.hex())}}
    s.touch(); Ok(json!({"refreshed":refreshed,"removed":artifacts.len(),"missingSources":missing,"associative":true}))
}

fn run_leader3d(s: &mut Session, p: &Value) -> Result<Value> {
    let arrow = p.get("arrow").ok_or_else(|| bad("leader3d", "arrow is required")).and_then(|_| vec3_param_or(p, "arrow", Vec3::ZERO))?;
    let landing = p.get("landing").ok_or_else(|| bad("leader3d", "landing is required")).and_then(|_| vec3_param_or(p, "landing", Vec3::ZERO))?;
    if (landing-arrow).len() <= 1e-12 { return Err(bad("leader3d", "arrow and landing must be different")); }
    let value = p.get("text").and_then(Value::as_str).unwrap_or("").trim();
    if value.is_empty() { return Err(bad("leader3d", "text must not be empty")); }
    if value.len() > 4096 { return Err(bad("leader3d", "text is too long")); }
    let text_height = positive_or(p, "textHeight", s.doc()?.header.f64("TEXTSIZE", 0.2).max(0.2))?;
    let lh = s.add_entity(EntityKind::Line(Line { a: arrow, b: landing }))?;
    tag_3d_annotation(s, lh, "leader3d-line")?;
    let dogleg = vec3_param_or(p, "dogleg", Vec3::ZERO)?;
    let text_at = if dogleg.len() > 1e-12 {
        let end = landing + dogleg;
        let dh=s.add_entity(EntityKind::Line(Line{a:landing,b:end}))?;
        tag_3d_annotation(s,dh,"leader3d-dogleg")?;
        end
    } else { landing };
    let style = s.doc()?.header.str("TEXTSTYLE", "Standard");
    let th = s.add_entity(EntityKind::Text(Text {
        insert: text_at, align_pt: None, height: text_height, value: value.to_string(), rotation: 0.0,
        width_factor: 1.0, oblique: 0.0, style, halign: HAlign::Left, valign: VAlign::Baseline,
    }))?;
    tag_3d_annotation(s, th, "leader3d-text")?;
    Ok(json!({"handles":[lh.hex(),th.hex()],"arrow":[arrow.x,arrow.y,arrow.z],"landing":[landing.x,landing.y,landing.z],"dogleg":[dogleg.x,dogleg.y,dogleg.z],"text":value,"associative":false,"representation":"native LINE/TEXT projected as 3D annotation"}))
}

fn run_text3d(s: &mut Session, p: &Value) -> Result<Value> {
    let at = vec3_param_or(p, "at", Vec3::ZERO)?;
    let value = p.get("text").and_then(Value::as_str).unwrap_or("").trim();
    if value.is_empty() { return Err(bad("text3d", "text must not be empty")); }
    if value.len() > 4096 { return Err(bad("text3d", "text is too long")); }
    let height = positive_or(p, "height", s.doc()?.header.f64("TEXTSIZE", 0.2).max(0.2))?;
    let style = s.doc()?.header.str("TEXTSTYLE", "Standard");
    let h = s.add_entity(EntityKind::Text(Text {
        insert: at,
        align_pt: None,
        height,
        value: value.to_string(),
        rotation: 0.0,
        width_factor: 1.0,
        oblique: 0.0,
        style,
        halign: HAlign::Left,
        valign: VAlign::Baseline,
    }))?;
    s.doc_mut()?.modify_entity(h, |e| e.common.xdata.extend([
        RawTag { code: 1001, value: "TBT360_3DANNOTATION".into() },
        RawTag { code: 1010, value: at.x.to_string() },
        RawTag { code: 1020, value: at.y.to_string() },
        RawTag { code: 1030, value: at.z.to_string() },
    ]))?;
    Ok(json!({"handle":h.hex(),"at":[at.x,at.y,at.z],"text":value,"height":height,"representation":"billboard TEXT anchored in 3D"}))
}


fn pointcloud_group(e: &cadcraft_doc::Entity) -> Option<&str> {
    let mut tagged = false;
    for t in &e.common.xdata {
        if t.code == 1001 && t.value == "TBT360_POINTCLOUD" { tagged = true; continue; }
        if tagged && t.code == 1000 { return Some(t.value.as_str()); }
    }
    None
}

fn run_pointcloud_list(s: &mut Session, _p: &Value) -> Result<Value> {
    #[derive(Default)] struct Stats { count:usize, visible:usize, min:Option<Vec3>, max:Option<Vec3>, sum:Vec3 }
    let mut groups: std::collections::BTreeMap<String, Stats> = std::collections::BTreeMap::new();
    for e in s.doc()?.model.iter() {
        let Some(name)=pointcloud_group(e) else { continue; };
        let EntityKind::Point(pt)=&e.kind else { continue; };
        let v=groups.entry(name.to_string()).or_default(); v.count+=1; if e.common.visible {v.visible+=1;} v.sum=v.sum+pt.p;
        v.min=Some(v.min.map_or(pt.p,|q|Vec3::new(q.x.min(pt.p.x),q.y.min(pt.p.y),q.z.min(pt.p.z))));
        v.max=Some(v.max.map_or(pt.p,|q|Vec3::new(q.x.max(pt.p.x),q.y.max(pt.p.y),q.z.max(pt.p.z))));
    }
    Ok(json!({"groups":groups.into_iter().map(|(name,v)|{let c=if v.count>0{v.sum/(v.count as f64)}else{Vec3::ZERO};let lo=v.min.unwrap_or(Vec3::ZERO);let hi=v.max.unwrap_or(Vec3::ZERO);json!({"name":name,"count":v.count,"visible":v.visible>0,"visibleCount":v.visible,"centroid":[c.x,c.y,c.z],"min":[lo.x,lo.y,lo.z],"max":[hi.x,hi.y,hi.z]})}).collect::<Vec<_>>() }))
}

fn run_pointcloud_visible(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.visible","name is required"));}
    let visible=p.get("visible").and_then(Value::as_bool).ok_or_else(||bad("pointcloud.visible","visible must be boolean"))?;
    let handles:Vec<_>=s.doc()?.model.iter().filter(|e|pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name))).map(|e|e.handle).collect();
    if handles.is_empty(){return Err(bad("pointcloud.visible","no such point cloud"));}
    let d=s.doc_mut()?;
    for h in &handles { d.modify_entity(*h, |e| e.common.visible=visible)?; }
    s.touch();
    Ok(json!({"name":name,"count":handles.len(),"visible":visible}))
}

fn run_pointcloud_isolate(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.isolate","name is required"));}
    let exists=s.doc()?.model.iter().any(|e|pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name)));
    if !exists{return Err(bad("pointcloud.isolate","no such point cloud"));}
    let handles:Vec<_>=s.doc()?.model.iter().filter(|e|pointcloud_group(e).is_some()).map(|e|e.handle).collect();
    let d=s.doc_mut()?;
    for h in &handles { d.modify_entity(*h,|e|e.common.visible=pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name)))?; }
    s.touch(); Ok(json!({"isolated":name,"pointCount":handles.len()}))
}

fn run_pointcloud_showall(s: &mut Session, _p: &Value) -> Result<Value> {
    let handles:Vec<_>=s.doc()?.model.iter().filter(|e|pointcloud_group(e).is_some()).map(|e|e.handle).collect();
    let d=s.doc_mut()?; for h in &handles { d.modify_entity(*h,|e|e.common.visible=true)?; }
    s.touch(); Ok(json!({"shown":handles.len()}))
}

fn run_pointcloud_sample(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.sample","name is required"));}
    let keep_every=p.get("keepEvery").and_then(Value::as_u64).unwrap_or(1).clamp(1,1000) as usize;
    let handles:Vec<_>=s.doc()?.model.iter()
        .filter(|e|pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name)))
        .map(|e|e.handle).collect();
    if handles.is_empty(){return Err(bad("pointcloud.sample","no such point cloud"));}
    let d=s.doc_mut()?;
    let mut visible=0usize;
    for (i,h) in handles.iter().enumerate(){
        let show=i%keep_every==0; if show { visible+=1; }
        d.modify_entity(*h,|e|e.common.visible=show)?;
    }
    s.touch();
    Ok(json!({"name":name,"count":handles.len(),"visible":visible,"keepEvery":keep_every,"nonDestructive":true}))
}


fn run_pointcloud_selectvisible(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.selectvisible","name is required"));}
    let max=p.get("maxPoints").and_then(Value::as_u64).unwrap_or(200_000).clamp(1,200_000) as usize;
    let handles:Vec<_>=s.doc()?.model.iter()
        .filter(|e|e.common.visible && pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name)) && matches!(&e.kind,EntityKind::Point(_)))
        .map(|e|e.handle).take(max+1).collect();
    if handles.is_empty(){return Err(bad("pointcloud.selectvisible","cloud has no visible points"));}
    if handles.len()>max{return Err(bad("pointcloud.selectvisible",format!("visible point count exceeds maxPoints ({max})")));}
    let count=handles.len();
    s.set_selection(handles);
    Ok(json!({"name":name,"selected":count,"selectionReplaced":true,"maxPoints":max}))
}

fn run_pointcloud_crop(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.crop","name is required"));}
    let parse = |key: &str| -> Result<Vec3> {
        let a=p.get(key).and_then(Value::as_array).ok_or_else(||bad("pointcloud.crop",format!("{key} must be [x,y,z]")))?;
        if a.len()!=3 { return Err(bad("pointcloud.crop",format!("{key} must be [x,y,z]"))); }
        let q=Vec3::new(a[0].as_f64().unwrap_or(f64::NAN),a[1].as_f64().unwrap_or(f64::NAN),a[2].as_f64().unwrap_or(f64::NAN));
        if !q.is_finite(){return Err(bad("pointcloud.crop",format!("{key} coordinates must be finite")));}
        Ok(q)
    };
    let a=parse("min")?; let b=parse("max")?;
    let lo=Vec3::new(a.x.min(b.x),a.y.min(b.y),a.z.min(b.z));
    let hi=Vec3::new(a.x.max(b.x),a.y.max(b.y),a.z.max(b.z));
    let invert=p.get("invert").and_then(Value::as_bool).unwrap_or(false);
    let points:Vec<_>=s.doc()?.model.iter().filter_map(|e|{
        if !pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name)){return None;}
        let EntityKind::Point(pt)=&e.kind else{return None;}; Some((e.handle,pt.p))
    }).collect();
    if points.is_empty(){return Err(bad("pointcloud.crop","no such point cloud"));}
    let d=s.doc_mut()?; let mut visible=0usize;
    for (h,q) in &points {
        let inside=q.x>=lo.x&&q.x<=hi.x&&q.y>=lo.y&&q.y<=hi.y&&q.z>=lo.z&&q.z<=hi.z;
        let show=if invert{!inside}else{inside}; if show{visible+=1;}
        d.modify_entity(*h,|e|e.common.visible=show)?;
    }
    s.touch();
    Ok(json!({"name":name,"count":points.len(),"visible":visible,"min":[lo.x,lo.y,lo.z],"max":[hi.x,hi.y,hi.z],"invert":invert,"nonDestructive":true}))
}

fn pointcloud_points(s: &Session, name: &str, command: &str) -> Result<Vec<(cadcraft_doc::Handle, Vec3)>> {
    let points: Vec<_> = s.doc()?.model.iter().filter_map(|e| {
        if !pointcloud_group(e).is_some_and(|n| n.eq_ignore_ascii_case(name)) { return None; }
        let EntityKind::Point(pt) = &e.kind else { return None; };
        Some((e.handle, pt.p))
    }).collect();
    if points.is_empty() { return Err(bad(command, "no such point cloud")); }
    Ok(points)
}

fn vec3_required(p: &Value, key: &str, command: &str) -> Result<Vec3> {
    let a = p.get(key).and_then(Value::as_array).ok_or_else(|| bad(command, format!("{key} must be [x,y,z]")))?;
    if a.len() != 3 { return Err(bad(command, format!("{key} must be [x,y,z]"))); }
    let q = Vec3::new(a[0].as_f64().unwrap_or(f64::NAN), a[1].as_f64().unwrap_or(f64::NAN), a[2].as_f64().unwrap_or(f64::NAN));
    if !q.is_finite() { return Err(bad(command, format!("{key} coordinates must be finite"))); }
    Ok(q)
}

fn run_pointcloud_translate(s: &mut Session, p: &Value) -> Result<Value> {
    let name = p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() { return Err(bad("pointcloud.translate", "name is required")); }
    let delta = vec3_required(p, "delta", "pointcloud.translate")?;
    let points = pointcloud_points(s, name, "pointcloud.translate")?;
    let d = s.doc_mut()?;
    for (h, _) in &points {
        d.modify_entity(*h, |e| if let EntityKind::Point(pt) = &mut e.kind { pt.p = pt.p + delta; })?;
    }
    s.touch();
    Ok(json!({"name":name,"count":points.len(),"delta":[delta.x,delta.y,delta.z],"persistent":true}))
}

fn run_pointcloud_scale(s: &mut Session, p: &Value) -> Result<Value> {
    let name = p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() { return Err(bad("pointcloud.scale", "name is required")); }
    let factor = p.get("factor").and_then(Value::as_f64).unwrap_or(f64::NAN);
    if !factor.is_finite() || factor <= 0.0 || factor > 1_000_000.0 { return Err(bad("pointcloud.scale", "factor must be positive and finite")); }
    let points = pointcloud_points(s, name, "pointcloud.scale")?;
    let centroid = points.iter().fold(Vec3::ZERO, |acc, (_, q)| acc + *q) / points.len() as f64;
    let origin = if p.get("origin").is_some() { vec3_required(p, "origin", "pointcloud.scale")? } else { centroid };
    let d = s.doc_mut()?;
    for (h, _) in &points {
        d.modify_entity(*h, |e| if let EntityKind::Point(pt) = &mut e.kind { pt.p = origin + (pt.p - origin) * factor; })?;
    }
    s.touch();
    Ok(json!({"name":name,"count":points.len(),"factor":factor,"origin":[origin.x,origin.y,origin.z],"persistent":true}))
}

fn run_pointcloud_rotate(s: &mut Session, p: &Value) -> Result<Value> {
    let name = p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() { return Err(bad("pointcloud.rotate", "name is required")); }
    let angle = p.get("angle").and_then(Value::as_f64).unwrap_or(f64::NAN);
    if !angle.is_finite() { return Err(bad("pointcloud.rotate", "angle must be finite degrees")); }
    let points = pointcloud_points(s, name, "pointcloud.rotate")?;
    let centroid = points.iter().fold(Vec3::ZERO, |acc, (_, q)| acc + *q) / points.len() as f64;
    let origin = if p.get("origin").is_some() { vec3_required(p, "origin", "pointcloud.rotate")? } else { centroid };
    let r = angle.to_radians(); let (sn, cs) = r.sin_cos();
    let d = s.doc_mut()?;
    for (h, _) in &points {
        d.modify_entity(*h, |e| if let EntityKind::Point(pt) = &mut e.kind {
            let q = pt.p - origin; pt.p = origin + Vec3::new(q.x * cs - q.y * sn, q.x * sn + q.y * cs, q.z);
        })?;
    }
    s.touch();
    Ok(json!({"name":name,"count":points.len(),"angle":angle,"origin":[origin.x,origin.y,origin.z],"axis":"world-z","persistent":true}))
}

fn run_pointcloud_recenter(s: &mut Session, p: &Value) -> Result<Value> {
    let name = p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() { return Err(bad("pointcloud.recenter", "name is required")); }
    let points = pointcloud_points(s, name, "pointcloud.recenter")?;
    let centroid = points.iter().fold(Vec3::ZERO, |acc, (_, q)| acc + *q) / points.len() as f64;
    let target = if p.get("target").is_some() { vec3_required(p, "target", "pointcloud.recenter")? } else { Vec3::ZERO };
    let delta = target - centroid;
    let d = s.doc_mut()?;
    for (h, _) in &points { d.modify_entity(*h, |e| if let EntityKind::Point(pt) = &mut e.kind { pt.p = pt.p + delta; })?; }
    s.touch();
    Ok(json!({"name":name,"count":points.len(),"fromCentroid":[centroid.x,centroid.y,centroid.z],"target":[target.x,target.y,target.z],"delta":[delta.x,delta.y,delta.z],"persistent":true}))
}

fn set_pointcloud_group_name(e: &mut cadcraft_doc::Entity, new_name: &str) {
    let mut app=false;
    for tag in &mut e.common.xdata {
        if tag.code==1001 && tag.value=="TBT360_POINTCLOUD" { app=true; continue; }
        if app && tag.code==1000 { tag.value=new_name.to_string(); break; }
    }
}

fn run_pointcloud_rename(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    let new_name=p.get("newName").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() || new_name.is_empty() { return Err(bad("pointcloud.rename", "name and newName are required")); }
    if name.eq_ignore_ascii_case(new_name) { return Ok(json!({"name":name,"newName":new_name,"changed":0,"persistent":true})); }
    let points=pointcloud_points(s,name,"pointcloud.rename")?;
    let target_exists=s.doc()?.model.iter().any(|e|pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(new_name)));
    if target_exists { return Err(bad("pointcloud.rename", "newName already exists; use pointcloud.merge")); }
    let d=s.doc_mut()?; for (h,_) in &points { d.modify_entity(*h,|e|set_pointcloud_group_name(e,new_name))?; }
    s.touch();
    Ok(json!({"name":name,"newName":new_name,"changed":points.len(),"persistent":true}))
}

fn run_pointcloud_merge(s: &mut Session, p: &Value) -> Result<Value> {
    let source=p.get("source").and_then(Value::as_str).unwrap_or("").trim();
    let target=p.get("target").and_then(Value::as_str).unwrap_or("").trim();
    if source.is_empty() || target.is_empty() { return Err(bad("pointcloud.merge", "source and target are required")); }
    if source.eq_ignore_ascii_case(target) { return Err(bad("pointcloud.merge", "source and target must differ")); }
    let points=pointcloud_points(s,source,"pointcloud.merge")?;
    let d=s.doc_mut()?; for (h,_) in &points { d.modify_entity(*h,|e|set_pointcloud_group_name(e,target))?; }
    s.touch();
    Ok(json!({"source":source,"target":target,"merged":points.len(),"persistent":true,"xyzPreserved":true}))
}

fn run_pointcloud_bounds(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.bounds","name is required"));}
    let points=pointcloud_points(s,name,"pointcloud.bounds")?;
    let mut lo=points[0].1; let mut hi=points[0].1; let mut sum=Vec3::ZERO; let mut visible=0usize;
    let d=s.doc()?;
    for (h,q) in &points {
        lo=Vec3::new(lo.x.min(q.x),lo.y.min(q.y),lo.z.min(q.z)); hi=Vec3::new(hi.x.max(q.x),hi.y.max(q.y),hi.z.max(q.z)); sum=sum+*q;
        if d.entity(*h).is_some_and(|e|e.common.visible){visible+=1;}
    }
    let c=sum/(points.len() as f64); let diag=(hi-lo).len();
    Ok(json!({"name":name,"count":points.len(),"visibleCount":visible,"centroid":[c.x,c.y,c.z],"min":[lo.x,lo.y,lo.z],"max":[hi.x,hi.y,hi.z],"diagonal":diag}))
}

fn run_pointcloud_box3d(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.box3d","name is required"));}
    let points=pointcloud_points(s,name,"pointcloud.box3d")?;
    let mut lo=points[0].1; let mut hi=points[0].1;
    for (_,q) in &points {
        lo=Vec3::new(lo.x.min(q.x),lo.y.min(q.y),lo.z.min(q.z));
        hi=Vec3::new(hi.x.max(q.x),hi.y.max(q.y),hi.z.max(q.z));
    }
    let v=[
        Vec3::new(lo.x,lo.y,lo.z), Vec3::new(hi.x,lo.y,lo.z),
        Vec3::new(hi.x,hi.y,lo.z), Vec3::new(lo.x,hi.y,lo.z),
        Vec3::new(lo.x,lo.y,hi.z), Vec3::new(hi.x,lo.y,hi.z),
        Vec3::new(hi.x,hi.y,hi.z), Vec3::new(lo.x,hi.y,hi.z),
    ];
    let edges=[(0,1),(1,2),(2,3),(3,0),(4,5),(5,6),(6,7),(7,4),(0,4),(1,5),(2,6),(3,7)];
    let mut created=Vec::with_capacity(12);
    for (a,b) in edges {
        let h=s.add_entity(EntityKind::Line(Line{a:v[a],b:v[b]}))?;
        tag_3d_annotation(s,h,"pointcloud-bounds")?;
        created.push(h.hex());
    }
    s.touch();
    Ok(json!({"name":name,"created":created.len(),"handles":created,"min":[lo.x,lo.y,lo.z],"max":[hi.x,hi.y,hi.z],"representation":"12 native 3D LINE bounds edges","persistent":true,"undoable":true}))
}

fn run_pointcloud_color(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.color","name is required"));}
    let raw=p.get("color").ok_or_else(||bad("pointcloud.color","color is required"))?;
    let color=match raw {
        Value::String(v)=>Color::parse(v),
        Value::Number(n)=>n.as_i64().and_then(|i|i16::try_from(i).ok()).map(Color::from_aci),
        _=>None,
    }.ok_or_else(||bad("pointcloud.color","color must be a valid named/true color or ACI index"))?;
    let points=pointcloud_points(s,name,"pointcloud.color")?; let d=s.doc_mut()?;
    for (h,_) in &points {d.modify_entity(*h,|e|e.common.color=color)?;}
    s.touch(); Ok(json!({"name":name,"changed":points.len(),"color":color.name(),"xyzPreserved":true,"persistent":true}))
}

fn run_pointcloud_exportxyz(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.exportxyz","name is required"));}
    let visible_only=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false);
    let delimiter=match p.get("delimiter").and_then(Value::as_str).unwrap_or("comma").to_ascii_lowercase().as_str(){"comma"=>",","space"=>" ","tab"=>"\t",_=>return Err(bad("pointcloud.exportxyz","delimiter must be comma, space or tab"))};
    let points=pointcloud_points(s,name,"pointcloud.exportxyz")?; let d=s.doc()?; let mut lines=Vec::with_capacity(points.len().min(200_000));
    for (h,q) in points {if visible_only && !d.entity(h).is_some_and(|e|e.common.visible){continue;} lines.push(format!("{:.12}{}{}{}{}",q.x,delimiter,q.y,delimiter,q.z));}
    let count=lines.len(); Ok(json!({"name":name,"count":count,"visibleOnly":visible_only,"delimiter":delimiter,"text":lines.join("\n"),"readOnly":true}))
}


fn run_pointcloud_rotate3d(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.rotate3d","name is required"));}
    let axis=vec3_required(p,"axis","pointcloud.rotate3d")?;
    let axis_len=axis.len();
    if axis_len<=1e-12{return Err(bad("pointcloud.rotate3d","axis must be non-zero"));}
    let k=axis/axis_len;
    let angle=p.get("angle").and_then(Value::as_f64).unwrap_or(f64::NAN);
    if !angle.is_finite(){return Err(bad("pointcloud.rotate3d","angle must be finite degrees"));}
    let points=pointcloud_points(s,name,"pointcloud.rotate3d")?;
    let centroid=points.iter().fold(Vec3::ZERO,|acc,(_,q)|acc+*q)/(points.len() as f64);
    let origin=if p.get("origin").is_some(){vec3_required(p,"origin","pointcloud.rotate3d")?}else{centroid};
    let r=angle.to_radians(); let (sn,cs)=r.sin_cos();
    let d=s.doc_mut()?;
    for (h,_) in &points {
        d.modify_entity(*h,|e|if let EntityKind::Point(pt)=&mut e.kind {
            let q=pt.p-origin;
            let rotated=q*cs+k.cross(q)*sn+k*(k.dot(q)*(1.0-cs));
            pt.p=origin+rotated;
        })?;
    }
    s.touch();
    Ok(json!({"name":name,"count":points.len(),"axis":[k.x,k.y,k.z],"angle":angle,"origin":[origin.x,origin.y,origin.z],"persistent":true}))
}

fn run_pointcloud_filterz(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.filterz","name is required"));}
    let a=p.get("minZ").and_then(Value::as_f64).unwrap_or(f64::NAN);
    let b=p.get("maxZ").and_then(Value::as_f64).unwrap_or(f64::NAN);
    if !a.is_finite()||!b.is_finite(){return Err(bad("pointcloud.filterz","minZ and maxZ must be finite"));}
    let lo=a.min(b); let hi=a.max(b); let invert=p.get("invert").and_then(Value::as_bool).unwrap_or(false);
    let points=pointcloud_points(s,name,"pointcloud.filterz")?;
    let d=s.doc_mut()?; let mut visible=0usize;
    for (h,q) in &points {
        let inside=q.z>=lo&&q.z<=hi; let show=if invert{!inside}else{inside}; if show{visible+=1;}
        d.modify_entity(*h,|e|e.common.visible=show)?;
    }
    s.touch();
    Ok(json!({"name":name,"count":points.len(),"visible":visible,"minZ":lo,"maxZ":hi,"invert":invert,"nonDestructive":true}))
}

fn run_pointcloud_voxel(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.voxel","name is required"));}
    let size=p.get("size").and_then(Value::as_f64).unwrap_or(f64::NAN);
    if !size.is_finite()||size<=0.0{return Err(bad("pointcloud.voxel","size must be positive and finite"));}
    let visible_only_source=p.get("visibleOnlySource").and_then(Value::as_bool).unwrap_or(false);
    let points=pointcloud_points(s,name,"pointcloud.voxel")?;
    let mut seen=std::collections::BTreeSet::new();
    let mut keep=std::collections::BTreeSet::new();
    {
        let d=s.doc()?;
        for (h,q) in &points {
            if visible_only_source && !d.entity(*h).is_some_and(|e|e.common.visible){continue;}
            let key=((q.x/size).floor() as i64,(q.y/size).floor() as i64,(q.z/size).floor() as i64);
            if seen.insert(key){keep.insert(*h);}
        }
    }
    let d=s.doc_mut()?;
    for (h,_) in &points { d.modify_entity(*h,|e|e.common.visible=keep.contains(h))?; }
    s.touch();
    Ok(json!({"name":name,"count":points.len(),"visible":keep.len(),"voxelSize":size,"nonDestructive":true,"xyzPreserved":true}))
}

fn run_pointcloud_clipplane(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.clipplane","name is required"));}
    let origin=vec3_required(p,"origin","pointcloud.clipplane")?;
    let normal=vec3_required(p,"normal","pointcloud.clipplane")?;
    let nlen=normal.len();
    if nlen<=1e-12{return Err(bad("pointcloud.clipplane","normal must be non-zero"));}
    let n=normal/nlen;
    let keep_positive=p.get("keepPositive").and_then(Value::as_bool).unwrap_or(true);
    let points=pointcloud_points(s,name,"pointcloud.clipplane")?;
    let d=s.doc_mut()?; let mut visible=0usize;
    for (h,q) in &points {
        let side=(*q-origin).dot(n);
        let show=if keep_positive{side>=-1e-12}else{side<=1e-12};
        if show{visible+=1;}
        d.modify_entity(*h,|e|e.common.visible=show)?;
    }
    s.touch();
    Ok(json!({"name":name,"count":points.len(),"visible":visible,"origin":[origin.x,origin.y,origin.z],"normal":[n.x,n.y,n.z],"keepPositive":keep_positive,"nonDestructive":true}))
}

fn facet_handles(s: &Session, p: &Value, cmd: &str) -> Result<Vec<cadcraft_doc::Handle>> {
    let handles: Vec<cadcraft_doc::Handle> = if let Some(raw) = p.get("handles").and_then(Value::as_array) {
        raw.iter().filter_map(Value::as_str).filter_map(cadcraft_doc::Handle::parse_hex).collect()
    } else {
        s.selection()
    };
    let valid: Vec<_> = {
        let d = s.doc()?;
        handles.into_iter().filter(|h| matches!(d.entity(*h).map(|e| &e.kind), Some(EntityKind::Face3d(_)))).collect()
    };
    if valid.is_empty() { return Err(bad(cmd, "select one or more 3DFACE entities")); }
    Ok(valid)
}

fn facet_selection_center(s: &Session, handles: &[cadcraft_doc::Handle]) -> Result<Vec3> {
    let d = s.doc()?;
    let mut sum = Vec3::ZERO;
    let mut n = 0usize;
    for h in handles {
        if let Some(e) = d.entity(*h) {
            if let EntityKind::Face3d(f) = &e.kind {
                for q in f.corners { sum = sum + q; n += 1; }
            }
        }
    }
    if n == 0 { return Err(bad("facettransform3d", "selection contains no 3DFACE vertices")); }
    Ok(sum / n as f64)
}

fn rotate_about_axis(q: Vec3, origin: Vec3, axis: Vec3, angle_rad: f64) -> Vec3 {
    let u = axis / axis.len();
    let v = q - origin;
    let (sn, cs) = angle_rad.sin_cos();
    origin + v * cs + u.cross(v) * sn + u * (u.dot(v) * (1.0 - cs))
}

fn run_facettransform3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles = facet_handles(s, p, "facettransform3d")?;
    let mode = p.get("mode").and_then(Value::as_str).unwrap_or("").trim().to_ascii_lowercase();
    let default_origin = facet_selection_center(s, &handles)?;
    let origin = p.get("origin").map(|_| vec3_required(p, "origin", "facettransform3d")).transpose()?.unwrap_or(default_origin);
    let mut transform: Box<dyn Fn(Vec3) -> Vec3> = match mode.as_str() {
        "move" => {
            let d = vec3_required(p, "delta", "facettransform3d")?;
            Box::new(move |q| q + d)
        }
        "scale" => {
            let f = p.get("factor").and_then(Value::as_f64).unwrap_or(f64::NAN);
            if !f.is_finite() || f <= 0.0 { return Err(bad("facettransform3d", "factor must be positive and finite")); }
            Box::new(move |q| origin + (q - origin) * f)
        }
        "rotate" => {
            let axis = vec3_required(p, "axis", "facettransform3d")?;
            if axis.len() <= 1e-12 { return Err(bad("facettransform3d", "axis must be non-zero")); }
            let angle = p.get("angle").and_then(Value::as_f64).unwrap_or(f64::NAN);
            if !angle.is_finite() { return Err(bad("facettransform3d", "angle must be finite")); }
            let rad = angle.to_radians();
            Box::new(move |q| rotate_about_axis(q, origin, axis, rad))
        }
        _ => return Err(bad("facettransform3d", "mode must be move, scale or rotate")),
    };
    let d = s.doc_mut()?;
    for h in &handles {
        d.modify_entity(*h, |e| if let EntityKind::Face3d(f) = &mut e.kind {
            for q in &mut f.corners { *q = transform(*q); }
        })?;
    }
    s.touch();
    Ok(json!({"mode":mode,"changed":handles.len(),"origin":[origin.x,origin.y,origin.z],"representation":"faceted 3DFACE","undoable":true,"persistent":true}))
}

fn run_facetmirror3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles = facet_handles(s, p, "facetmirror3d")?;
    let origin = vec3_required(p, "origin", "facetmirror3d")?;
    let normal = vec3_required(p, "normal", "facetmirror3d")?;
    let nlen = normal.len();
    if nlen <= 1e-12 { return Err(bad("facetmirror3d", "normal must be non-zero")); }
    let n = normal / nlen;
    let d = s.doc_mut()?;
    for h in &handles {
        d.modify_entity(*h, |e| if let EntityKind::Face3d(f) = &mut e.kind {
            for q in &mut f.corners {
                let signed = (*q - origin).dot(n);
                *q = *q - n * (2.0 * signed);
            }
            let [a, b, c, d] = f.corners;
            f.corners = if (d - c).len() <= 1e-12 { [a, c, b, b] } else { [a, d, c, b] };
            f.hidden_edges = ((f.hidden_edges & 0b0001) << 3) | ((f.hidden_edges & 0b0010) << 1) | ((f.hidden_edges & 0b0100) >> 1) | ((f.hidden_edges & 0b1000) >> 3);
        })?;
    }
    s.touch();
    Ok(json!({"changed":handles.len(),"origin":[origin.x,origin.y,origin.z],"normal":[n.x,n.y,n.z],"representation":"faceted 3DFACE","undoable":true,"persistent":true}))
}

fn run_pointcloud_nearest(s: &mut Session, p: &Value) -> Result<Value> {
    let name = p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() { return Err(bad("pointcloud.nearest", "name is required")); }
    let at = vec3_required(p, "at", "pointcloud.nearest")?;
    let max_results = p.get("maxResults").and_then(Value::as_u64).unwrap_or(1).clamp(1, 64) as usize;
    let visible_only = p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false);
    let points = pointcloud_points(s, name, "pointcloud.nearest")?;
    let d = s.doc()?;
    let mut ranked = Vec::new();
    for (h, q) in points {
        if visible_only && !d.entity(h).is_some_and(|e| e.common.visible) { continue; }
        ranked.push(((q - at).len(), h, q));
    }
    ranked.sort_by(|a,b| a.0.total_cmp(&b.0));
    let results: Vec<Value> = ranked.into_iter().take(max_results).map(|(distance,h,q)| json!({"handle":h.hex(),"point":[q.x,q.y,q.z],"distance":distance})).collect();
    Ok(json!({"name":name,"at":[at.x,at.y,at.z],"visibleOnly":visible_only,"results":results,"readOnly":true}))
}

fn run_pointcloud_radius(s: &mut Session, p: &Value) -> Result<Value> {
    let name = p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() { return Err(bad("pointcloud.radius", "name is required")); }
    let at = vec3_required(p, "at", "pointcloud.radius")?;
    let radius = p.get("radius").and_then(Value::as_f64).unwrap_or(f64::NAN);
    if !radius.is_finite() || radius <= 0.0 { return Err(bad("pointcloud.radius", "radius must be positive and finite")); }
    let max_results = p.get("maxResults").and_then(Value::as_u64).unwrap_or(256).clamp(1, 4096) as usize;
    let visible_only = p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false);
    let points = pointcloud_points(s, name, "pointcloud.radius")?;
    let d = s.doc()?;
    let mut ranked = Vec::new();
    for (h, q) in points {
        if visible_only && !d.entity(h).is_some_and(|e| e.common.visible) { continue; }
        let distance = (q - at).len();
        if distance <= radius { ranked.push((distance, h, q)); }
    }
    ranked.sort_by(|a,b| a.0.total_cmp(&b.0));
    let total = ranked.len();
    let results: Vec<Value> = ranked.into_iter().take(max_results).map(|(distance,h,q)| json!({"handle":h.hex(),"point":[q.x,q.y,q.z],"distance":distance})).collect();
    Ok(json!({"name":name,"at":[at.x,at.y,at.z],"radius":radius,"visibleOnly":visible_only,"matches":total,"truncated":total>results.len(),"results":results,"readOnly":true}))
}


fn run_pointcloud_sliceplane(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.sliceplane","name is required"));}
    let origin=vec3_required(p,"origin","pointcloud.sliceplane")?;
    let normal=vec3_required(p,"normal","pointcloud.sliceplane")?;
    let nlen=normal.len();
    if nlen<=1e-12{return Err(bad("pointcloud.sliceplane","normal must be non-zero"));}
    let n=normal/nlen;
    let half=p.get("halfThickness").and_then(Value::as_f64).unwrap_or(f64::NAN);
    if !half.is_finite()||half<=0.0{return Err(bad("pointcloud.sliceplane","halfThickness must be positive and finite"));}
    let invert=p.get("invert").and_then(Value::as_bool).unwrap_or(false);
    let points=pointcloud_points(s,name,"pointcloud.sliceplane")?;
    let d=s.doc_mut()?; let mut visible=0usize;
    for (h,q) in &points {
        let in_band=((*q-origin).dot(n)).abs()<=half;
        let show=if invert{!in_band}else{in_band};
        if show{visible+=1;}
        d.modify_entity(*h,|e|e.common.visible=show)?;
    }
    s.touch();
    Ok(json!({"name":name,"count":points.len(),"visible":visible,"origin":[origin.x,origin.y,origin.z],"normal":[n.x,n.y,n.z],"halfThickness":half,"invert":invert,"nonDestructive":true}))
}

fn run_facetoffset3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles=facet_handles(s,p,"facetoffset3d")?;
    let distance=p.get("distance").and_then(Value::as_f64).unwrap_or(f64::NAN);
    if !distance.is_finite(){return Err(bad("facetoffset3d","distance must be finite"));}
    let d=s.doc_mut()?; let mut changed=0usize; let mut skipped=0usize;
    for h in &handles {
        let mut moved=false;
        d.modify_entity(*h,|e| if let EntityKind::Face3d(f)=&mut e.kind {
            let n=(f.corners[1]-f.corners[0]).cross(f.corners[2]-f.corners[0]);
            let len=n.len();
            if len>1e-12 { let delta=(n/len)*distance; for q in &mut f.corners{*q=*q+delta;} moved=true; }
        })?;
        if moved{changed+=1}else{skipped+=1;}
    }
    if changed>0{s.touch();}
    Ok(json!({"changed":changed,"skippedDegenerate":skipped,"distance":distance,"representation":"faceted 3DFACE","undoable":true,"persistent":true,"warning":"offset is per-face normal, not B-Rep shell offset"}))
}

fn run_facetcleanup3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles:Vec<cadcraft_doc::Handle>=if let Some(raw)=p.get("handles").and_then(Value::as_array){
        raw.iter().filter_map(Value::as_str).filter_map(cadcraft_doc::Handle::parse_hex).collect()
    }else{s.selection()};
    if handles.is_empty(){return Err(bad("facetcleanup3d","select one or more 3DFACE entities"));}
    let tol=p.get("tolerance").and_then(Value::as_f64).unwrap_or(1e-8);
    if !tol.is_finite()||tol<=0.0{return Err(bad("facetcleanup3d","tolerance must be positive and finite"));}
    let delete=p.get("delete").and_then(Value::as_bool).unwrap_or(false);
    let bad_handles:Vec<_>={let d=s.doc()?; handles.into_iter().filter(|h|{
        let Some(e)=d.entity(*h) else{return false;}; let EntityKind::Face3d(f)=&e.kind else{return false;};
        let a=(f.corners[1]-f.corners[0]).cross(f.corners[2]-f.corners[0]).len()*0.5;
        let b=if (f.corners[3]-f.corners[2]).len()>tol {(f.corners[2]-f.corners[0]).cross(f.corners[3]-f.corners[0]).len()*0.5}else{0.0};
        a+b<=tol*tol
    }).collect()};
    if delete&&!bad_handles.is_empty(){let d=s.doc_mut()?; for h in &bad_handles{d.remove_entity(*h);} s.touch();}
    Ok(json!({"degenerate":bad_handles.len(),"deleted":if delete{bad_handles.len()}else{0},"handles":bad_handles.iter().map(|h|h.hex()).collect::<Vec<_>>(),"tolerance":tol,"representation":"faceted 3DFACE","undoable":delete}))
}

fn run_facetnormals3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles: Vec<cadcraft_doc::Handle> = if let Some(raw)=p.get("handles").and_then(Value::as_array) {
        raw.iter().filter_map(Value::as_str).filter_map(cadcraft_doc::Handle::parse_hex).collect()
    } else { s.selection() };
    if handles.is_empty() { return Err(bad("facetnormals3d", "select one or more 3DFACE entities")); }
    let max_faces=p.get("maxFaces").and_then(Value::as_u64).unwrap_or(1000).clamp(1,5000) as usize;
    let d=s.doc()?; let mut out=Vec::new(); let mut total_area=0.0; let mut considered=0usize;
    for h in handles {
        let Some(e)=d.entity(h) else {continue;}; let EntityKind::Face3d(f)=&e.kind else {continue;}; considered+=1;
        let a=f.corners[0]; let b=f.corners[1]; let c=f.corners[2]; let n=(b-a).cross(c-a); let nlen=n.len();
        let tri=(f.corners[3]-f.corners[2]).len()<=1e-12;
        let area=if tri {nlen*0.5} else {nlen*0.5 + (f.corners[2]-a).cross(f.corners[3]-a).len()*0.5};
        total_area+=area;
        if out.len()<max_faces {
            let centroid=if tri {(a+b+c)/3.0} else {(a+b+c+f.corners[3])/4.0};
            let unit=if nlen>1e-12 {n/nlen} else {Vec3::ZERO};
            out.push(json!({"handle":h.hex(),"centroid":[centroid.x,centroid.y,centroid.z],"normal":[unit.x,unit.y,unit.z],"area":area,"degenerate":nlen<=1e-12}));
        }
    }
    if considered==0 { return Err(bad("facetnormals3d", "selection contains no 3DFACE entities")); }
    Ok(json!({"faceCount":considered,"reported":out.len(),"truncated":considered>out.len(),"totalArea":total_area,"faces":out,"representation":"faceted 3DFACE"}))
}

fn run_facetflip3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles: Vec<cadcraft_doc::Handle> = if let Some(raw) = p.get("handles").and_then(Value::as_array) {
        raw.iter().filter_map(Value::as_str).filter_map(cadcraft_doc::Handle::parse_hex).collect()
    } else { s.selection() };
    if handles.is_empty() { return Err(bad("facetflip3d", "select one or more 3DFACE entities")); }
    let valid: Vec<_> = {
        let d = s.doc()?;
        handles.into_iter().filter(|h| matches!(d.entity(*h).map(|e| &e.kind), Some(EntityKind::Face3d(_)))).collect()
    };
    if valid.is_empty() { return Err(bad("facetflip3d", "selection contains no 3DFACE entities")); }
    let d = s.doc_mut()?;
    for h in &valid {
        d.modify_entity(*h, |e| if let EntityKind::Face3d(f) = &mut e.kind {
            let [a, b, c, d] = f.corners;
            f.corners = if (d - c).len() <= 1e-12 { [a, c, b, b] } else { [a, d, c, b] };
            f.hidden_edges = ((f.hidden_edges & 0b0001) << 3) | ((f.hidden_edges & 0b0010) << 1) | ((f.hidden_edges & 0b0100) >> 1) | ((f.hidden_edges & 0b1000) >> 3);
        })?;
    }
    s.touch();
    Ok(json!({"flipped":valid.len(),"representation":"faceted 3DFACE","undoable":true}))
}

fn run_facetorientout3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles: Vec<cadcraft_doc::Handle> = if let Some(raw)=p.get("handles").and_then(Value::as_array){
        raw.iter().filter_map(Value::as_str).filter_map(cadcraft_doc::Handle::parse_hex).collect()
    } else { s.selection() };
    if handles.is_empty(){return Err(bad("facetorientout3d","select two or more 3DFACE entities"));}
    let valid:Vec<_>={let d=s.doc()?; handles.into_iter().filter(|h|matches!(d.entity(*h).map(|e|&e.kind),Some(EntityKind::Face3d(_)))).collect()};
    if valid.len()<2{return Err(bad("facetorientout3d","select two or more 3DFACE entities"));}
    let center={
        let d=s.doc()?; let mut sum=Vec3::ZERO; let mut n=0usize;
        for h in &valid {if let Some(e)=d.entity(*h){if let EntityKind::Face3d(f)=&e.kind {let tri=(f.corners[3]-f.corners[2]).len()<=1e-12; let c=if tri{(f.corners[0]+f.corners[1]+f.corners[2])/3.0}else{(f.corners[0]+f.corners[1]+f.corners[2]+f.corners[3])/4.0}; sum=sum+c; n+=1;}}}
        sum/(n as f64)
    };
    let mut flip=Vec::new(); {let d=s.doc()?; for h in &valid {if let Some(e)=d.entity(*h){if let EntityKind::Face3d(f)=&e.kind {let tri=(f.corners[3]-f.corners[2]).len()<=1e-12; let c=if tri{(f.corners[0]+f.corners[1]+f.corners[2])/3.0}else{(f.corners[0]+f.corners[1]+f.corners[2]+f.corners[3])/4.0}; let n=(f.corners[1]-f.corners[0]).cross(f.corners[2]-f.corners[0]); if n.len()>1e-12 && n.dot(c-center)<0.0 {flip.push(*h);}}}}}
    if !flip.is_empty(){let d=s.doc_mut()?; for h in &flip {d.modify_entity(*h,|e|if let EntityKind::Face3d(f)=&mut e.kind {let [a,b,c,d]=f.corners; f.corners=if (d-c).len()<=1e-12{[a,c,b,b]}else{[a,d,c,b]}; f.hidden_edges=((f.hidden_edges&1)<<3)|((f.hidden_edges&2)<<1)|((f.hidden_edges&4)>>1)|((f.hidden_edges&8)>>3);})?;} s.touch();}
    Ok(json!({"faceCount":valid.len(),"flipped":flip.len(),"center":[center.x,center.y,center.z],"representation":"faceted 3DFACE","heuristic":"selection-centroid outward orientation","undoable":true}))
}

fn run_facetdedup3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles=facet_handles(s,p,"facetdedup3d")?;
    let tol=p.get("tolerance").and_then(Value::as_f64).unwrap_or(1e-8);
    if !tol.is_finite()||tol<=0.0{return Err(bad("facetdedup3d","tolerance must be positive and finite"));}
    let delete=p.get("delete").and_then(Value::as_bool).unwrap_or(false);
    let keyp=|q:Vec3|->(i64,i64,i64){((q.x/tol).round() as i64,(q.y/tol).round() as i64,(q.z/tol).round() as i64)};
    let mut seen:std::collections::BTreeMap<Vec<(i64,i64,i64)>,cadcraft_doc::Handle>=std::collections::BTreeMap::new();
    let mut duplicates=Vec::new();
    {
        let d=s.doc()?;
        for h in &handles {
            let Some(e)=d.entity(*h) else{continue;}; let EntityKind::Face3d(f)=&e.kind else{continue;};
            let tri=(f.corners[3]-f.corners[2]).len()<=tol;
            let mut pts=if tri{vec![keyp(f.corners[0]),keyp(f.corners[1]),keyp(f.corners[2])]}else{f.corners.iter().copied().map(keyp).collect()};
            pts.sort();
            if let Some(first)=seen.get(&pts){duplicates.push((*h,*first));}else{seen.insert(pts,*h);}
        }
    }
    if delete && !duplicates.is_empty(){
        let d=s.doc_mut()?; for (h,_) in &duplicates{d.remove_entity(*h);} s.touch();
    }
    Ok(json!({"faceCount":handles.len(),"duplicateCount":duplicates.len(),"duplicates":duplicates.iter().map(|(h,first)|json!({"handle":h.hex(),"sameAs":first.hex()})).collect::<Vec<_>>(),"deleted":if delete{duplicates.len()}else{0},"tolerance":tol,"representation":"faceted 3DFACE"}))
}

fn run_facetstats3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles=facet_handles(s,p,"facetstats3d")?;
    let d=s.doc()?;
    let mut triangles=0usize; let mut quads=0usize; let mut degenerate=0usize;
    let mut total_area=0.0f64; let mut edge_count=0usize; let mut min_edge=f64::INFINITY; let mut max_edge=0.0f64; let mut sum_edge=0.0f64;
    for h in &handles {
        let Some(e)=d.entity(*h) else{continue;}; let EntityKind::Face3d(f)=&e.kind else{continue;};
        let tri=(f.corners[3]-f.corners[2]).len()<=1e-12;
        let pts:Vec<Vec3>=if tri{triangles+=1;vec![f.corners[0],f.corners[1],f.corners[2]]}else{quads+=1;f.corners.to_vec()};
        let a=(pts[1]-pts[0]).cross(pts[2]-pts[0]).len()*0.5;
        let b=if pts.len()==4{(pts[2]-pts[0]).cross(pts[3]-pts[0]).len()*0.5}else{0.0};
        let area=a+b; total_area+=area; if area<=1e-16{degenerate+=1;}
        for i in 0..pts.len(){let len=(pts[(i+1)%pts.len()]-pts[i]).len(); edge_count+=1; sum_edge+=len; min_edge=min_edge.min(len); max_edge=max_edge.max(len);}
    }
    if edge_count==0{min_edge=0.0;}
    Ok(json!({"faces":handles.len(),"triangles":triangles,"quads":quads,"degenerate":degenerate,"totalArea":total_area,"edgeSamples":edge_count,"minEdge":min_edge,"maxEdge":max_edge,"meanEdge":if edge_count>0{sum_edge/(edge_count as f64)}else{0.0},"readOnly":true,"representation":"faceted 3DFACE statistics"}))
}

fn run_massprops3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles:Vec<cadcraft_doc::Handle>=if let Some(raw)=p.get("handles").and_then(Value::as_array){
        raw.iter().filter_map(|v|v.as_str()).filter_map(cadcraft_doc::Handle::parse_hex).collect()
    }else{s.selection()};
    if handles.is_empty(){return Err(bad("massprops3d","select one or more 3DFACE entities"));}
    let d=s.doc()?;
    let mut faces=0usize; let mut area=0.0; let mut signed_volume=0.0;
    let mut lo:Option<Vec3>=None; let mut hi:Option<Vec3>=None;
    let mut add_tri=|a:Vec3,b:Vec3,c:Vec3|{
        area += (b-a).cross(c-a).len()*0.5;
        signed_volume += a.dot(b.cross(c))/6.0;
        for q in [a,b,c]{
            lo=Some(lo.map_or(q,|v|Vec3::new(v.x.min(q.x),v.y.min(q.y),v.z.min(q.z))));
            hi=Some(hi.map_or(q,|v|Vec3::new(v.x.max(q.x),v.y.max(q.y),v.z.max(q.z))));
        }
    };
    for h in handles {
        let Some(e)=d.entity(h) else{continue;}; let EntityKind::Face3d(f)=&e.kind else{continue;};
        faces+=1; add_tri(f.corners[0],f.corners[1],f.corners[2]);
        if (f.corners[3]-f.corners[2]).len()>1e-12 { add_tri(f.corners[0],f.corners[2],f.corners[3]); }
    }
    if faces==0{return Err(bad("massprops3d","selection contains no 3DFACE entities"));}
    let lo=lo.unwrap_or(Vec3::ZERO); let hi=hi.unwrap_or(Vec3::ZERO);
    Ok(json!({"faceCount":faces,"surfaceArea":area,"signedVolume":signed_volume,"volume":signed_volume.abs(),"bounds":{"min":[lo.x,lo.y,lo.z],"max":[hi.x,hi.y,hi.z]},"closedShellRequiredForVolume":true,"representation":"faceted 3DFACE"}))
}

fn run_facetcheck3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles:Vec<cadcraft_doc::Handle>=if let Some(raw)=p.get("handles").and_then(Value::as_array){
        raw.iter().filter_map(|v|v.as_str()).filter_map(cadcraft_doc::Handle::parse_hex).collect()
    }else{s.selection()};
    if handles.is_empty(){return Err(bad("facetcheck3d","select one or more 3DFACE entities"));}
    let tol=p.get("tolerance").and_then(Value::as_f64).unwrap_or(1e-8);
    if !tol.is_finite()||tol<=0.0{return Err(bad("facetcheck3d","tolerance must be positive and finite"));}
    let key=|q:Vec3|->(i64,i64,i64){((q.x/tol).round() as i64,(q.y/tol).round() as i64,(q.z/tol).round() as i64)};
    let mut edges:std::collections::BTreeMap<((i64,i64,i64),(i64,i64,i64)),usize>=std::collections::BTreeMap::new();
    let mut faces=0usize; let mut degenerate=0usize;
    let d=s.doc()?;
    for h in handles {
        let Some(e)=d.entity(h) else{continue;}; let EntityKind::Face3d(f)=&e.kind else{continue;}; faces+=1;
        let tri=(f.corners[3]-f.corners[2]).len()<=tol;
        let pts=if tri{vec![f.corners[0],f.corners[1],f.corners[2]]}else{f.corners.to_vec()};
        if (pts[1]-pts[0]).cross(pts[2]-pts[0]).len()<=tol*tol {degenerate+=1;}
        for i in 0..pts.len(){ let a=key(pts[i]); let b=key(pts[(i+1)%pts.len()]); let e=if a<=b{(a,b)}else{(b,a)}; *edges.entry(e).or_insert(0)+=1; }
    }
    if faces==0{return Err(bad("facetcheck3d","selection contains no 3DFACE entities"));}
    let boundary=edges.values().filter(|&&n|n==1).count();
    let manifold=edges.values().filter(|&&n|n==2).count();
    let nonmanifold=edges.values().filter(|&&n|n>2).count();
    Ok(json!({"faceCount":faces,"uniqueEdges":edges.len(),"boundaryEdges":boundary,"manifoldEdges":manifold,"nonManifoldEdges":nonmanifold,"degenerateFaces":degenerate,"closedShellCandidate":boundary==0&&nonmanifold==0&&degenerate==0,"tolerance":tol,"representation":"faceted 3DFACE"}))
}

fn run_pointcloud_style(s: &mut Session, p: &Value) -> Result<Value> {
    if let Some(size) = p.get("size").and_then(Value::as_f64) {
        if !size.is_finite() || !(0.1..=1000.0).contains(&size) {
            return Err(bad("pointcloud.style", "size must be between 0.1 and 1000"));
        }
        s.doc_mut()?.header.set_f64("POINTCLOUDSIZE", size);
    }
    if let Some(mode) = p.get("mode").and_then(Value::as_i64) {
        if !(0..=99).contains(&mode) { return Err(bad("pointcloud.style", "mode must be 0..99")); }
        s.doc_mut()?.header.set_i64("POINTCLOUDMODE", mode);
    }
    if !p.is_null() { s.touch(); }
    let d=s.doc()?;
    Ok(json!({"size":d.header.f64("POINTCLOUDSIZE",1.5),"mode":d.header.i64("POINTCLOUDMODE",0),"persisted":true}))
}


fn run_pointcloud_zstats(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.zstats","name is required"));}
    let bins=p.get("bins").and_then(Value::as_u64).unwrap_or(10).clamp(2,100) as usize;
    let visible_only=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false);
    let d=s.doc()?;
    let mut zs=Vec::new();
    for e in d.model.iter() {
        if !pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name)){continue;}
        if visible_only && !e.common.visible {continue;}
        if let EntityKind::Point(pt)=&e.kind {zs.push(pt.p.z);}
    }
    if zs.is_empty(){return Err(bad("pointcloud.zstats","no matching points"));}
    let min=zs.iter().copied().fold(f64::INFINITY,f64::min);
    let max=zs.iter().copied().fold(f64::NEG_INFINITY,f64::max);
    let mean=zs.iter().sum::<f64>()/zs.len() as f64;
    let mut hist=vec![0usize;bins];
    if (max-min).abs()<=1e-12 {hist[0]=zs.len();} else {
        for z in &zs {let mut i=(((z-min)/(max-min))*bins as f64).floor() as usize; if i>=bins{i=bins-1;} hist[i]+=1;}
    }
    let step=if (max-min).abs()<=1e-12{0.0}else{(max-min)/bins as f64};
    Ok(json!({"name":name,"count":zs.len(),"minZ":min,"maxZ":max,"meanZ":mean,"bins":bins,"binSize":step,"histogram":hist,"visibleOnly":visible_only}))
}

fn run_pointcloud_exportply(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.exportply","name is required"));}
    let visible_only=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false);
    let max_points=p.get("maxPoints").and_then(Value::as_u64).unwrap_or(200_000).clamp(1,200_000) as usize;
    let d=s.doc()?; let mut pts=Vec::new();
    for e in d.model.iter() {
        if !pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name)){continue;}
        if visible_only && !e.common.visible {continue;}
        if let EntityKind::Point(pt)=&e.kind {pts.push(pt.p); if pts.len()>max_points{return Err(bad("pointcloud.exportply","point count exceeds maxPoints"));}}
    }
    if pts.is_empty(){return Err(bad("pointcloud.exportply","no matching points"));}
    let count=pts.len();
    let mut text=format!("ply\nformat ascii 1.0\ncomment TBT360 point cloud {}\nelement vertex {}\nproperty double x\nproperty double y\nproperty double z\nend_header\n",name,count);
    for q in pts {use std::fmt::Write as _; let _=writeln!(&mut text,"{:.12} {:.12} {:.12}",q.x,q.y,q.z);}
    Ok(json!({"name":name,"count":count,"format":"ascii ply 1.0","text":text,"visibleOnly":visible_only}))
}

fn run_facettriangulate3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles=facet_handles(s,p,"facettriangulate3d")?;
    let mut quads=Vec::new();
    { let d=s.doc()?; for h in handles { if let Some(e)=d.entity(h) { if let EntityKind::Face3d(f)=&e.kind { if (f.corners[3]-f.corners[2]).len()>1e-12 {quads.push((h,f.clone(),e.common.clone()));} } } } }
    if quads.is_empty(){return Ok(json!({"triangulated":0,"created":0,"reason":"selection contains only triangle 3DFACE entities"}));}
    let mut created=Vec::new();
    for (h,f,common) in quads.iter() {
        let [a,b,c,d]=f.corners; let old=f.hidden_edges;
        s.doc_mut()?.modify_entity(*h,|e|if let EntityKind::Face3d(face)=&mut e.kind {face.corners=[a,b,c,c]; face.hidden_edges=(old&0b0011)|0b1000;})?;
        let nh=s.add_entity(EntityKind::Face3d(Face3d{corners:[a,c,d,d],hidden_edges:0b0001|((old&0b0100)>>1)|(old&0b1000)}))?;
        s.doc_mut()?.modify_entity(nh,|e|e.common=common.clone())?; created.push(nh.hex());
    }
    s.touch(); Ok(json!({"triangulated":quads.len(),"created":created.len(),"createdHandles":created,"representation":"native triangle 3DFACE"}))
}

fn run_facetweld3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles=facet_handles(s,p,"facetweld3d")?;
    let tol=p.get("tolerance").and_then(Value::as_f64).unwrap_or(1e-6);
    if !tol.is_finite()||tol<=0.0{return Err(bad("facetweld3d","tolerance must be positive and finite"));}
    let key=|q:Vec3|->(i64,i64,i64){((q.x/tol).round() as i64,(q.y/tol).round() as i64,(q.z/tol).round() as i64)};
    let mut clusters:std::collections::BTreeMap<(i64,i64,i64),(Vec3,usize)>=std::collections::BTreeMap::new();
    {let d=s.doc()?; for h in &handles {if let Some(e)=d.entity(*h){if let EntityKind::Face3d(f)=&e.kind {for q in f.corners {let entry=clusters.entry(key(q)).or_insert((Vec3::ZERO,0));entry.0=entry.0+q;entry.1+=1;}}}}}
    let centers:std::collections::BTreeMap<_,_>=clusters.iter().map(|(k,(sum,n))|(*k,*sum/(*n as f64))).collect();
    let mut moved=0usize;
    {let d=s.doc_mut()?; for h in &handles {d.modify_entity(*h,|e|if let EntityKind::Face3d(f)=&mut e.kind {for q in &mut f.corners {if let Some(c)=centers.get(&key(*q)){if (*q-*c).len()>1e-12{*q=*c;moved+=1;}}}})?;}}
    if moved>0{s.touch();}
    Ok(json!({"faces":handles.len(),"clusters":centers.len(),"movedVertices":moved,"tolerance":tol}))
}


fn run_pointcloud_dedup(s: &mut Session, p: &Value) -> Result<Value> {
    let name = p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty() { return Err(bad("pointcloud.dedup", "name is required")); }
    let tol = p.get("tolerance").and_then(Value::as_f64).unwrap_or(1e-8);
    if !tol.is_finite() || tol <= 0.0 { return Err(bad("pointcloud.dedup", "tolerance must be positive and finite")); }
    let delete = p.get("delete").and_then(Value::as_bool).unwrap_or(false);
    let key = |q: Vec3| -> (i64, i64, i64) {
        ((q.x / tol).round() as i64, (q.y / tol).round() as i64, (q.z / tol).round() as i64)
    };
    let points = pointcloud_points(s, name, "pointcloud.dedup")?;
    let mut seen = std::collections::BTreeMap::<(i64, i64, i64), cadcraft_doc::Handle>::new();
    let mut duplicates = Vec::new();
    for (h, q) in &points {
        if let Some(first) = seen.get(&key(*q)) { duplicates.push((*h, *first)); } else { seen.insert(key(*q), *h); }
    }
    if delete && !duplicates.is_empty() {
        let d = s.doc_mut()?;
        for (h, _) in &duplicates { d.remove_entity(*h); }
        s.touch();
    }
    Ok(json!({
        "name": name,
        "count": points.len(),
        "duplicateCount": duplicates.len(),
        "deleted": if delete { duplicates.len() } else { 0 },
        "tolerance": tol,
        "duplicates": duplicates.iter().take(1000).map(|(h, first)| json!({"handle":h.hex(),"sameAs":first.hex()})).collect::<Vec<_>>(),
        "truncated": duplicates.len() > 1000,
        "xyzPreservedForRepresentatives": true
    }))
}

fn run_pointcloud_density(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.density","name is required"));}
    let visible_only=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false);
    let d=s.doc()?; let mut pts=Vec::new();
    for e in d.model.iter() {
        if !pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name)){continue;}
        if visible_only && !e.common.visible {continue;}
        if let EntityKind::Point(pt)=&e.kind {pts.push(pt.p);}
    }
    if pts.is_empty(){return Err(bad("pointcloud.density","no matching points"));}
    let mut lo=pts[0]; let mut hi=pts[0]; let mut sum=Vec3::ZERO;
    for q in &pts {lo.x=lo.x.min(q.x);lo.y=lo.y.min(q.y);lo.z=lo.z.min(q.z);hi.x=hi.x.max(q.x);hi.y=hi.y.max(q.y);hi.z=hi.z.max(q.z);sum=sum+*q;}
    let dx=(hi.x-lo.x).abs(); let dy=(hi.y-lo.y).abs(); let dz=(hi.z-lo.z).abs();
    let volume=dx*dy*dz; let area_xy=dx*dy; let n=pts.len() as f64;
    Ok(json!({"name":name,"count":pts.len(),"visibleOnly":visible_only,"centroid":[(sum/n).x,(sum/n).y,(sum/n).z],"min":[lo.x,lo.y,lo.z],"max":[hi.x,hi.y,hi.z],"boundsVolume":volume,"xyArea":area_xy,"pointsPerVolume":if volume>1e-18{Some(n/volume)}else{None},"pointsPerXYArea":if area_xy>1e-18{Some(n/area_xy)}else{None},"readOnly":true}))
}


fn pointcloud_plane_solution(s: &Session, p: &Value, command: &str) -> Result<(String, Vec<Vec3>, Vec3, Vec3, Vec3, Vec3, f64, f64, f64, f64, f64, f64)> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad(command,"name is required"));}
    let visible_only=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false);
    let d=s.doc()?; let mut pts=Vec::new();
    for e in d.model.iter() {
        if !pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name)){continue;}
        if visible_only && !e.common.visible {continue;}
        if let EntityKind::Point(pt)=&e.kind {pts.push(pt.p);}
    }
    if pts.len()<3{return Err(bad(command,"at least three matching points are required"));}
    let mut c=Vec3::ZERO; for q in &pts {c=c+*q;} c=c/(pts.len() as f64);
    let mut a=[[0.0f64;3];3];
    for q in &pts {let w=*q-c; let x=[w.x,w.y,w.z]; for i in 0..3 {for j in 0..3 {a[i][j]+=x[i]*x[j];}}}
    let count=pts.len() as f64; for i in 0..3 {for j in 0..3 {a[i][j]/=count;}}
    let mut eig=[[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]];
    for _ in 0..24 {
        let mut p0=0usize; let mut q0=1usize; let mut largest=a[0][1].abs();
        for i in 0..3 {for j in (i+1)..3 {if a[i][j].abs()>largest {largest=a[i][j].abs();p0=i;q0=j;}}}
        if largest<1e-14 {break;}
        let phi=0.5*(2.0*a[p0][q0]).atan2(a[q0][q0]-a[p0][p0]); let cs=phi.cos(); let sn=phi.sin();
        let app=cs*cs*a[p0][p0]-2.0*sn*cs*a[p0][q0]+sn*sn*a[q0][q0];
        let aqq=sn*sn*a[p0][p0]+2.0*sn*cs*a[p0][q0]+cs*cs*a[q0][q0];
        a[p0][p0]=app; a[q0][q0]=aqq; a[p0][q0]=0.0; a[q0][p0]=0.0;
        for r in 0..3 {if r!=p0 && r!=q0 {let arp=a[r][p0]; let arq=a[r][q0]; a[r][p0]=cs*arp-sn*arq; a[p0][r]=a[r][p0]; a[r][q0]=sn*arp+cs*arq; a[q0][r]=a[r][q0];}}
        for r in 0..3 {let vrp=eig[r][p0]; let vrq=eig[r][q0]; eig[r][p0]=cs*vrp-sn*vrq; eig[r][q0]=sn*vrp+cs*vrq;}
    }
    let mut k=0usize; if a[1][1]<a[k][k]{k=1;} if a[2][2]<a[k][k]{k=2;}
    let normal=Vec3::new(eig[0][k],eig[1][k],eig[2][k]).normalized();
    if normal.len()<=1e-12{return Err(bad(command,"best-fit plane is numerically undefined"));}
    let reference=if normal.z.abs()<0.9 {Vec3::Z} else {Vec3::X};
    let u=reference.cross(normal).normalized(); let v=normal.cross(u).normalized();
    let mut min_u=f64::INFINITY; let mut max_u=f64::NEG_INFINITY; let mut min_v=f64::INFINITY; let mut max_v=f64::NEG_INFINITY;
    let mut sum_sq=0.0_f64; let mut max_dev=0.0_f64;
    for q in &pts {let rel=*q-c; let pu=rel.dot(u); let pv=rel.dot(v); let dev=rel.dot(normal).abs(); min_u=min_u.min(pu);max_u=max_u.max(pu);min_v=min_v.min(pv);max_v=max_v.max(pv);sum_sq+=dev*dev;max_dev=max_dev.max(dev);}
    let rms=(sum_sq/(pts.len() as f64)).sqrt();
    Ok((name.to_string(),pts,c,normal,u,v,min_u,max_u,min_v,max_v,rms,max_dev))
}

fn run_pointcloud_planefit(s: &mut Session, p: &Value) -> Result<Value> {
    let (name,pts,c,n,u,v,min_u,max_u,min_v,max_v,rms,max_dev)=pointcloud_plane_solution(s,p,"pointcloud.planefit")?;
    Ok(json!({"name":name,"count":pts.len(),"visibleOnly":p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false),"centroid":[c.x,c.y,c.z],"normal":[n.x,n.y,n.z],"axisU":[u.x,u.y,u.z],"axisV":[v.x,v.y,v.z],"uRange":[min_u,max_u],"vRange":[min_v,max_v],"rmsDeviation":rms,"maxDeviation":max_dev,"readOnly":true,"method":"least-squares covariance/Jacobi eigen plane"}))
}

fn run_pointcloud_plane3d(s: &mut Session, p: &Value) -> Result<Value> {
    let (name,pts,c,n,u,v,min_u,max_u,min_v,max_v,_,_)=pointcloud_plane_solution(s,p,"pointcloud.plane3d")?;
    let margin=p.get("margin").and_then(Value::as_f64).unwrap_or(0.0); if !margin.is_finite()||margin<0.0{return Err(bad("pointcloud.plane3d","margin must be non-negative and finite"));}
    let a=c+u*(min_u-margin)+v*(min_v-margin); let b=c+u*(max_u+margin)+v*(min_v-margin); let c2=c+u*(max_u+margin)+v*(max_v+margin); let d=c+u*(min_u-margin)+v*(max_v+margin);
    let h=s.add_entity(EntityKind::Face3d(Face3d{corners:[a,b,c2,d],hidden_edges:0}))?;
    s.doc_mut()?.modify_entity(h,|e|e.common.xdata.extend([RawTag{code:1001,value:"TBT360_POINTCLOUD_FITPLANE".into()},RawTag{code:1000,value:name.clone()}]))?; s.touch();
    Ok(json!({"name":name,"handle":h.hex(),"count":pts.len(),"centroid":[c.x,c.y,c.z],"normal":[n.x,n.y,n.z],"margin":margin,"representation":"native 3DFACE best-fit plane"}))
}

fn run_pointcloud_copyvisible(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    let new_name=p.get("newName").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty()||new_name.is_empty(){return Err(bad("pointcloud.copyvisible","name and newName are required"));}
    if name.eq_ignore_ascii_case(new_name){return Err(bad("pointcloud.copyvisible","newName must differ from source name"));}
    if s.doc()?.model.iter().any(|e|pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(new_name))){return Err(bad("pointcloud.copyvisible","newName already exists"));}
    let source=pointcloud_points(s,name,"pointcloud.copyvisible")?;
    let visible:Vec<Vec3>={let d=s.doc()?;source.iter().filter_map(|(h,q)|d.entity(*h).is_some_and(|e|e.common.visible).then_some(*q)).collect()};
    if visible.is_empty(){return Err(bad("pointcloud.copyvisible","source has no visible points"));}
    let mut handles=Vec::with_capacity(visible.len());
    for q in visible {
        let h=s.add_entity(EntityKind::Point(Point{p:q, angle:0.0}))?;
        s.doc_mut()?.modify_entity(h,|e|e.common.xdata.extend([
            RawTag{code:1001,value:"TBT360_POINTCLOUD".into()},
            RawTag{code:1000,value:new_name.to_string()},
        ]))?;
        handles.push(h.hex());
    }
    s.touch();
    Ok(json!({"source":name,"newName":new_name,"created":handles.len(),"handles":handles,"sourcePreserved":true,"persistent":true}))
}



fn run_pointcloud_octreestats(s:&mut Session,p:&Value)->Result<Value>{
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim(); if name.is_empty(){return Err(bad("pointcloud.octreestats","name is required"));}
    let max_depth=p.get("maxDepth").and_then(Value::as_u64).unwrap_or(8).clamp(1,12) as usize;
    let leaf_size=p.get("leafSize").and_then(Value::as_u64).unwrap_or(128).clamp(8,4096) as usize;
    let visible_only=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false);
    let d=s.doc()?; let mut pts=pointcloud_points(s,name,"pointcloud.octreestats")?; if visible_only{pts.retain(|(h,_)|d.entity(*h).is_some_and(|e|e.common.visible));}
    let q=pts.into_iter().map(|(_,p)|p).collect::<Vec<_>>(); if q.is_empty(){return Err(bad("pointcloud.octreestats","no matching points"));}
    fn rec(pts:&[Vec3],depth:usize,max_depth:usize,leaf_size:usize,nodes:&mut usize,leaves:&mut usize,max_seen:&mut usize,occup:&mut Vec<usize>){
        *nodes+=1;*max_seen=(*max_seen).max(depth);if pts.len()<=leaf_size||depth>=max_depth{*leaves+=1;occup.push(pts.len());return;}
        let mut min=pts[0];let mut max=pts[0];for p in &pts[1..]{min=min.min(*p);max=max.max(*p);}let c=(min+max)*0.5;
        let mut buckets:[Vec<Vec3>;8]=std::array::from_fn(|_|Vec::new());for p in pts{let mut i=0usize;if p.x>=c.x{i|=1;}if p.y>=c.y{i|=2;}if p.z>=c.z{i|=4;}buckets[i].push(*p);}let non_empty=buckets.iter().filter(|b|!b.is_empty()).count();if non_empty<=1{*leaves+=1;occup.push(pts.len());return;}for b in &buckets{if !b.is_empty(){rec(b,depth+1,max_depth,leaf_size,nodes,leaves,max_seen,occup);}}
    }
    let mut nodes=0usize;let mut leaves=0usize;let mut depth=0usize;let mut occup=Vec::new();rec(&q,0,max_depth,leaf_size,&mut nodes,&mut leaves,&mut depth,&mut occup);occup.sort_unstable();let mean=q.len() as f64/leaves.max(1) as f64;let max_occ=*occup.last().unwrap_or(&0);let median=occup.get(occup.len()/2).copied().unwrap_or(0);
    Ok(json!({"name":name,"count":q.len(),"visibleOnly":visible_only,"nodeCount":nodes,"leafCount":leaves,"maxDepthReached":depth,"leafSize":leaf_size,"meanLeafOccupancy":mean,"medianLeafOccupancy":median,"maxLeafOccupancy":max_occ,"indexType":"bounded adaptive octree statistics","readOnly":true}))
}

fn run_pointcloud_gridindex(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.gridindex","name is required"));}
    let cell=p.get("cellSize").and_then(Value::as_f64).unwrap_or(f64::NAN);
    if !cell.is_finite()||cell<=0.0{return Err(bad("pointcloud.gridindex","cellSize must be positive and finite"));}
    let visible_only=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false);
    let d=s.doc()?; let mut grid=std::collections::HashMap::<(i64,i64,i64),usize>::new(); let mut count=0usize;
    for e in d.model.iter() {
        if !pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name)){continue;}
        if visible_only && !e.common.visible {continue;}
        let EntityKind::Point(pt)=&e.kind else {continue;};
        let k=((pt.p.x/cell).floor() as i64,(pt.p.y/cell).floor() as i64,(pt.p.z/cell).floor() as i64);
        *grid.entry(k).or_insert(0)+=1; count+=1;
    }
    if count==0{return Err(bad("pointcloud.gridindex","no matching points"));}
    let occupied=grid.len(); let max_occ=grid.values().copied().max().unwrap_or(0); let mean=count as f64/occupied.max(1) as f64;
    Ok(json!({"name":name,"count":count,"visibleOnly":visible_only,"cellSize":cell,"occupiedCells":occupied,"maxOccupancy":max_occ,"meanOccupancy":mean,"indexType":"uniform 3D grid summary","readOnly":true}))
}

fn pointclass_clear_and_set(e:&mut cadcraft_doc::Entity,label:&str){
    let mut out=Vec::with_capacity(e.common.xdata.len()+2); let mut skip=false;
    for t in &e.common.xdata {
        if t.code==1001 && t.value.eq_ignore_ascii_case("TBT360_POINTCLASS") { skip=true; continue; }
        if skip { if t.code==1000 {skip=false; continue;} skip=false; }
        out.push(t.clone());
    }
    out.push(RawTag{code:1001,value:"TBT360_POINTCLASS".into()});
    out.push(RawTag{code:1000,value:label.into()}); e.common.xdata=out;
}

fn run_pointcloud_classifyz(s:&mut Session,p:&Value)->Result<Value>{
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim(); if name.is_empty(){return Err(bad("pointcloud.classifyz","name is required"));}
    let mut breaks=p.get("breaks").and_then(Value::as_array).ok_or_else(||bad("pointcloud.classifyz","breaks must be an array"))?.iter().map(|v|v.as_f64().unwrap_or(f64::NAN)).collect::<Vec<_>>();
    if breaks.iter().any(|v|!v.is_finite()){return Err(bad("pointcloud.classifyz","break values must be finite"));} breaks.sort_by(|a,b|a.total_cmp(b)); breaks.dedup_by(|a,b|(*a-*b).abs()<=1e-12);
    let labels=p.get("labels").and_then(Value::as_array).map(|a|a.iter().filter_map(Value::as_str).map(str::to_string).collect::<Vec<_>>()).unwrap_or_default();
    let visible_only=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false); let points=pointcloud_points(s,name,"pointcloud.classifyz")?;
    let mut counts=vec![0usize;breaks.len()+1]; let d=s.doc_mut()?;
    for (h,q) in points { let visible=d.entity(h).is_some_and(|e|e.common.visible); if visible_only&&!visible{continue;} let idx=breaks.partition_point(|z|q.z>=*z); let label=labels.get(idx).cloned().unwrap_or_else(||format!("ZCLASS_{idx}")); d.modify_entity(h,|e|pointclass_clear_and_set(e,&label))?; counts[idx]+=1; }
    s.touch(); Ok(json!({"name":name,"breaks":breaks,"counts":counts,"labels":labels,"persistent":true,"metadata":"TBT360_POINTCLASS"}))
}

fn run_pointcloud_icptranslate(s:&mut Session,p:&Value)->Result<Value>{
    let source=p.get("source").and_then(Value::as_str).unwrap_or("").trim(); let target=p.get("target").and_then(Value::as_str).unwrap_or("").trim();
    if source.is_empty()||target.is_empty(){return Err(bad("pointcloud.icptranslate","source and target are required"));} if source.eq_ignore_ascii_case(target){return Err(bad("pointcloud.icptranslate","source and target must differ"));}
    let iterations=p.get("iterations").and_then(Value::as_u64).unwrap_or(8).clamp(1,50) as usize; let max_pairs=p.get("maxPairs").and_then(Value::as_u64).unwrap_or(4000).clamp(100,20000) as usize; let visible_only=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false); let apply=p.get("apply").and_then(Value::as_bool).unwrap_or(false);
    let d=s.doc()?; let mut a=pointcloud_points(s,source,"pointcloud.icptranslate")?; let mut b=pointcloud_points(s,target,"pointcloud.icptranslate")?;
    if visible_only {a.retain(|(h,_)|d.entity(*h).is_some_and(|e|e.common.visible)); b.retain(|(h,_)|d.entity(*h).is_some_and(|e|e.common.visible));}
    if a.is_empty()||b.is_empty(){return Err(bad("pointcloud.icptranslate","source and target need visible points"));}
    let step=(a.len()/max_pairs).max(1); let sample=a.iter().step_by(step).take(max_pairs).map(|(_,q)|*q).collect::<Vec<_>>(); let targets=b.iter().map(|(_,q)|*q).collect::<Vec<_>>();
    let rms_for=|pts:&[Vec3],delta:Vec3|->f64{let mut ss=0.0;for q in pts{let x=*q+delta;let mut best=f64::INFINITY;for t in &targets{best=best.min((x-*t).len());}ss+=best*best;} (ss/pts.len().max(1) as f64).sqrt()};
    let before=rms_for(&sample,Vec3::ZERO); let mut delta=Vec3::ZERO;
    for _ in 0..iterations {let mut sum=Vec3::ZERO; for q in &sample{let x=*q+delta;let mut best=(f64::INFINITY,targets[0]);for t in &targets{let dd=(x-*t).len();if dd<best.0{best=(dd,*t);}} sum=sum+(best.1-x);} let step_delta=sum/(sample.len() as f64); delta=delta+step_delta; if step_delta.len()<1e-9{break;}}
    let after=rms_for(&sample,delta); if apply {let d=s.doc_mut()?;for (h,_) in &a{d.modify_entity(*h,|e|if let EntityKind::Point(pt)=&mut e.kind{pt.p=pt.p+delta;})?;}s.touch();}
    Ok(json!({"source":source,"target":target,"pairs":sample.len(),"iterations":iterations,"translation":[delta.x,delta.y,delta.z],"rmsBefore":before,"rmsAfter":after,"improved":after<=before,"applied":apply,"method":"bounded translation-only ICP nearest-neighbor"}))
}


fn cloud_points_visible(s:&Session,name:&str,visible_only:bool,command:&str)->Result<Vec<(cadcraft_doc::Handle,Vec3)>>{
    let d=s.doc()?; let mut pts=pointcloud_points(s,name,command)?;
    if visible_only{pts.retain(|(h,_)|d.entity(*h).is_some_and(|e|e.common.visible));}
    if pts.len()<3{return Err(bad(command,"at least three matching points are required"));}
    Ok(pts)
}

fn quat_rotate(q:[f64;4],v:Vec3)->Vec3{
    let [w,x,y,z]=q;
    let m00=1.0-2.0*(y*y+z*z); let m01=2.0*(x*y-z*w); let m02=2.0*(x*z+y*w);
    let m10=2.0*(x*y+z*w); let m11=1.0-2.0*(x*x+z*z); let m12=2.0*(y*z-x*w);
    let m20=2.0*(x*z-y*w); let m21=2.0*(y*z+x*w); let m22=1.0-2.0*(x*x+y*y);
    Vec3::new(m00*v.x+m01*v.y+m02*v.z,m10*v.x+m11*v.y+m12*v.z,m20*v.x+m21*v.y+m22*v.z)
}

fn horn_fit(src:&[Vec3],dst:&[Vec3])->([f64;4],Vec3){
    let n=src.len().min(dst.len()).max(1) as f64;
    let mut cs=Vec3::ZERO; let mut ct=Vec3::ZERO;
    for q in src{cs=cs+*q;} for q in dst{ct=ct+*q;} cs=cs/n; ct=ct/n;
    let mut sxx=0.0;let mut sxy=0.0;let mut sxz=0.0;let mut syx=0.0;let mut syy=0.0;let mut syz=0.0;let mut szx=0.0;let mut szy=0.0;let mut szz=0.0;
    for (a,b) in src.iter().zip(dst){let u=*a-cs;let v=*b-ct;sxx+=u.x*v.x;sxy+=u.x*v.y;sxz+=u.x*v.z;syx+=u.y*v.x;syy+=u.y*v.y;syz+=u.y*v.z;szx+=u.z*v.x;szy+=u.z*v.y;szz+=u.z*v.z;}
    let tr=sxx+syy+szz;
    let nmat=[[tr,syz-szy,szx-sxz,sxy-syx],[syz-szy,sxx-syy-szz,sxy+syx,szx+sxz],[szx-sxz,sxy+syx,-sxx+syy-szz,syz+szy],[sxy-syx,szx+sxz,syz+szy,-sxx-syy+szz]];
    let mut q=[1.0,0.0,0.0,0.0];
    for _ in 0..32{let mut nq=[0.0;4];for i in 0..4{for j in 0..4{nq[i]+=nmat[i][j]*q[j];}}let len=(nq.iter().map(|v|v*v).sum::<f64>()).sqrt();if len<=1e-18{break;}for i in 0..4{q[i]=nq[i]/len;}}
    let t=ct-quat_rotate(q,cs); (q,t)
}

fn run_pointcloud_icprigid(s:&mut Session,p:&Value)->Result<Value>{
    let source=p.get("source").and_then(Value::as_str).unwrap_or("").trim(); let target=p.get("target").and_then(Value::as_str).unwrap_or("").trim();
    if source.is_empty()||target.is_empty()||source.eq_ignore_ascii_case(target){return Err(bad("pointcloud.icprigid","source and target must be different non-empty names"));}
    let iterations=p.get("iterations").and_then(Value::as_u64).unwrap_or(10).clamp(1,30) as usize; let max_pairs=p.get("maxPairs").and_then(Value::as_u64).unwrap_or(3000).clamp(100,12000) as usize; let visible=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false); let apply=p.get("apply").and_then(Value::as_bool).unwrap_or(false);
    let a=cloud_points_visible(s,source,visible,"pointcloud.icprigid")?; let b=cloud_points_visible(s,target,visible,"pointcloud.icprigid")?; let target_pts=b.iter().map(|(_,q)|*q).collect::<Vec<_>>();
    let step=(a.len()/max_pairs).max(1); let sample=a.iter().step_by(step).take(max_pairs).map(|(_,q)|*q).collect::<Vec<_>>(); let mut qtot=[1.0,0.0,0.0,0.0]; let mut ttot=Vec3::ZERO;
    let rms=|qq:[f64;4],tt:Vec3|{let mut ss=0.0;for p0 in &sample{let x=quat_rotate(qq,*p0)+tt;let mut best=f64::INFINITY;for y in &target_pts{best=best.min((x-*y).len());}ss+=best*best;}(ss/sample.len().max(1) as f64).sqrt()}; let before=rms(qtot,ttot);
    let mut used=0usize;
    for _ in 0..iterations{let mut src_fit=Vec::with_capacity(sample.len());let mut dst_fit=Vec::with_capacity(sample.len());for p0 in &sample{let x=quat_rotate(qtot,*p0)+ttot;let mut best=(f64::INFINITY,target_pts[0]);for y in &target_pts{let d=(x-*y).len();if d<best.0{best=(d,*y);}}src_fit.push(x);dst_fit.push(best.1);}let (dq,dt)=horn_fit(&src_fit,&dst_fit);used+=1;ttot=quat_rotate(dq,ttot)+dt;let [aw,ax,ay,az]=dq;let [bw,bx,by,bz]=qtot;qtot=[aw*bw-ax*bx-ay*by-az*bz,aw*bx+ax*bw+ay*bz-az*by,aw*by-ax*bz+ay*bw+az*bx,aw*bz+ax*by-ay*bx+az*bw];let norm=(qtot.iter().map(|v|v*v).sum::<f64>()).sqrt();for i in 0..4{qtot[i]/=norm.max(1e-18);}if dt.len()<1e-9 && (1.0-dq[0].abs())<1e-10{break;}}
    let after=rms(qtot,ttot);
    if apply{let d=s.doc_mut()?;for (h,_) in &a{d.modify_entity(*h,|e|if let EntityKind::Point(pt)=&mut e.kind{pt.p=quat_rotate(qtot,pt.p)+ttot;})?;}s.touch();}
    Ok(json!({"source":source,"target":target,"pairs":sample.len(),"iterationsUsed":used,"quaternionWxyz":qtot,"translation":[ttot.x,ttot.y,ttot.z],"rmsBefore":before,"rmsAfter":after,"improved":after<=before,"applied":apply,"method":"bounded 6DoF rigid ICP nearest-neighbor + Horn quaternion fit"}))
}

fn run_pointcloud_octreebuild(s:&mut Session,p:&Value)->Result<Value>{
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim(); if name.is_empty(){return Err(bad("pointcloud.octreebuild","name is required"));}
    let depth=p.get("maxDepth").and_then(Value::as_u64).unwrap_or(8).clamp(1,12) as usize; let leaf=p.get("leafSize").and_then(Value::as_u64).unwrap_or(128).clamp(8,4096) as usize; let visible=p.get("visibleOnly").and_then(Value::as_bool).unwrap_or(false); let pts=cloud_points_visible(s,name,visible,"pointcloud.octreebuild")?;
    let mut lo=pts[0].1; let mut hi=pts[0].1; for (_,q) in &pts{lo.x=lo.x.min(q.x);lo.y=lo.y.min(q.y);lo.z=lo.z.min(q.z);hi.x=hi.x.max(q.x);hi.y=hi.y.max(q.y);hi.z=hi.z.max(q.z);} let size=(hi.x-lo.x).max((hi.y-lo.y).max(hi.z-lo.z)).max(1e-9);
    let cells=(1usize<<depth.min(20)) as f64; let mut counts=std::collections::BTreeMap::<(i64,i64,i64),usize>::new(); let d=s.doc_mut()?;
    for (h,q) in pts{let ix=(((q.x-lo.x)/size)*cells).floor().clamp(0.0,cells-1.0) as i64;let iy=(((q.y-lo.y)/size)*cells).floor().clamp(0.0,cells-1.0) as i64;let iz=(((q.z-lo.z)/size)*cells).floor().clamp(0.0,cells-1.0) as i64;let key=(ix,iy,iz);*counts.entry(key).or_insert(0)+=1;let code=format!("{ix}:{iy}:{iz}:{depth}");d.modify_entity(h,|e|{e.common.xdata.retain(|t|t.code!=1001||!t.value.eq_ignore_ascii_case("TBT360_OCTREE"));e.common.xdata.extend([RawTag{code:1001,value:"TBT360_OCTREE".into()},RawTag{code:1000,value:code.clone()}]);})?;}
    d.header.set_str("TBT360_OCTREE_NAME",name);d.header.set_i64("TBT360_OCTREE_DEPTH",depth as i64);d.header.set_i64("TBT360_OCTREE_LEAF",leaf as i64);s.touch(); Ok(json!({"name":name,"maxDepth":depth,"leafSize":leaf,"occupiedLeafCodes":counts.len(),"persistent":true,"metadata":"TBT360_OCTREE","streamingFoundation":true}))
}

fn run_surfacepatch(s:&mut Session,p:&Value)->Result<Value>{
    let arr=p.get("corners").and_then(Value::as_array).ok_or_else(||bad("surfacepatch","corners must contain four 3D points"))?;if arr.len()!=4{return Err(bad("surfacepatch","exactly four corners are required"));}let mut c=[Vec3::ZERO;4];for (i,v) in arr.iter().enumerate(){c[i]=vec3_value(v).ok_or_else(||bad("surfacepatch","invalid corner"))?;}
    let nu=p.get("u").and_then(Value::as_u64).unwrap_or(8).clamp(1,64) as usize;let nv=p.get("v").and_then(Value::as_u64).unwrap_or(8).clamp(1,64) as usize;let bilinear=|u:f64,v:f64|c[0]*((1.0-u)*(1.0-v))+c[1]*(u*(1.0-v))+c[2]*(u*v)+c[3]*((1.0-u)*v);let mut hs=Vec::new();for j in 0..nv{for i in 0..nu{let u0=i as f64/nu as f64;let u1=(i+1) as f64/nu as f64;let v0=j as f64/nv as f64;let v1=(j+1) as f64/nv as f64;let h=s.add_entity(EntityKind::Face3d(Face3d{corners:[bilinear(u0,v0),bilinear(u1,v0),bilinear(u1,v1),bilinear(u0,v1)],hidden_edges:0}))?;hs.push(h.hex());}}s.touch();Ok(json!({"u":nu,"v":nv,"faces":hs.len(),"handles":hs,"representation":"faceted bilinear patch; not NURBS"}))
}

fn run_solidaudit3d(s:&mut Session,p:&Value)->Result<Value>{
    let hs=facet_handles(s,p,"solidaudit3d")?;let tol=p.get("tolerance").and_then(Value::as_f64).unwrap_or(1e-8);let key=|q:Vec3|((q.x/tol).round() as i64,(q.y/tol).round() as i64,(q.z/tol).round() as i64);let mut edges=std::collections::BTreeMap::<((i64,i64,i64),(i64,i64,i64)),usize>::new();let mut volume=0.0;let d=s.doc()?;for h in &hs{if let Some(e)=d.entity(*h){if let EntityKind::Face3d(f)=&e.kind{let tri=(f.corners[3]-f.corners[2]).len()<=tol;let pts=if tri{vec![f.corners[0],f.corners[1],f.corners[2]]}else{f.corners.to_vec()};for i in 0..pts.len(){let a=key(pts[i]);let b=key(pts[(i+1)%pts.len()]);let k=if a<=b{(a,b)}else{(b,a)};*edges.entry(k).or_insert(0)+=1;}volume+=f.corners[0].dot(f.corners[1].cross(f.corners[2]))/6.0;if !tri{volume+=f.corners[0].dot(f.corners[2].cross(f.corners[3]))/6.0;}}}}let boundary=edges.values().filter(|&&n|n==1).count();let nonmanifold=edges.values().filter(|&&n|n>2).count();Ok(json!({"faces":hs.len(),"edges":edges.len(),"boundaryEdges":boundary,"nonManifoldEdges":nonmanifold,"watertight":boundary==0&&nonmanifold==0,"signedVolumeEstimate":volume,"kernel":"faceted shell audit, not ACIS/B-Rep","readOnly":true}))
}

fn run_hiddenline3d(s:&mut Session,p:&Value)->Result<Value>{
    let hs=facet_handles(s,p,"hiddenline3d")?;let view=vec3_param_or(p,"view",Vec3::new(1.0,-1.0,1.0))?.normalized();let origin=vec3_param_or(p,"origin",Vec3::ZERO)?;let scale=p.get("scale").and_then(Value::as_f64).unwrap_or(1.0).max(1e-9);let up=if view.z.abs()<0.95{Vec3::Z}else{Vec3::Y};let ux=up.cross(view).normalized();let uy=view.cross(ux).normalized();let tol=1e-8;let key=|q:Vec3|((q.x/tol).round() as i64,(q.y/tol).round() as i64,(q.z/tol).round() as i64);let mut edges=std::collections::BTreeMap::<((i64,i64,i64),(i64,i64,i64)),(Vec3,Vec3,Vec<f64>)>::new();{let d=s.doc()?;for h in &hs{let Some(e)=d.entity(*h) else{continue;};let EntityKind::Face3d(f)=&e.kind else{continue;};let tri=(f.corners[3]-f.corners[2]).len()<=tol;let pts=if tri{vec![f.corners[0],f.corners[1],f.corners[2]]}else{f.corners.to_vec()};let n=(pts[1]-pts[0]).cross(pts[2]-pts[0]).normalized();let side=n.dot(view);for i in 0..pts.len(){let a=pts[i];let b=pts[(i+1)%pts.len()];let ka=key(a);let kb=key(b);let k=if ka<=kb{(ka,kb)}else{(kb,ka)};edges.entry(k).and_modify(|e|e.2.push(side)).or_insert((a,b,vec![side]));}}}let mut created=Vec::new();for (_, (a,b,sides)) in edges{let draw=sides.len()==1 || (sides.iter().any(|v|*v>=0.0)&&sides.iter().any(|v|*v<0.0));if !draw{continue;}let pa=origin+Vec3::new((a.dot(ux))*scale,(a.dot(uy))*scale,0.0);let pb=origin+Vec3::new((b.dot(ux))*scale,(b.dot(uy))*scale,0.0);let h=s.add_entity(EntityKind::Line(Line{a:pa,b:pb}))?;created.push(h.hex());}if !created.is_empty(){s.touch();}Ok(json!({"sourceFaces":hs.len(),"createdLines":created.len(),"handles":created,"projection":"orthographic silhouette/boundary extraction","occlusion":"approximate; no z-buffer occlusion"}))
}

fn run_pbrmaterial(s:&mut Session,p:&Value)->Result<Value>{
    let mut hs=if let Some(a)=p.get("handles").and_then(Value::as_array){a.iter().filter_map(Value::as_str).filter_map(cadcraft_doc::Handle::parse_hex).collect::<Vec<_>>()}else{s.selection().to_vec()};if hs.is_empty(){return Err(bad("pbrmaterial","select entities or provide handles"));}let metallic=p.get("metallic").and_then(Value::as_f64).unwrap_or(0.0).clamp(0.0,1.0);let rough=p.get("roughness").and_then(Value::as_f64).unwrap_or(0.5).clamp(0.0,1.0);let opacity=p.get("opacity").and_then(Value::as_f64).unwrap_or(1.0).clamp(0.0,1.0);let color=p.get("baseColor").and_then(Value::as_str).unwrap_or("#cccccc");let tex=p.get("texture").and_then(Value::as_str).unwrap_or("");let normal=p.get("normalMap").and_then(Value::as_str).unwrap_or("");let payload=format!("baseColor={color};metallic={metallic};roughness={rough};opacity={opacity};texture={tex};normalMap={normal}");let d=s.doc_mut()?;let mut applied=0;for h in hs.drain(..){if d.entity(h).is_none(){continue;}d.modify_entity(h,|e|{e.common.xdata.retain(|t|t.code!=1001||!t.value.eq_ignore_ascii_case("TBT360_PBR"));e.common.xdata.extend([RawTag{code:1001,value:"TBT360_PBR".into()},RawTag{code:1000,value:payload.clone()}]);})?;applied+=1;}if applied>0{s.touch();}Ok(json!({"applied":applied,"metallic":metallic,"roughness":rough,"opacity":opacity,"baseColor":color,"texture":tex,"normalMap":normal,"rendererSupport":"metadata foundation; current renderer may not shade all PBR channels"}))
}

fn run_facetcrease3d(s:&mut Session,p:&Value)->Result<Value>{
    let handles=facet_handles(s,p,"facetcrease3d")?; let angle=p.get("angle").and_then(Value::as_f64).unwrap_or(30.0); if !angle.is_finite()||!(0.0..=180.0).contains(&angle){return Err(bad("facetcrease3d","angle must be between 0 and 180 degrees"));}
    let tol=p.get("tolerance").and_then(Value::as_f64).unwrap_or(1e-8); if !tol.is_finite()||tol<=0.0{return Err(bad("facetcrease3d","tolerance must be positive"));}
    let key=|q:Vec3|->(i64,i64,i64){((q.x/tol).round() as i64,(q.y/tol).round() as i64,(q.z/tol).round() as i64)};
    let mut map=std::collections::BTreeMap::<((i64,i64,i64),(i64,i64,i64)),Vec<(Vec3,Vec3,Vec3)>>::new();
    {let d=s.doc()?;for h in &handles{let Some(e)=d.entity(*h) else{continue;};let EntityKind::Face3d(f)=&e.kind else{continue;};let tri=(f.corners[3]-f.corners[2]).len()<=tol;let pts=if tri{vec![f.corners[0],f.corners[1],f.corners[2]]}else{f.corners.to_vec()};let n=(pts[1]-pts[0]).cross(pts[2]-pts[0]).normalized();if n.len()<=1e-12{continue;}for i in 0..pts.len(){let a=pts[i];let b=pts[(i+1)%pts.len()];let ka=key(a);let kb=key(b);let k=if ka<=kb{(ka,kb)}else{(kb,ka)};map.entry(k).or_default().push((a,b,n));}}}
    let mut created=Vec::new(); for entries in map.values(){if entries.len()!=2{continue;}let n0=entries[0].2;let n1=entries[1].2;let deg=n0.dot(n1).clamp(-1.0,1.0).acos().to_degrees();if deg+1e-9>=angle{let h=s.add_entity(EntityKind::Line(Line{a:entries[0].0,b:entries[0].1}))?;tag_3d_annotation(s,h,"facet-crease")?;created.push(h.hex());}}
    if !created.is_empty(){s.touch();} Ok(json!({"faceCount":handles.len(),"angle":angle,"created":created.len(),"createdHandles":created,"representation":"native 3D LINE crease edges"}))
}

fn run_facetboundary3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles=facet_handles(s,p,"facetboundary3d")?;
    let tol=p.get("tolerance").and_then(Value::as_f64).unwrap_or(1e-8);
    if !tol.is_finite()||tol<=0.0{return Err(bad("facetboundary3d","tolerance must be positive and finite"));}
    let key=|q:Vec3|->(i64,i64,i64){((q.x/tol).round() as i64,(q.y/tol).round() as i64,(q.z/tol).round() as i64)};
    let mut edges:std::collections::BTreeMap<((i64,i64,i64),(i64,i64,i64)),(usize,Vec3,Vec3)>=std::collections::BTreeMap::new();
    {let d=s.doc()?;for h in &handles{if let Some(e)=d.entity(*h){if let EntityKind::Face3d(f)=&e.kind{
        let tri=(f.corners[3]-f.corners[2]).len()<=tol; let pts=if tri{vec![f.corners[0],f.corners[1],f.corners[2]]}else{f.corners.to_vec()};
        for i in 0..pts.len(){let a=pts[i];let b=pts[(i+1)%pts.len()];let ka=key(a);let kb=key(b);let ek=if ka<=kb{(ka,kb)}else{(kb,ka)};let ent=edges.entry(ek).or_insert((0,a,b));ent.0+=1;}
    }}}}
    let boundary:Vec<_>=edges.values().filter(|(n,_,_)|*n==1).map(|(_,a,b)|(*a,*b)).collect();
    let mut created=Vec::with_capacity(boundary.len());
    for (a,b) in boundary {let h=s.add_entity(EntityKind::Line(Line{a,b}))?;tag_3d_annotation(s,h,"facet-boundary")?;created.push(h.hex());}
    if !created.is_empty(){s.touch();}
    Ok(json!({"faceCount":handles.len(),"boundaryEdges":created.len(),"createdHandles":created,"tolerance":tol,"representation":"native 3D LINE boundary extraction"}))
}

fn run_facetsubdivide3d(s: &mut Session, p: &Value) -> Result<Value> {
    let handles = facet_handles(s, p, "facetsubdivide3d")?;
    let mut source = Vec::new();
    {
        let d = s.doc()?;
        for h in &handles {
            if let Some(e) = d.entity(*h) {
                if let EntityKind::Face3d(f) = &e.kind {
                    source.push((*h, f.clone(), e.common.clone()));
                }
            }
        }
    }
    if source.is_empty() {
        return Err(bad("facetsubdivide3d", "selection contains no 3DFACE entities"));
    }
    let mut created = Vec::new();
    let mut triangles = 0usize;
    let mut quads = 0usize;
    for (h, f, common) in source {
        let [a, b, c, d] = f.corners;
        let triangle = (d - c).len() <= 1e-12;
        if triangle {
            triangles += 1;
            let ab = (a + b) / 2.0;
            let bc = (b + c) / 2.0;
            let ca = (c + a) / 2.0;
            s.doc_mut()?.modify_entity(h, |e| {
                if let EntityKind::Face3d(face) = &mut e.kind {
                    face.corners = [a, ab, ca, ca];
                    face.hidden_edges = 0;
                }
            })?;
            for corners in [[ab, b, bc, bc], [ca, bc, c, c], [ab, bc, ca, ca]] {
                let nh = s.add_entity(EntityKind::Face3d(Face3d { corners, hidden_edges: 0 }))?;
                s.doc_mut()?.modify_entity(nh, |e| e.common = common.clone())?;
                created.push(nh.hex());
            }
        } else {
            quads += 1;
            let ab = (a + b) / 2.0;
            let bc = (b + c) / 2.0;
            let cd = (c + d) / 2.0;
            let da = (d + a) / 2.0;
            let center = (a + b + c + d) / 4.0;
            s.doc_mut()?.modify_entity(h, |e| {
                if let EntityKind::Face3d(face) = &mut e.kind {
                    face.corners = [a, ab, center, da];
                    face.hidden_edges = 0;
                }
            })?;
            for corners in [
                [ab, b, bc, center],
                [center, bc, c, cd],
                [da, center, cd, d],
            ] {
                let nh = s.add_entity(EntityKind::Face3d(Face3d { corners, hidden_edges: 0 }))?;
                s.doc_mut()?.modify_entity(nh, |e| e.common = common.clone())?;
                created.push(nh.hex());
            }
        }
    }
    s.touch();
    Ok(json!({
        "sourceFaces": handles.len(),
        "triangles": triangles,
        "quads": quads,
        "created": created.len(),
        "createdHandles": created,
        "resultFaces": handles.len() + triangles * 3 + quads * 3,
        "representation": "native subdivided 3DFACE",
        "undoable": true
    }))
}

fn run_sectioncutstyle(s: &mut Session, p: &Value) -> Result<Value> {
    let current=s.doc()?.header.i64("SECTIONCUTLINES",1)!=0;
    let lines=p.get("lines").and_then(Value::as_bool).unwrap_or(!current);
    s.doc_mut()?.header.set_i64("SECTIONCUTLINES",if lines{1}else{0});
    s.touch();
    Ok(json!({"lines":lines}))
}

fn run_pointcloud_delete(s: &mut Session, p: &Value) -> Result<Value> {
    let name=p.get("name").and_then(Value::as_str).unwrap_or("").trim();
    if name.is_empty(){return Err(bad("pointcloud.delete","name is required"));}
    let handles:Vec<_>=s.doc()?.model.iter().filter(|e|pointcloud_group(e).is_some_and(|n|n.eq_ignore_ascii_case(name))).map(|e|e.handle).collect();
    if handles.is_empty(){return Err(bad("pointcloud.delete","no such point cloud"));}
    let d=s.doc_mut()?;
    for h in &handles { d.remove_entity(*h); }
    s.touch();
    Ok(json!({"name":name,"deleted":handles.len()}))
}

fn run_pointcloud(s: &mut Session, p: &Value) -> Result<Value> {
    let raw = p.get("points").and_then(Value::as_array).ok_or_else(|| bad("pointcloud", "points must be [[x,y,z],...]"))?;
    let max_points = p.get("maxPoints").and_then(Value::as_u64).unwrap_or(200_000).clamp(1, 200_000) as usize;
    if raw.is_empty() { return Err(bad("pointcloud", "point cloud must contain at least one point")); }
    if raw.len() > max_points { return Err(bad("pointcloud", "point count exceeds maxPoints")); }
    let name = p.get("name").and_then(Value::as_str).unwrap_or("PointCloud").trim();
    if name.is_empty() || name.len() > 128 { return Err(bad("pointcloud", "name must contain 1..128 characters")); }
    let mut handles = Vec::with_capacity(raw.len());
    for item in raw {
        let a=item.as_array().ok_or_else(|| bad("pointcloud", "each point must be [x,y,z]"))?;
        if a.len()!=3 { return Err(bad("pointcloud", "each point must be [x,y,z]")); }
        let vals: Vec<f64>=a.iter().map(|v| v.as_f64().unwrap_or(f64::NAN)).collect();
        if vals.iter().any(|v| !v.is_finite()) { return Err(bad("pointcloud", "point coordinates must be finite")); }
        let h=s.add_entity(EntityKind::Point(Point { p: Vec3::new(vals[0],vals[1],vals[2]), angle: 0.0 }))?;
        s.doc_mut()?.modify_entity(h, |e| e.common.xdata.extend([
            RawTag { code: 1001, value: "TBT360_POINTCLOUD".into() },
            RawTag { code: 1000, value: name.to_string() },
        ]))?;
        handles.push(h.hex());
    }
    Ok(json!({"name":name,"count":handles.len(),"handles":handles,"representation":"native POINT entities","acis":false}))
}


fn added(h: cadcraft_doc::Handle) -> Result<Value> {
    Ok(json!({ "handle": h.hex() }))
}

/// Re-export for tests.
pub(crate) fn _unused(_: Polyline) {}

#[cfg(test)]
mod r18_tests {
    use super::*;
    use crate::Session;
    #[test]
    fn pointcloud_stats_and_isolation_state() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"A","points":[[0,0,0],[2,2,2]]})).unwrap();
        run_pointcloud(&mut s,&json!({"name":"B","points":[[9,0,0]]})).unwrap();
        assert_eq!(run_pointcloud_list(&mut s,&Value::Null).unwrap()["groups"].as_array().unwrap().len(),2);
        run_pointcloud_isolate(&mut s,&json!({"name":"A"})).unwrap();
        assert!(run_pointcloud_list(&mut s,&Value::Null).unwrap()["groups"].as_array().unwrap().iter().any(|g|g["name"]=="B" && g["visible"]==false));
        run_pointcloud_showall(&mut s,&Value::Null).unwrap();
        assert!(run_pointcloud_list(&mut s,&Value::Null).unwrap()["groups"].as_array().unwrap().iter().all(|g|g["visible"]==true));
    }
    #[test]
    fn angle_and_radius_3d_annotation_geometry() {
        let mut s=Session::new();
        let a=run_dim3d_angle(&mut s,&json!({"vertex":[0,0,0],"a":[1,0,0],"b":[0,1,0]})).unwrap();
        assert!((a["angle"].as_f64().unwrap()-90.0).abs()<1e-9);
        let r=run_dim3d_radius(&mut s,&json!({"center":[0,0,0],"point":[3,4,0]})).unwrap();
        assert!((r["radius"].as_f64().unwrap()-5.0).abs()<1e-9);
    }
    #[test]
    fn pointcloud_sampling_is_non_destructive() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"LOD","points":[[0,0,0],[1,0,0],[2,0,0],[3,0,0],[4,0,0]]})).unwrap();
        let v=run_pointcloud_sample(&mut s,&json!({"name":"LOD","keepEvery":2})).unwrap();
        assert_eq!(v["count"],5); assert_eq!(v["visible"],3);
        run_pointcloud_showall(&mut s,&Value::Null).unwrap();
        let groups=run_pointcloud_list(&mut s,&Value::Null).unwrap();
        assert_eq!(groups["groups"][0]["visibleCount"],5);
    }
    #[test]
    fn pointcloud_display_style_persists_and_validates() {
        let mut s=Session::new();
        let v=run_pointcloud_style(&mut s,&json!({"size":3.5,"mode":2})).unwrap();
        assert!((v["size"].as_f64().unwrap()-3.5).abs()<1e-12);
        assert_eq!(v["mode"],2);
        assert_eq!(s.doc().unwrap().header.i64("POINTCLOUDMODE",0),2);
        assert!(run_pointcloud_style(&mut s,&json!({"size":0.0})).is_err());
        assert!(run_pointcloud_style(&mut s,&json!({"mode":100})).is_err());
    }
    #[test]
    fn section_cut_lines_state_persists_in_header() {
        let mut s=Session::new();
        run_sectioncutstyle(&mut s,&json!({"lines":false})).unwrap();
        assert_eq!(s.doc().unwrap().header.i64("SECTIONCUTLINES",1),0);
        run_sectioncutstyle(&mut s,&json!({"lines":true})).unwrap();
        assert_eq!(s.doc().unwrap().header.i64("SECTIONCUTLINES",0),1);
    }
    #[test]
    fn pointcloud_crop_is_non_destructive_and_invertible() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"CROP","points":[[0,0,0],[5,5,5],[10,10,10]]})).unwrap();
        let v=run_pointcloud_crop(&mut s,&json!({"name":"CROP","min":[-1,-1,-1],"max":[6,6,6]})).unwrap();
        assert_eq!(v["count"],3); assert_eq!(v["visible"],2);
        let inv=run_pointcloud_crop(&mut s,&json!({"name":"CROP","min":[-1,-1,-1],"max":[6,6,6],"invert":true})).unwrap();
        assert_eq!(inv["visible"],1);
        assert_eq!(run_pointcloud_list(&mut s,&Value::Null).unwrap()["groups"][0]["count"],3);
    }
    #[test]
    fn facet_massprops_reports_area_volume_and_bounds() {
        let mut s=Session::new();
        let h=run_3dface(&mut s,&json!({"points":[[0,0,0],[1,0,0],[0,1,0]]})).unwrap()["handle"].as_str().unwrap().to_string();
        let v=run_massprops3d(&mut s,&json!({"handles":[h]})).unwrap();
        assert_eq!(v["faceCount"],1);
        assert!((v["surfaceArea"].as_f64().unwrap()-0.5).abs()<1e-12);
        assert_eq!(v["volume"],0.0);
        assert_eq!(v["bounds"]["max"],json!([1.0,1.0,0.0]));
    }
    #[test]
    fn facet_shell_check_finds_boundary_edges() {
        let mut s=Session::new();
        let h=run_3dface(&mut s,&json!({"points":[[0,0,0],[1,0,0],[0,1,0]]})).unwrap()["handle"].as_str().unwrap().to_string();
        let v=run_facetcheck3d(&mut s,&json!({"handles":[h]})).unwrap();
        assert_eq!(v["faceCount"],1); assert_eq!(v["boundaryEdges"],3); assert_eq!(v["closedShellCandidate"],false);
    }
    #[test]
    fn pointcloud_translate_and_scale_preserve_group_points() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"MOVE","points":[[0,0,0],[2,0,0]]})).unwrap();
        run_pointcloud_translate(&mut s,&json!({"name":"MOVE","delta":[1,2,3]})).unwrap();
        let moved=run_pointcloud_list(&mut s,&Value::Null).unwrap();
        assert_eq!(moved["groups"][0]["centroid"],json!([2.0,2.0,3.0]));
        run_pointcloud_scale(&mut s,&json!({"name":"MOVE","factor":2.0})).unwrap();
        let scaled=run_pointcloud_list(&mut s,&Value::Null).unwrap();
        assert_eq!(scaled["groups"][0]["min"],json!([0.0,2.0,3.0]));
        assert_eq!(scaled["groups"][0]["max"],json!([4.0,2.0,3.0]));
    }
    #[test]
    fn pointcloud_rotate_and_recenter_transform_xyz() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"ROT","points":[[1,0,0],[3,0,0]]})).unwrap();
        run_pointcloud_rotate(&mut s,&json!({"name":"ROT","angle":90.0,"origin":[0,0,0]})).unwrap();
        let rotated=run_pointcloud_list(&mut s,&Value::Null).unwrap();
        assert!((rotated["groups"][0]["centroid"][0].as_f64().unwrap()).abs()<1e-9);
        assert!((rotated["groups"][0]["centroid"][1].as_f64().unwrap()-2.0).abs()<1e-9);
        run_pointcloud_recenter(&mut s,&json!({"name":"ROT"})).unwrap();
        let centered=run_pointcloud_list(&mut s,&Value::Null).unwrap();
        for i in 0..3 { assert!(centered["groups"][0]["centroid"][i].as_f64().unwrap().abs()<1e-9); }
    }
    #[test]
    fn pointcloud_rename_and_merge_preserve_xyz() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"A","points":[[1,2,3]]})).unwrap();
        run_pointcloud(&mut s,&json!({"name":"B","points":[[4,5,6]]})).unwrap();
        run_pointcloud_rename(&mut s,&json!({"name":"A","newName":"C"})).unwrap();
        run_pointcloud_merge(&mut s,&json!({"source":"C","target":"B"})).unwrap();
        let listed=run_pointcloud_list(&mut s,&Value::Null).unwrap();
        assert_eq!(listed["groups"].as_array().unwrap().len(),1);
        assert_eq!(listed["groups"][0]["name"],"B"); assert_eq!(listed["groups"][0]["count"],2);
        assert_eq!(listed["groups"][0]["min"],json!([1.0,2.0,3.0])); assert_eq!(listed["groups"][0]["max"],json!([4.0,5.0,6.0]));
    }
    #[test]
    fn pointcloud_bounds_color_and_export_preserve_xyz() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"QA","points":[[0,0,0],[3,4,12]]})).unwrap();
        let b=run_pointcloud_bounds(&mut s,&json!({"name":"QA"})).unwrap();
        assert_eq!(b["count"],2); assert_eq!(b["min"],json!([0.0,0.0,0.0])); assert_eq!(b["max"],json!([3.0,4.0,12.0])); assert!((b["diagonal"].as_f64().unwrap()-13.0).abs()<1e-12);
        let c=run_pointcloud_color(&mut s,&json!({"name":"QA","color":1})).unwrap(); assert_eq!(c["changed"],2);
        let x=run_pointcloud_exportxyz(&mut s,&json!({"name":"QA","delimiter":"space"})).unwrap(); assert_eq!(x["count"],2); assert!(x["text"].as_str().unwrap().contains("3.000000000000 4.000000000000 12.000000000000"));
        let after=run_pointcloud_bounds(&mut s,&json!({"name":"QA"})).unwrap(); assert_eq!(after["max"],json!([3.0,4.0,12.0]));
    }
    #[test]
    fn facet_orient_outward_flips_inward_faces() {
        let mut s=Session::new();
        let a=run_3dface(&mut s,&json!({"points":[[0,0,0],[0,1,0],[1,0,0]]})).unwrap()["handle"].as_str().unwrap().to_string();
        let b=run_3dface(&mut s,&json!({"points":[[0,0,1],[1,0,1],[0,1,1]]})).unwrap()["handle"].as_str().unwrap().to_string();
        let v=run_facetorientout3d(&mut s,&json!({"handles":[a,b]})).unwrap(); assert_eq!(v["faceCount"],2); assert!(v["flipped"].as_u64().unwrap()>=1);
    }
    #[test]
    fn facet_normal_report_returns_unit_normal_and_area() {
        let mut s=Session::new();
        let h=run_3dface(&mut s,&json!({"points":[[0,0,0],[2,0,0],[0,2,0]]})).unwrap()["handle"].as_str().unwrap().to_string();
        let v=run_facetnormals3d(&mut s,&json!({"handles":[h]})).unwrap();
        assert_eq!(v["faceCount"],1); assert_eq!(v["faces"][0]["normal"],json!([0.0,0.0,1.0])); assert_eq!(v["faces"][0]["area"],2.0);
    }
    #[test]
    fn facet_flip_reverses_face_normal() {
        let mut s=Session::new();
        let h=run_3dface(&mut s,&json!({"points":[[0,0,0],[1,0,0],[0,1,0]]})).unwrap()["handle"].as_str().unwrap().to_string();
        let handle=cadcraft_doc::Handle::parse_hex(&h).unwrap();
        let before=match &s.doc().unwrap().entity(handle).unwrap().kind { EntityKind::Face3d(f)=>(f.corners[1]-f.corners[0]).cross(f.corners[2]-f.corners[0]), _=>Vec3::ZERO };
        run_facetflip3d(&mut s,&json!({"handles":[h]})).unwrap();
        let after=match &s.doc().unwrap().entity(handle).unwrap().kind { EntityKind::Face3d(f)=>(f.corners[1]-f.corners[0]).cross(f.corners[2]-f.corners[0]), _=>Vec3::ZERO };
        assert!(before.dot(after)<0.0);
    }


    #[test]
    fn pointcloud_nearest_returns_ranked_read_only_results() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"NEAR","points":[[0,0,0],[5,0,0],[2,0,0]]})).unwrap();
        let v=run_pointcloud_nearest(&mut s,&json!({"name":"NEAR","at":[1,0,0],"maxResults":2})).unwrap();
        assert_eq!(v["results"].as_array().unwrap().len(),2);
        assert_eq!(v["results"][0]["point"],json!([0.0,0.0,0.0]));
        assert_eq!(v["results"][1]["point"],json!([2.0,0.0,0.0]));
    }
    #[test]
    fn facet_transform_moves_scales_and_rotates_native_faces() {
        let mut s=Session::new();
        let h=run_3dface(&mut s,&json!({"points":[[0,0,0],[1,0,0],[0,1,0]]})).unwrap()["handle"].as_str().unwrap().to_string();
        run_facettransform3d(&mut s,&json!({"mode":"move","handles":[h.clone()],"delta":[1,2,3]})).unwrap();
        let handle=cadcraft_doc::Handle::parse_hex(&h).unwrap();
        let first=match &s.doc().unwrap().entity(handle).unwrap().kind{EntityKind::Face3d(f)=>f.corners[0],_=>Vec3::ZERO};
        assert_eq!([first.x,first.y,first.z],[1.0,2.0,3.0]);
        run_facettransform3d(&mut s,&json!({"mode":"scale","handles":[h.clone()],"factor":2.0,"origin":[1,2,3]})).unwrap();
        run_facettransform3d(&mut s,&json!({"mode":"rotate","handles":[h],"axis":[0,0,1],"angle":90.0,"origin":[1,2,3]})).unwrap();
    }

    #[test]
    fn pointcloud_voxel_and_clipplane_are_non_destructive_filters() {
        let mut s=Session::new(); s.new_doc();
        run_pointcloud(&mut s,&json!({"name":"FILTERS","points":[[0,0,0],[0.1,0.1,0.1],[2,0,0],[-2,0,0]]})).unwrap();
        let v=run_pointcloud_voxel(&mut s,&json!({"name":"FILTERS","size":1.0})).unwrap();
        assert_eq!(v["visible"],3);
        let c=run_pointcloud_clipplane(&mut s,&json!({"name":"FILTERS","origin":[0,0,0],"normal":[1,0,0],"keepPositive":true})).unwrap();
        assert_eq!(c["visible"],3);
        assert_eq!(pointcloud_points(&s,"FILTERS","test").unwrap().len(),4);
    }

    #[test]
    fn facet_dedup_reports_then_deletes_duplicate_faces() {
        let mut s=Session::new(); s.new_doc();
        let a=run_3dface(&mut s,&json!({"points":[[0,0,0],[1,0,0],[0,1,0]]})).unwrap();
        let b=run_3dface(&mut s,&json!({"points":[[1,0,0],[0,1,0],[0,0,0]]})).unwrap();
        let ha=a["handle"].as_str().unwrap(); let hb=b["handle"].as_str().unwrap();
        let r=run_facetdedup3d(&mut s,&json!({"handles":[ha,hb]})).unwrap(); assert_eq!(r["duplicateCount"],1);
        let d=run_facetdedup3d(&mut s,&json!({"handles":[ha,hb],"delete":true})).unwrap(); assert_eq!(d["deleted"],1);
    }

    #[test]
    fn pointcloud_radius_filters_and_sorts_without_mutation() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"RADIUS","points":[[0,0,0],[1,0,0],[3,0,0],[0,2,0]]})).unwrap();
        let before=run_pointcloud_bounds(&mut s,&json!({"name":"RADIUS"})).unwrap();
        let v=run_pointcloud_radius(&mut s,&json!({"name":"RADIUS","at":[0,0,0],"radius":2.1,"maxResults":10})).unwrap();
        assert_eq!(v["matches"],3);
        assert_eq!(v["results"][0]["point"],json!([0.0,0.0,0.0]));
        assert_eq!(v["results"][1]["point"],json!([1.0,0.0,0.0]));
        let after=run_pointcloud_bounds(&mut s,&json!({"name":"RADIUS"})).unwrap();
        assert_eq!(before["min"],after["min"]); assert_eq!(before["max"],after["max"]);
    }
    #[test]
    fn facet_mirror_reflects_vertices_and_preserves_normal_orientation() {
        let mut s=Session::new();
        let h=run_3dface(&mut s,&json!({"points":[[1,0,0],[2,0,0],[1,1,0]]})).unwrap()["handle"].as_str().unwrap().to_string();
        let handle=cadcraft_doc::Handle::parse_hex(&h).unwrap();
        let before=match &s.doc().unwrap().entity(handle).unwrap().kind{EntityKind::Face3d(f)=>(f.corners[1]-f.corners[0]).cross(f.corners[2]-f.corners[0]),_=>Vec3::ZERO};
        let v=run_facetmirror3d(&mut s,&json!({"handles":[h],"origin":[0,0,0],"normal":[1,0,0]})).unwrap();
        assert_eq!(v["changed"],1);
        let f=match &s.doc().unwrap().entity(handle).unwrap().kind{EntityKind::Face3d(f)=>f,_=>panic!("expected face")};
        assert_eq!(f.corners[0].x,-1.0);
        let after=(f.corners[1]-f.corners[0]).cross(f.corners[2]-f.corners[0]);
        assert!(before.dot(after)>0.0);
    }

    #[test]
    fn pointcloud_sliceplane_is_non_destructive_and_bounded() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"SLICE","points":[[0,0,-2],[0,0,-0.25],[0,0,0.25],[0,0,2]]})).unwrap();
        let v=run_pointcloud_sliceplane(&mut s,&json!({"name":"SLICE","origin":[0,0,0],"normal":[0,0,1],"halfThickness":0.5})).unwrap();
        assert_eq!(v["visible"],2);
        assert_eq!(pointcloud_points(&s,"SLICE","test").unwrap().len(),4);
    }
    #[test]
    fn facet_offset_moves_face_along_its_normal() {
        let mut s=Session::new();
        let h=run_3dface(&mut s,&json!({"points":[[0,0,0],[1,0,0],[0,1,0]]})).unwrap()["handle"].as_str().unwrap().to_string();
        let v=run_facetoffset3d(&mut s,&json!({"handles":[h.clone()],"distance":2.0})).unwrap();
        assert_eq!(v["changed"],1);
        let handle=cadcraft_doc::Handle::parse_hex(&h).unwrap();
        let z=match &s.doc().unwrap().entity(handle).unwrap().kind{EntityKind::Face3d(f)=>f.corners[0].z,_=>0.0};
        assert!((z-2.0).abs()<1e-12);
    }

#[cfg(test)]
mod r25_tests {
    use super::*;
    use crate::Session;
    #[test]
    fn pointcloud_rotate3d_and_filterz_are_real_geometry_workflows() {
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"R25","points":[[1,0,0],[0,0,2]]})).unwrap();
        let r=run_pointcloud_rotate3d(&mut s,&json!({"name":"R25","axis":[0,1,0],"angle":90.0,"origin":[0,0,0]})).unwrap();
        assert_eq!(r["count"],2);
        let f=run_pointcloud_filterz(&mut s,&json!({"name":"R25","minZ":-2.0,"maxZ":0.5})).unwrap();
        assert_eq!(f["count"],2);
    }
    #[test]
    fn facet_cleanup_reports_without_deleting_by_default() {
        let mut s=Session::new();
        let h=run_3dface(&mut s,&json!({"points":[[0,0,0],[0,0,0],[0,0,0]]})).unwrap()["handle"].as_str().unwrap().to_string();
        let v=run_facetcleanup3d(&mut s,&json!({"handles":[h.clone()]})).unwrap();
        assert_eq!(v["degenerate"],1);
        let handle=cadcraft_doc::Handle::parse_hex(&h).unwrap();
        assert!(s.doc().unwrap().entity(handle).is_some());
    }
}

}

#[cfg(test)]
mod r30_tests {
    use super::*; use crate::Session;
    #[test] fn triangulate_and_weld_facets(){
        let mut s=Session::new();
        let h=s.add_entity(EntityKind::Face3d(Face3d{corners:[Vec3::new(0.0,0.0,0.0),Vec3::new(1.0,0.0,0.0),Vec3::new(1.0,1.0,0.0),Vec3::new(0.0,1.0,0.0)],hidden_edges:0})).unwrap();
        let v=run_facettriangulate3d(&mut s,&json!({"handles":[h.hex()]})).unwrap(); assert_eq!(v["triangulated"],1);
        let all:Vec<String>=s.doc().unwrap().model.iter().filter(|e|matches!(e.kind,EntityKind::Face3d(_))).map(|e|e.handle.hex()).collect();
        let w=run_facetweld3d(&mut s,&json!({"handles":all,"tolerance":1e-5})).unwrap(); assert_eq!(w["faces"],2);
    }
    #[test] fn r44_pointcloud_plane_fit_and_geometry_are_real(){
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"PLANE","points":[[0,0,1],[2,0,1],[0,2,1],[2,2,1],[1,1,1.01]]})).unwrap();
        let fit=run_pointcloud_planefit(&mut s,&json!({"name":"PLANE"})).unwrap();
        assert_eq!(fit["count"],5); assert!(fit["rmsDeviation"].as_f64().unwrap()<0.01);
        let g=run_pointcloud_plane3d(&mut s,&json!({"name":"PLANE","margin":0.25})).unwrap();
        let h=Handle::parse_hex(g["handle"].as_str().unwrap()).unwrap(); assert!(matches!(&s.doc().unwrap().entity(h).unwrap().kind,EntityKind::Face3d(_)));
    }

    #[test] fn r31_pointcloud_density_copyvisible_and_facet_boundary_workflows(){
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"SRC","points":[[0,0,0],[1,0,0],[0,1,1]]})).unwrap();
        let points=pointcloud_points(&s,"SRC","test").unwrap();
        s.doc_mut().unwrap().modify_entity(points[1].0,|e|e.common.visible=false).unwrap();
        let den=run_pointcloud_density(&mut s,&json!({"name":"SRC","visibleOnly":true})).unwrap(); assert_eq!(den["count"],2);
        let cp=run_pointcloud_copyvisible(&mut s,&json!({"name":"SRC","newName":"VISIBLE"})).unwrap(); assert_eq!(cp["created"],2);
        assert_eq!(pointcloud_points(&s,"VISIBLE","test").unwrap().len(),2);
        let a=s.add_entity(EntityKind::Face3d(Face3d{corners:[Vec3::new(0.,0.,0.),Vec3::new(1.,0.,0.),Vec3::new(1.,1.,0.),Vec3::new(0.,1.,0.)],hidden_edges:0})).unwrap();
        let b=s.add_entity(EntityKind::Face3d(Face3d{corners:[Vec3::new(1.,0.,0.),Vec3::new(2.,0.,0.),Vec3::new(2.,1.,0.),Vec3::new(1.,1.,0.)],hidden_edges:0})).unwrap();
        let out=run_facetboundary3d(&mut s,&json!({"handles":[a.hex(),b.hex()]})).unwrap(); assert_eq!(out["boundaryEdges"],6);
    }

    #[test] fn pointcloud_zstats_and_ply_are_read_only(){
        let mut s=Session::new(); run_pointcloud(&mut s,&json!({"name":"P","points":[[0,0,1],[0,0,2],[0,0,3]]})).unwrap();
        let z=run_pointcloud_zstats(&mut s,&json!({"name":"P","bins":2})).unwrap(); assert_eq!(z["count"],3); assert_eq!(z["minZ"],1.0); assert_eq!(z["maxZ"],3.0);
        let p=run_pointcloud_exportply(&mut s,&json!({"name":"P"})).unwrap(); assert!(p["text"].as_str().unwrap().starts_with("ply\nformat ascii 1.0"));
    }
    #[test] fn r32_subdivide_preserves_native_face_geometry(){
        let mut s=Session::new();
        let h=s.add_entity(EntityKind::Face3d(Face3d{corners:[Vec3::new(0.,0.,0.),Vec3::new(2.,0.,0.),Vec3::new(2.,2.,0.),Vec3::new(0.,2.,0.)],hidden_edges:0})).unwrap();
        let out=run_facetsubdivide3d(&mut s,&json!({"handles":[h.hex()]})).unwrap();
        assert_eq!(out["sourceFaces"],1); assert_eq!(out["created"],3);
        let faces=s.doc().unwrap().model.iter().filter(|e|matches!(e.kind,EntityKind::Face3d(_))).count(); assert_eq!(faces,4);
    }
    #[test] fn r32_pointcloud_dedup_reports_then_removes_duplicates(){
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"DUP","points":[[0,0,0],[0.00001,0,0],[1,0,0]]})).unwrap();
        let report=run_pointcloud_dedup(&mut s,&json!({"name":"DUP","tolerance":0.001})).unwrap(); assert_eq!(report["duplicateCount"],1); assert_eq!(report["deleted"],0);
        let clean=run_pointcloud_dedup(&mut s,&json!({"name":"DUP","tolerance":0.001,"delete":true})).unwrap(); assert_eq!(clean["deleted"],1);
        assert_eq!(pointcloud_points(&s,"DUP","test").unwrap().len(),2);
    }

    #[test] fn r45_grid_classification_and_icp_translation_are_real(){
        let mut s=Session::new();
        run_pointcloud(&mut s,&json!({"name":"A","points":[[0,0,0],[1,0,1],[2,0,2]]})).unwrap();
        run_pointcloud(&mut s,&json!({"name":"B","points":[[5,0,0],[6,0,1],[7,0,2]]})).unwrap();
        let g=run_pointcloud_gridindex(&mut s,&json!({"name":"A","cellSize":1.0})).unwrap(); assert_eq!(g["count"],3);
        let c=run_pointcloud_classifyz(&mut s,&json!({"name":"A","breaks":[0.5,1.5]})).unwrap(); assert_eq!(c["counts"],json!([1,1,1]));
        let r=run_pointcloud_icptranslate(&mut s,&json!({"source":"A","target":"B","apply":true,"iterations":10})).unwrap(); assert!(r["rmsAfter"].as_f64().unwrap()<=r["rmsBefore"].as_f64().unwrap());
        let bounds=run_pointcloud_bounds(&mut s,&json!({"name":"A"})).unwrap(); assert!(bounds["min"][0].as_f64().unwrap()>4.9);
    }

    #[test] fn r45_facet_crease_extracts_feature_edge(){
        let mut s=Session::new();
        let a=s.add_entity(EntityKind::Face3d(Face3d{corners:[Vec3::new(0.,0.,0.),Vec3::new(1.,0.,0.),Vec3::new(1.,1.,0.),Vec3::new(0.,1.,0.)],hidden_edges:0})).unwrap();
        let b=s.add_entity(EntityKind::Face3d(Face3d{corners:[Vec3::new(1.,0.,0.),Vec3::new(1.,0.,1.),Vec3::new(1.,1.,1.),Vec3::new(1.,1.,0.)],hidden_edges:0})).unwrap();
        let r=run_facetcrease3d(&mut s,&json!({"handles":[a.hex(),b.hex()],"angle":45.0})).unwrap(); assert_eq!(r["created"],1);
    }

    #[test] fn r46_assoc_dim_refresh_tracks_source_line(){
        let mut s=Session::new(); let h=s.add_entity(EntityKind::Line(Line{a:Vec3::new(0.,0.,0.),b:Vec3::new(3.,4.,0.)})).unwrap();
        let a=run_dim3d_assoc(&mut s,&json!({"handle":h.hex()})).unwrap(); assert_eq!(a["associative"],true);
        s.doc_mut().unwrap().modify_entity(h,|e|if let EntityKind::Line(l)=&mut e.kind{l.b=Vec3::new(6.,8.,0.);}).unwrap();
        let r=run_dim3d_refresh(&mut s,&json!({})).unwrap(); assert_eq!(r["refreshed"],1);
        let linked=s.doc().unwrap().model.iter().filter(|e|dim3d_assoc_meta(e).is_some()).count(); assert!(linked>=4);
    }
    #[test] fn r46_octree_stats_are_bounded_and_read_only(){
        let mut s=Session::new();run_pointcloud(&mut s,&json!({"name":"OCT","points":[[0,0,0],[1,0,0],[0,1,0],[1,1,0],[0,0,1],[1,0,1],[0,1,1],[1,1,1]]})).unwrap();
        let v=run_pointcloud_octreestats(&mut s,&json!({"name":"OCT","leafSize":8,"maxDepth":6})).unwrap();assert_eq!(v["count"],8);assert!(v["leafCount"].as_u64().unwrap()>=1);
    }

}
