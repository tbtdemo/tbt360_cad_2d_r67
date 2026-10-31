use super::blocks::{insert, sub_ctx};
use super::dim_draw::{MAX_FILL_SOLIDS, MAX_FILL_TEXTS, block_text_box, dim_text_fill, dimension, dimension_in, solid_covers, starts_text_marks};
use super::display::{Kind, NO_LAYER, VIEWPORT_CONTENT};
use super::props::{resolve, transparency};
use super::text_entity::{mtext, place_text_entity};
use super::*;

/// Orthographic projection used only for genuinely 3D entities. 2D entities keep their exact
/// XY geometry. VIEWYAW/VIEWPITCH are degrees persisted in the drawing header by 3DORBIT.

fn shade_face(d: &Drawing, rgb: Rgb, corners: &[Vec3; 4]) -> Rgb {
    let a=corners[1]-corners[0];
    let b=corners[2]-corners[0];
    let n=a.cross(b).normalized();
    if n == Vec3::ZERO { return rgb; }
    let light=Vec3::new(d.header.f64("LIGHTDIRX",0.4), d.header.f64("LIGHTDIRY",-0.6), d.header.f64("LIGHTDIRZ",0.7)).normalized();
    let intensity=d.header.f64("LIGHTINTENSITY",1.0).clamp(0.0,10.0);
    let lambert=n.dot(light).abs();
    let factor=(0.25 + 0.75*lambert*intensity.min(1.5)).clamp(0.12,1.15);
    let c=|v:u8| -> u8 { (f64::from(v)*factor).round().clamp(0.0,255.0) as u8 };
    Rgb(c(rgb.0),c(rgb.1),c(rgb.2))
}

fn visual_style(d: &Drawing) -> String { d.header.str("VISUALSTYLE", "2DWireframe") }

fn section_marker(e: &Entity) -> bool {
    e.common.xdata.iter().any(|t| t.code == 1001 && t.value == "TBT360_SECTION")
}

fn active_section(d: &Drawing) -> Option<(Vec3, Vec3, bool)> {
    if d.header.i64("SECTIONCLIP", 0) == 0 { return None; }
    let positive = !d.header.str("SECTIONCLIPSIDE", "positive").eq_ignore_ascii_case("negative");
    for e in d.model.iter() {
        if !section_marker(e) { continue; }
        let mut ox=None; let mut oy=None; let mut oz=None;
        let mut nx=None; let mut ny=None; let mut nz=None;
        for t in &e.common.xdata {
            let val=|| t.value.parse::<f64>().ok();
            match t.code { 1010=>ox=val(),1020=>oy=val(),1030=>oz=val(),1011=>nx=val(),1021=>ny=val(),1031=>nz=val(),_=>{} }
        }
        if let (Some(ox),Some(oy),Some(oz),Some(nx),Some(ny),Some(nz))=(ox,oy,oz,nx,ny,nz) {
            let n=Vec3::new(nx,ny,nz).normalized();
            if n != Vec3::ZERO { return Some((Vec3::new(ox,oy,oz),n,positive)); }
        }
    }
    None
}

fn clipped_by_section(d: &Drawing, pts: &[Vec3]) -> bool {
    let Some((origin,normal,positive))=active_section(d) else { return false; };
    if pts.is_empty() { return false; }
    let hidden=|p:Vec3| { let side=(p-origin).dot(normal); if positive { side < 0.0 } else { side > 0.0 } };
    pts.iter().copied().all(hidden)
}

fn section_inside(p: Vec3, origin: Vec3, normal: Vec3, positive: bool) -> bool {
    let side = (p - origin).dot(normal);
    if positive { side >= -1e-10 } else { side <= 1e-10 }
}

/// Clip a 3D line segment against the active section half-space.
fn clip_section_segment(d: &Drawing, a: Vec3, b: Vec3) -> Option<(Vec3, Vec3)> {
    let Some((origin, normal, positive)) = active_section(d) else { return Some((a, b)); };
    let ia = section_inside(a, origin, normal, positive);
    let ib = section_inside(b, origin, normal, positive);
    if ia && ib { return Some((a, b)); }
    if !ia && !ib { return None; }
    let da = (a - origin).dot(normal);
    let db = (b - origin).dot(normal);
    let denom = da - db;
    if denom.abs() <= 1e-12 { return if ia { Some((a, b)) } else { None }; }
    let t = (da / denom).clamp(0.0, 1.0);
    let q = a + (b - a) * t;
    if ia { Some((a, q)) } else { Some((q, b)) }
}

/// Sutherland-Hodgman clipping of a 3D polygon against the active section plane.
fn section_cut_points(d: &Drawing, input: &[Vec3]) -> Vec<Vec3> {
    let Some((origin, normal, _positive)) = active_section(d) else { return Vec::new(); };
    if input.len() < 2 { return Vec::new(); }
    let mut hits=Vec::new();
    for i in 0..input.len() {
        let a=input[i]; let b=input[(i+1)%input.len()];
        let da=(a-origin).dot(normal); let db=(b-origin).dot(normal);
        if da.abs() <= 1e-10 { hits.push(a); }
        if (da < -1e-10 && db > 1e-10) || (da > 1e-10 && db < -1e-10) {
            let t=da/(da-db); hits.push(a+(b-a)*t);
        }
    }
    let mut unique=Vec::new();
    for p in hits { if !unique.iter().any(|q: &Vec3| (*q-p).len() <= 1e-8) { unique.push(p); } }
    unique
}

fn clip_section_polygon(d: &Drawing, input: &[Vec3]) -> Vec<Vec3> {
    let Some((origin, normal, positive)) = active_section(d) else { return input.to_vec(); };
    if input.is_empty() { return Vec::new(); }
    let mut out = Vec::with_capacity(input.len() + 2);
    let Some(&mut_prev) = input.last() else { return Vec::new(); };
    let mut prev = mut_prev;
    let mut prev_in = section_inside(prev, origin, normal, positive);
    for &cur in input {
        let cur_in = section_inside(cur, origin, normal, positive);
        if cur_in != prev_in {
            let da = (prev - origin).dot(normal);
            let db = (cur - origin).dot(normal);
            let denom = da - db;
            if denom.abs() > 1e-12 {
                let t = (da / denom).clamp(0.0, 1.0);
                out.push(prev + (cur - prev) * t);
            }
        }
        if cur_in { out.push(cur); }
        prev = cur;
        prev_in = cur_in;
    }
    out
}

fn project3(d: &Drawing, p: Vec3) -> Vec2 {
    let yaw = d.header.f64("VIEWYAW", 0.0).to_radians();
    let pitch = d.header.f64("VIEWPITCH", 0.0).to_radians();
    let (cy, sy) = (yaw.cos(), yaw.sin());
    let (cp, sp) = (pitch.cos(), pitch.sin());
    let x = cy * p.x - sy * p.y;
    let y0 = sy * p.x + cy * p.y;
    let y = cp * y0 - sp * p.z;
    let depth = sp * y0 + cp * p.z;
    if d.header.str("VIEWPROJ", "orthographic").eq_ignore_ascii_case("perspective") {
        let distance = d.header.f64("VIEWDIST", 100.0).abs().max(1e-3);
        let denom = (distance + depth).max(distance * 0.05);
        let scale = distance / denom;
        Vec2::new(x * scale, y * scale)
    } else {
        Vec2::new(x, y)
    }
}

fn annotation3d(e: &Entity) -> bool {
    e.common.xdata.iter().any(|t| t.code == 1001 && t.value == "TBT360_3DANNOTATION")
}

fn pointcloud_entity(e: &Entity) -> bool {
    e.common.xdata.iter().any(|t| t.code == 1001 && t.value == "TBT360_POINTCLOUD")
}

fn view_normal_depth(d: &Drawing, n: Vec3) -> f64 {
    let yaw = d.header.f64("VIEWYAW", 0.0).to_radians();
    let pitch = d.header.f64("VIEWPITCH", 0.0).to_radians();
    let (cy, sy) = (yaw.cos(), yaw.sin());
    let (cp, sp) = (pitch.cos(), pitch.sin());
    let y0 = sy * n.x + cy * n.y;
    sp * y0 + cp * n.z
}

/// The layer `e` is drawn on, as an index in the layer table: layer 0 inside a block is the
/// block reference's layer.
fn entity_layer(ctx: &Ctx, e: &Entity) -> u32 {
    let name = if e.common.layer == "0" { ctx.block_layer.as_deref().unwrap_or("0") } else { e.common.layer.as_str() };
    ctx.d.layers.iter().position(|l| l.name.eq_ignore_ascii_case(name)).and_then(|i| u32::try_from(i).ok()).unwrap_or(NO_LAYER)
}

pub(crate) fn entity(b: &mut Builder, ctx: &Ctx, e: &Entity) {
    // Hidden by ISOLATEOBJECTS / HIDEOBJECTS (and so in viewports and plots too). A cancelled
    // build stops here (block contents come through here too).
    if ctx.d.is_hidden_object(e.handle) || b.opts.cancelled() {
        return;
    }
    // The entity's named plot style and transparency (set by `entity_parts`), for everything it
    // draws (block contents set their own).
    let (saved, saved_transparency) = (b.named, b.transparency);
    b.named = b.styles.and_then(|s| s.named_index(ctx, &e.common));
    entity_parts(b, ctx, e);
    b.named = saved;
    b.transparency = saved_transparency;
    if b.plotting && b.list.prim_layers.len() < b.list.prims.len() {
        // What it drew that its block contents didn't claim is on its own layer.
        let layer = entity_layer(ctx, e);
        let from = b.list.prim_layers.len();
        b.list.prim_layers.resize(b.list.prims.len(), layer);
        if ctx.top == VIEWPORT_CONTENT {
            // Model space seen through a viewport is clipped into the layout's list; the layer
            // travels in the handle until then (see [`take_viewport_layers`]).
            for p in b.list.prims.iter_mut().skip(from) {
                p.handle = Handle(u64::from(layer));
            }
        }
    }
}

fn entity_parts(b: &mut Builder, ctx: &Ctx, e: &Entity) {
    // A top-level annotative object draws at the view's annotation scale: a viewport's scale for
    // model space seen through it, else the drawing's (CANNOSCALE in model space, 1:1 on paper).
    if ctx.depth == 0 && cadcraft_doc::is_annotative(e) {
        let mode = if ctx.top == VIEWPORT_CONTENT {
            cadcraft_doc::anno_draw(e, 1.0 / ctx.xf.scale_factor().max(1e-12), ctx.d.anno_all_visible())
        } else {
            ctx.d.anno_draw(e)
        };
        match mode {
            cadcraft_doc::AnnoDraw::Hidden => return,
            cadcraft_doc::AnnoDraw::Scaled(f) => {
                let mut scaled = cadcraft_doc::anno_scaled(e, f);
                scaled.common.anno_scales.clear();
                return entity_parts(b, ctx, &scaled);
            }
            cadcraft_doc::AnnoDraw::Plain => {}
        }
    }
    let (rgb, lw, lt, ltscale, visible, layer_transparency) = resolve(ctx, e, b.plotting);
    if !visible {
        return;
    }
    if b.opts.transparency {
        b.transparency = transparency(e.common.transparency, layer_transparency, ctx.block_transparency);
    }
    // A plot style's linetype replaces the object's.
    let lt = match b.styles.and_then(|s| s.linetype(rgb, b.named)) {
        Some(over) => over,
        None => lt,
    };
    let tol = b.opts.tolerance / ctx.xf.scale_factor().max(1e-12);
    let lw = if b.opts.lineweights { lw } else { 0.0 };
    // Stroke a polyline with the entity's linetype.
    let stroke = |b: &mut Builder, pts: &[Vec2]| match &lt {
        Some(lt) => {
            let min = b.opts.min_dash / ctx.xf.scale_factor().max(1e-12);
            for dash in linetype::apply(pts, lt, ltscale, min) {
                if dash.len() == 1 {
                    if let Some(p) = dash.first() {
                        b.point(ctx, rgb, *p);
                    }
                } else {
                    b.polyline(ctx, rgb, lw, &dash);
                }
            }
            // Text embedded in the linetype (shapes need SHX files and are not drawn).
            if b.opts.text {
                for sh in linetype_text::texts(ctx.d, pts, lt, ltscale, min) {
                    b.shaped(ctx, rgb, lw, &sh);
                }
            }
        }
        None => b.polyline(ctx, rgb, lw, pts),
    };
    match &e.kind {
        EntityKind::Line(l) if annotation3d(e) => {
            if clipped_by_section(ctx.d, &[l.a, l.b]) { return; }
            stroke(b, &[project3(ctx.d, l.a), project3(ctx.d, l.b)]);
        }
        EntityKind::Text(t) => {
            if !b.opts.text {
                return;
            }
            if annotation3d(e) {
                if clipped_by_section(ctx.d, &[t.insert]) { return; }
                let mut projected = t.clone();
                let p = project3(ctx.d, t.insert);
                projected.insert = Vec3::new(p.x, p.y, 0.0);
                if let Some(ap) = t.align_pt {
                    let q = project3(ctx.d, ap);
                    projected.align_pt = Some(Vec3::new(q.x, q.y, 0.0));
                }
                let (sh, _) = place_text_entity(ctx.d, &projected, &projected.value);
                b.shaped(ctx, rgb, lw, &sh);
            } else {
                let (sh, _) = place_text_entity(ctx.d, t, &t.value);
                b.shaped(ctx, rgb, lw, &sh);
            }
        }
        EntityKind::AttDef(a) => {
            let (sh, _) = place_text_entity(ctx.d, &a.text, &a.tag);
            b.shaped(ctx, rgb, lw, &sh);
        }
        EntityKind::MText(t) => {
            if !b.opts.text {
                return;
            }
            mtext(b, ctx, t, rgb, lw);
        }
        EntityKind::Insert(ins) => insert(b, ctx, e, ins, rgb),
        EntityKind::Dimension(dm) => {
            if let Some(blk) = dm.block.as_ref().and_then(|n| ctx.d.block(n))
                && ctx.depth < cadcraft_doc::MAX_BLOCK_DEPTH
            {
                let sub = Ctx { block_pstyle: b.named, block_transparency: b.transparency, ..sub_ctx(ctx, e, Mat3::IDENTITY) };
                // DIMTFILL: the block's MTEXT carries no background mask here, so a background
                // around each of the block's texts is drawn before the block's first text or first
                // line of the text's marks, except for a text that a SOLID of the block covers.
                let style = ctx.d.dim_style(&dm.style).cloned().unwrap_or_default().with_overrides(&dm.overrides);
                let g = if style.text_fill != 0 { dimension_in(ctx.d, dm) } else { DimGeometry::default() };
                let gap = style.text_gap.abs() * style.effective_scale(ctx.d.header.f64("DIMSCALE", 1.0));
                let boxes: Vec<Vec<Vec2>> = if g.text_fill.is_empty() {
                    Vec::new()
                } else {
                    let solids: Vec<&Entity> =
                        blk.entities.iter().map(|be| &**be).filter(|be| matches!(be.kind, EntityKind::Solid(_))).take(MAX_FILL_SOLIDS).collect();
                    // A text's own box (no gap): a SOLID holding all its corners is its background.
                    let covered =
                        |be: &Entity| block_text_box(ctx.d, be, 0.0).is_some_and(|t| solids.iter().any(|s| t.iter().all(|p| solid_covers(s, *p))));
                    blk.entities
                        .iter()
                        .filter(|be| matches!(be.kind, EntityKind::MText(_) | EntityKind::Text(_)))
                        .take(MAX_FILL_TEXTS)
                        .filter(|be| !covered(be))
                        .filter_map(|be| block_text_box(ctx.d, be, gap))
                        .collect()
                };
                let mut fill_pending = !boxes.is_empty();
                for be in blk.entities.iter() {
                    if !b.expand() {
                        break;
                    }
                    if fill_pending && starts_text_marks(be, &g) {
                        for corners in &boxes {
                            dim_text_fill(b, ctx, e, &style, corners);
                        }
                        fill_pending = false;
                    }
                    entity(b, &sub, be);
                }
            } else {
                dimension(b, ctx, e, dm, lw, lt.as_ref(), ltscale);
            }
        }
        EntityKind::Hatch(h) => hatch::draw(b, ctx, e, h, rgb, lw, tol),
        EntityKind::MLine(m) => mline::draw(b, ctx, e, m, rgb, lw, tol),
        EntityKind::LwPolyline(p) if p.const_width > 0.0 || p.vertices.iter().any(|v| v.start_width > 0.0 || v.end_width > 0.0) => {
            if b.opts.fill && ctx.d.header.i64("FILLMODE", 1) != 0 {
                // The GPU canvas has no MSAA: a band narrower than a pixel lights only where it
                // covers pixel centres, so whole sides vanish at some zooms (#1434). On screen the
                // centre line is drawn too, as a hairline (wide bands are drawn without lineweight),
                // so a thin band still shows as a one-pixel line. Always, not below a threshold:
                // the list serves a whole band of zooms. Plots and exports draw the band alone, and
                // a transparent band gets none (its alpha would add up along the centre).
                let hairline = !b.plotting && b.opts.min_dash > 0.0 && b.transparency == 0;
                match &lt {
                    // Dashed: each dash a filled piece of the band; dots a line across it.
                    Some(lt) => {
                        let min = b.opts.min_dash / ctx.xf.scale_factor().max(1e-12);
                        let (tris, ticks, centres) = wide::dashed(p, tol, lt, ltscale, min);
                        b.tris(ctx, rgb, &tris);
                        if hairline {
                            for c in &centres {
                                b.hairline(ctx, rgb, c);
                            }
                        }
                        for t in &ticks {
                            match t.as_slice() {
                                [q] => b.point(ctx, rgb, *q),
                                _ => b.polyline(ctx, rgb, lw, t),
                            }
                        }
                    }
                    None => {
                        b.tris(ctx, rgb, &fill::wide_polyline(p, tol));
                        if hairline {
                            let pl = Polyline { vertices: p.vertices.clone(), closed: p.closed };
                            b.hairline(ctx, rgb, &pl.tessellate(tol));
                        }
                    }
                }
            } else {
                let pl = Polyline { vertices: p.vertices.clone(), closed: p.closed };
                stroke(b, &pl.tessellate(tol));
            }
        }
        EntityKind::LwPolyline(p) => {
            let pl = Polyline { vertices: p.vertices.clone(), closed: p.closed };
            if p.plinegen || lt.is_none() {
                stroke(b, &pl.tessellate(tol));
            } else {
                for s in pl.segments() {
                    let mut pts = Vec::new();
                    s.tessellate(tol, &mut pts);
                    stroke(b, &pts);
                }
            }
        }
        EntityKind::Polyline3d(p) => {
            if p.points.len() < 2 { return; }
            for pair in p.points.windows(2) {
                if let Some((a, c)) = clip_section_segment(ctx.d, pair[0], pair[1]) {
                    stroke(b, &[project3(ctx.d, a), project3(ctx.d, c)]);
                }
            }
            if p.closed && p.points.len() > 2
                && let Some(&last) = p.points.last()
                && let Some((a, c)) = clip_section_segment(ctx.d, last, p.points[0])
            {
                stroke(b, &[project3(ctx.d, a), project3(ctx.d, c)]);
            }
        }
        EntityKind::Helix(h) => {
            let raw = h.fit_points();
            for pair in raw.windows(2) {
                if let Some((a, c)) = clip_section_segment(ctx.d, pair[0], pair[1]) {
                    stroke(b, &[project3(ctx.d, a), project3(ctx.d, c)]);
                }
            }
        }
        EntityKind::Face3d(f) => {
            let triangle = (f.corners[3] - f.corners[2]).len() <= 1e-12;
            if ctx.d.header.i64("BACKFACECULL", 0) != 0 && !section_marker(e) {
                let n = (f.corners[1] - f.corners[0]).cross(f.corners[2] - f.corners[0]);
                if n != Vec3::ZERO && view_normal_depth(ctx.d, n) < 0.0 { return; }
            }
            let base: Vec<Vec3> = if triangle { f.corners[..3].to_vec() } else { f.corners.to_vec() };
            let clipped = if section_marker(e) { base.clone() } else { clip_section_polygon(ctx.d, &base) };
            if clipped.len() < 3 { return; }
            let q: Vec<Vec2> = clipped.iter().copied().map(|p| project3(ctx.d, p)).collect();
            let style=visual_style(ctx.d);
            let filled=matches!(style.as_str(), "Conceptual"|"Realistic"|"Shaded"|"ShadedWithEdges"|"X-ray");
            let draw_edges=!matches!(style.as_str(), "Shaded");
            if b.opts.fill && filled && ctx.d.header.i64("FILLMODE", 1) != 0 {
                let material=cadcraft_color::Rgb::parse_hex(&ctx.d.header.str("MATERIALCOLOR", "")).unwrap_or(rgb.rgb);
                let mut face_ink=rgb;
                face_ink.rgb=shade_face(ctx.d, material, &f.corners);
                let saved=b.transparency;
                if style=="X-ray" { b.transparency=b.transparency.max(65); }
                let mut tris = Vec::with_capacity((q.len() - 2) * 3);
                for i in 1..q.len()-1 { tris.extend_from_slice(&[q[0], q[i], q[i+1]]); }
                b.tris(ctx, face_ink, &tris);
                b.transparency=saved;
            }
            if draw_edges {
                // If clipping introduced intersection vertices, draw the exact clipped boundary.
                if q.len() != base.len() || active_section(ctx.d).is_some() {
                    let mut outline=q.clone(); outline.push(q[0]); stroke(b,&outline);
                } else if triangle {
                    let edge = |bit: u8, a: Vec2, c: Vec2, b: &mut Builder| { if f.hidden_edges & bit == 0 { stroke(b, &[a, c]); } };
                    edge(1,q[0],q[1],b); edge(2,q[1],q[2],b); edge(4,q[2],q[0],b);
                } else {
                    let edge = |bit: u8, a: Vec2, c: Vec2, b: &mut Builder| { if f.hidden_edges & bit == 0 { stroke(b, &[a, c]); } };
                    edge(1,q[0],q[1],b); edge(2,q[1],q[2],b); edge(4,q[2],q[3],b); edge(8,q[3],q[0],b);
                }
            }
            if !section_marker(e) && ctx.d.header.i64("SECTIONCUTLINES", 1) != 0 && active_section(ctx.d).is_some() {
                let hits=section_cut_points(ctx.d,&base);
                if hits.len() >= 2 {
                    let cut: Vec<Vec2> = hits.iter().copied().map(|p|project3(ctx.d,p)).collect();
                    for pair in cut.chunks_exact(2) { b.hairline(ctx,rgb,&[pair[0],pair[1]]); }
                }
            }
        }
        EntityKind::Solid(s) | EntityKind::Trace(s) => {
            let c = &s.corners;
            let q = [c[0].xy(), c[1].xy(), c[3].xy(), c[2].xy()];
            if b.opts.fill && ctx.d.header.i64("FILLMODE", 1) != 0 {
                b.tris(ctx, rgb, &[q[0], q[1], q[2], q[0], q[2], q[3]]);
            } else {
                b.polyline(ctx, rgb, lw, &[q[0], q[1], q[2], q[3], q[0]]);
            }
        }
        EntityKind::Wipeout(w) => {
            // Hide what was drawn before it (draw order), then the frame per WIPEOUTFRAME:
            // 0 = none, 1 = shown and plotted, 2 = shown but not plotted.
            if b.opts.fill {
                let n = b.list.prims.len();
                b.tris(ctx, rgb, &fill::triangulate_evenodd(std::slice::from_ref(&w.boundary)));
                if let Some(p) = b.list.prims.get_mut(n) {
                    p.kind = Kind::Mask;
                }
            }
            let frame = match ctx.d.header.i64("WIPEOUTFRAME", 1) {
                0 => false,
                2 => !b.plotting,
                _ => true,
            };
            if frame {
                let mut pts = w.boundary.clone();
                if let Some(f) = pts.first().copied() {
                    pts.push(f);
                }
                b.polyline(ctx, rgb, lw, &pts);
            }
        }
        EntityKind::Unknown(u) if u.dxf_type.to_ascii_uppercase().ends_with("UNDERLAY") => {
            let value = |code: i32, default: f64| {
                u.tags.iter().find(|t| t.code == code).and_then(|t| t.value.parse::<f64>().ok()).unwrap_or(default)
            };
            let o = Vec2::new(value(10, 0.0), value(20, 0.0));
            let uvec = Vec2::new(value(11, 1.0), value(21, 0.0));
            let vvec = Vec2::new(value(12, 0.0), value(22, 1.0));
            let clipping = u.tags.iter().find(|t| t.code == 280).is_some_and(|t| t.value != "0" && t.value != "false");
            let xs: Vec<f64> = u.tags.iter().filter(|t| t.code == 14).filter_map(|t| t.value.parse::<f64>().ok()).collect();
            let ys: Vec<f64> = u.tags.iter().filter(|t| t.code == 24).filter_map(|t| t.value.parse::<f64>().ok()).collect();
            if clipping && xs.len() >= 3 && xs.len() == ys.len() {
                let mut pts: Vec<Vec2> = xs.into_iter().zip(ys).map(|(x, y)| Vec2::new(x, y)).collect();
                if let Some(first) = pts.first().copied() { pts.push(first); }
                b.polyline(ctx, rgb, lw, &pts);
            } else {
                let a = o;
                let c = o + uvec + vvec;
                let frame = [a, o + uvec, c, o + vvec, a];
                b.polyline(ctx, rgb, lw, &frame);
                b.polyline(ctx, rgb, lw, &[a, c]);
                b.polyline(ctx, rgb, lw, &[o + uvec, o + vvec]);
            }
        }
        EntityKind::Image(i) => {
            let o = i.insert.xy();
            let u = i.u.xy() * i.size.x;
            let v = i.v.xy() * i.size.y;
            b.polyline(ctx, rgb, lw, &[o, o + u, o + u + v, o + v, o]);
            b.polyline(ctx, rgb, lw, &[o, o + u + v]);
            b.polyline(ctx, rgb, lw, &[o + u, o + v]);
        }
        EntityKind::Table(t) => table(b, ctx, t, rgb, lw),
        EntityKind::Tolerance(t) => tolerance::draw(b, ctx, e, t, lw, tol),
        EntityKind::Leader(l) => {
            let pts: Vec<Vec2> = l.vertices.iter().map(|v| v.xy()).collect();
            stroke(b, &pts);
            if l.arrow
                && let (Some(a), Some(n)) = (pts.first(), pts.get(1))
            {
                let st = ctx.d.dim_style(&l.style);
                // DIMASZ × DIMSCALE of the leader's style (the drawing's DIMSCALE when it is 0).
                let size = st.map(|s| s.arrow_size).unwrap_or(0.18) * leader_scale(ctx.d, st);
                // DIMLDRBLK ("" = closed filled).
                let kind = dim::Arrowhead::parse(st.map(|s| s.leader_arrow_block.as_str()).unwrap_or(""));
                let g = dim::arrowhead(kind, *a, (*a - *n).normalized(), size);
                for l in &g.lines {
                    b.polyline(ctx, rgb, lw, l);
                }
                b.tris(ctx, rgb, &g.tris);
            }
        }
        EntityKind::MLeader(m) => {
            // Straight or spline leader lines; block content draws like a block reference.
            for pts in m.leader_paths() {
                b.polyline(ctx, rgb, lw, &pts);
                if let (Some(a), Some(n)) = (pts.first(), pts.get(1)) {
                    b.tris(ctx, rgb, &dim::arrow(*a, (*a - *n).normalized(), m.arrow_size));
                }
            }
            if let Some(t) = &m.text {
                let land = m.landing.xy();
                let dir = if t.insert.x >= land.x { 1.0 } else { -1.0 };
                b.polyline(ctx, rgb, lw, &[land, land + Vec2::new(m.dogleg * dir, 0.0)]);
                mtext(b, ctx, t, rgb, lw);
            }
            if let Some(ins) = &m.block {
                if m.dogleg > 0.0 && !m.leaders.is_empty() {
                    let land = m.landing.xy();
                    let dir = if ins.insert.x >= land.x { 1.0 } else { -1.0 };
                    b.polyline(ctx, rgb, lw, &[land, land + Vec2::new(m.dogleg * dir, 0.0)]);
                }
                insert(b, ctx, e, ins, rgb);
            }
        }
        EntityKind::Point(p) => {
            if clipped_by_section(ctx.d, &[p.p]) { return; }
            // 3D points (including point-cloud imports) follow the current orbit projection.
            let at = if p.p.z.abs() > 1e-12 { project3(ctx.d, p.p) } else { p.p.xy() };
            let view = b.opts.view_height / ctx.xf.scale_factor().max(1e-12);
            let cloud = pointcloud_entity(e);
            let raw_size = if cloud { ctx.d.header.f64("POINTCLOUDSIZE", 1.5) } else { ctx.d.header.f64("PDSIZE", 0.0) };
            let size = if cloud { raw_size / ctx.xf.scale_factor().max(1e-12) } else { point::size(raw_size, view) };
            let mode = if cloud { ctx.d.header.i64("POINTCLOUDMODE", 0) } else { ctx.d.header.i64("PDMODE", 0) };
            let fig = point::figure(mode, size, at, p.angle, tol);
            if fig.dot {
                b.point(ctx, rgb, at);
            }
            for l in &fig.lines {
                b.polyline(ctx, rgb, lw, l);
            }
        }
        kind => {
            for prim in kind.prims() {
                match prim {
                    Prim::Seg(s) => {
                        let mut pts = Vec::new();
                        s.tessellate(tol, &mut pts);
                        stroke(b, &pts);
                    }
                    Prim::Circle(c) => {
                        let mut pts = Vec::new();
                        c.tessellate(tol, &mut pts);
                        stroke(b, &pts);
                    }
                    Prim::Ellipse(el) => {
                        let mut pts = Vec::new();
                        el.tessellate(tol, &mut pts);
                        stroke(b, &pts);
                    }
                    Prim::Spline(s) => stroke(b, &s.tessellate(tol)),
                    Prim::Point(p) => b.point(ctx, rgb, p),
                    Prim::Infinite { base, dir, ray } => b.infinite(ctx, rgb, base, dir, ray),
                    Prim::Fill(pts) => {
                        let tris = fill::triangulate_evenodd(&[pts]);
                        b.tris(ctx, rgb, &tris);
                    }
                }
            }
        }
    }
}

/// Draw a polyline with a linetype (None: continuous).
pub(crate) fn stroke_with(b: &mut Builder, ctx: &Ctx, rgb: Ink, lw: f32, lt: Option<&cadcraft_doc::Linetype>, ltscale: f64, pts: &[Vec2]) {
    match lt {
        Some(lt) => {
            let min = b.opts.min_dash / ctx.xf.scale_factor().max(1e-12);
            for dash in linetype::apply(pts, lt, ltscale, min) {
                match dash.as_slice() {
                    [p] => b.point(ctx, rgb, *p),
                    _ => b.polyline(ctx, rgb, lw, &dash),
                }
            }
        }
        None => b.polyline(ctx, rgb, lw, pts),
    }
}

fn table(b: &mut Builder, ctx: &Ctx, t: &cadcraft_doc::Table, rgb: Ink, lw: f32) {
    table::draw(b, ctx, t, rgb, lw);
}

/// The overall scale of a leader drawn with dimension style `st`: its DIMSCALE, else the
/// drawing's.
fn leader_scale(d: &Drawing, st: Option<&cadcraft_doc::DimStyle>) -> f64 {
    let dimscale = d.header.f64("DIMSCALE", 1.0);
    st.map(|s| s.effective_scale(dimscale)).unwrap_or_else(|| cadcraft_doc::DimStyle::default().effective_scale(dimscale))
}
