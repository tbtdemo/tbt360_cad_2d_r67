//! Region booleans (UNION, SUBTRACT, INTERSECT) and MASSPROP.
//!
//! The booleans handle disjoint/nested boundaries directly. INTERSECT also supports the safe
//! convex-polygon overlap path; general crossing curves still report an explicit limitation.

use cadcraft_doc::{EntityKind, Handle, HatchLoop, MAX_REGION_LOOPS, Region};
use cadcraft_geom::{Line, Vec2, line_line, point_in_polygon};
use serde_json::{Value, json};

use super::super::machines::{SelOutcome, SelectPhase, SelectRun};
use super::super::{CommandSpec, bad, targets};

use super::SelfCheck;
use crate::{EngineError, Input, Interactive, Prompt, Result, Session, Step};

/// The region commands: UNION, SUBTRACT and INTERSECT combine regions, MASSPROP measures them.
pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("union", "Union", run_union)
            .menu(&["Modify", "Solid Editing", "Union"])
            .alias(&["uni"])
            .params("{handles?} (regions)")
            .interactive(|_| Ok(Box::new(SelectRun::new("union", "UNION")))),
        CommandSpec::new("subtract", "Subtract", run_subtract)
            .menu(&["Modify", "Solid Editing", "Subtract"])
            .alias(&["su"])
            .params("{from: [handles], remove: [handles]} (regions)")
            .interactive(|_| Ok(Box::new(SubtractM::default()))),
        CommandSpec::new("intersect", "Intersect", run_intersect)
            .menu(&["Modify", "Solid Editing", "Intersect"])
            .alias(&["in"])
            .params("{handles?} (regions)")
            .interactive(|_| Ok(Box::new(SelectRun::new("intersect", "INTERSECT")))),
        CommandSpec::new("massprop", "Region/Mass Properties", run_massprop)
            .menu(&["Tools", "Inquiry", "Region/Mass Properties"])
            .params("{handles?} (regions) → properties of each, plus `selfCheck` (found/analyzed/skipped with reasons, caps, implausible values)")
            .noundo()
            .interactive(|_| Ok(Box::new(SelectRun::new("massprop", "MASSPROP")))),
        CommandSpec::new("regioncheck", "Region Diagnostics", run_regioncheck)
            .alias(&["regiondiag"])
            .params("{handles?} → diagnostics for REGION loops, validity, area and self-intersection risk")
            .noundo(),
    ]
}

fn other(m: impl Into<String>) -> EngineError {
    EngineError::Other(m.into())
}

/// Most segment pairs compared when checking two regions for crossing boundaries.
const MAX_PAIRS: usize = 50_000_000;

/// The regions among `hs` (others are left alone).
fn regions(s: &Session, hs: &[Handle]) -> Result<Vec<(Handle, Region)>> {
    let d = s.doc()?;
    Ok(hs
        .iter()
        .filter_map(|h| match &d.entity(*h)?.kind {
            EntityKind::Region(r) => Some((*h, r.clone())),
            _ => None,
        })
        .collect())
}

/// Each loop of a region as points, with a chord tolerance for both regions' size.
fn tess(r: &Region, tol: f64) -> Vec<Vec<Vec2>> {
    r.tessellate(tol)
}

/// A point on a loop's boundary (its first edge's middle).
fn probe(l: &[Vec2]) -> Option<Vec2> {
    let n = l.len();
    (0..n).find_map(|i| {
        let (a, b) = (*l.get(i)?, *l.get((i + 1) % n)?);
        (a.dist(b) > 1e-12).then(|| (a + b) * 0.5)
    })
}

/// Whether two sets of loops have boundaries that cross, touch or overlap.
fn boundaries_meet(a: &[Vec<Vec2>], b: &[Vec<Vec2>], tol: f64) -> Result<bool> {
    let edges = |ls: &[Vec<Vec2>]| -> Vec<Line> {
        ls.iter().flat_map(|l| (0..l.len()).filter_map(move |i| Some(Line::new(*l.get(i)?, *l.get((i + 1) % l.len())?)))).collect()
    };
    let (ea, eb) = (edges(a), edges(b));
    if ea.len().saturating_mul(eb.len()) > MAX_PAIRS {
        return Err(other("These regions are too complex to combine."));
    }
    for x in &ea {
        for y in &eb {
            if line_line(x, y).is_some() || x.dist(y.a) < tol || y.dist(x.a) < tol {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Inside the area of loops `ls` (odd parity).
fn inside(ls: &[Vec<Vec2>], p: Vec2) -> bool {
    ls.iter().filter(|l| point_in_polygon(l, p)).count() % 2 == 1
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Op {
    Union,
    Subtract,
    Intersect,
}

/// Clip a convex polygon by another convex polygon. This intentionally excludes arcs and
/// concave/holey regions; those continue through the conservative unsupported path.
fn convex_intersection(a: &[Vec<Vec2>], b: &[Vec<Vec2>]) -> Option<Region> {
    if a.len() != 1 || b.len() != 1 || a[0].len() < 3 || b[0].len() < 3 {
        return None;
    }
    let clean = |src: &[Vec2]| {
        let mut v: Vec<Vec2> = src.iter().copied().filter(|p| p.is_finite()).collect();
        if v.len() > 1 && v.first() == v.last() {
            v.pop();
        }
        v
    };
    let mut subject = clean(&a[0]);
    let clip = clean(&b[0]);
    let area = |v: &[Vec2]| v.iter().enumerate().map(|(i, p)| p.cross(v[(i + 1) % v.len()])).sum::<f64>() * 0.5;
    let convex = |v: &[Vec2]| {
        let sign = area(v).signum();
        sign != 0.0 && (0..v.len()).all(|i| {
            let x = (v[(i + 1) % v.len()] - v[i]).cross(v[(i + 2) % v.len()] - v[(i + 1) % v.len()]);
            x.abs() < 1e-10 || x.signum() == sign
        })
    };
    if !convex(&subject) || !convex(&clip) {
        return None;
    }
    let orientation = area(&clip).signum();
    for i in 0..clip.len() {
        let edge_a = clip[i];
        let edge_b = clip[(i + 1) % clip.len()];
        let inside = |p: Vec2| (edge_b - edge_a).cross(p - edge_a) * orientation >= -1e-9;
        let input = std::mem::take(&mut subject);
        if input.is_empty() {
            break;
        }
        let mut out = Vec::new();
        for j in 0..input.len() {
            let p = input[j];
            let q = input[(j + 1) % input.len()];
            let pin = inside(p);
            let qin = inside(q);
            if pin != qin {
                let d = q - p;
                let e = edge_b - edge_a;
                let den = d.cross(e);
                if den.abs() > 1e-12 {
                    let t = (edge_a - p).cross(e) / den;
                    out.push(p + d * t.clamp(0.0, 1.0));
                }
            }
            if pin && qin {
                out.push(q);
            } else if !pin && qin {
                out.push(q);
            }
        }
        subject = out;
    }
    if subject.len() < 3 || area(&subject).abs() < 1e-12 {
        return Some(Region { loops: Vec::new() });
    }
    Some(Region::from_loop(subject.into_iter().map(cadcraft_geom::PolyVertex::new).collect()))
}

/// Boolean two axis-aligned rectangular regions by a bounded cell decomposition. The result is
/// a set of non-overlapping rectangular loops; this deliberately refuses rotated/curved/concave
/// inputs so it cannot silently produce an invalid region.
fn axis_rect(r: &Region) -> Option<(Vec2, Vec2)> {
    if r.loops.len() != 1 || r.loops[0].vertices.len() != 4 || r.loops[0].vertices.iter().any(|v| v.bulge.abs() > 1e-12) {
        return None;
    }
    let pts: Vec<Vec2> = r.loops[0].vertices.iter().map(|v| v.p).collect();
    let min = pts.iter().copied().reduce(|a, p| Vec2::new(a.x.min(p.x), a.y.min(p.y)))?;
    let max = pts.iter().copied().reduce(|a, p| Vec2::new(a.x.max(p.x), a.y.max(p.y)))?;
    (max.x - min.x > 1e-12 && max.y - min.y > 1e-12
        && pts.iter().all(|p| (p.x == min.x || p.x == max.x) && (p.y == min.y || p.y == max.y)))
        .then_some((min, max))
}

fn rect_boolean(a: &Region, b: &Region, op: Op) -> Option<Region> {
    let (amin, amax) = axis_rect(a)?;
    let (bmin, bmax) = axis_rect(b)?;
    let mut xs = vec![amin.x, amax.x, bmin.x, bmax.x];
    let mut ys = vec![amin.y, amax.y, bmin.y, bmax.y];
    xs.sort_by(f64::total_cmp);
    ys.sort_by(f64::total_cmp);
    xs.dedup_by(|x, y| (*x - *y).abs() < 1e-12);
    ys.dedup_by(|x, y| (*x - *y).abs() < 1e-12);
    let mut loops = Vec::new();
    for wx in xs.windows(2) {
        for wy in ys.windows(2) {
            let (x0, x1, y0, y1) = (wx[0], wx[1], wy[0], wy[1]);
            let p = Vec2::new((x0 + x1) * 0.5, (y0 + y1) * 0.5);
            let ia = p.x >= amin.x && p.x <= amax.x && p.y >= amin.y && p.y <= amax.y;
            let ib = p.x >= bmin.x && p.x <= bmax.x && p.y >= bmin.y && p.y <= bmax.y;
            let keep = match op {
                Op::Union => ia || ib,
                Op::Subtract => ia && !ib,
                Op::Intersect => ia && ib,
            };
            if keep {
                let pts = [Vec2::new(x0, y0), Vec2::new(x1, y0), Vec2::new(x1, y1), Vec2::new(x0, y1)];
                loops.push(HatchLoop { vertices: pts.into_iter().map(cadcraft_geom::PolyVertex::new).collect(), outer: true, sources: Vec::new() });
            }
        }
    }
    (loops.len() <= MAX_REGION_LOOPS).then_some(Region { loops })
}

fn overlap_diagnostic(a: &Region, b: &Region, op: Op) -> String {
    let multiple = a.loops.len() > 1 || b.loops.len() > 1;
    let curved = a.loops.iter().chain(&b.loops).any(|l| l.vertices.iter().any(|v| v.bulge.abs() > 1e-12));
    let operation = match op {
        Op::Intersect => "INTERSECT",
        Op::Union => "UNION",
        Op::Subtract => "SUBTRACT",
    };
    if multiple {
        return format!("{operation}: overlapping regions with multiple loops or holes are not supported yet; use single-loop regions.");
    }
    if curved {
        return format!("{operation}: overlapping curved boundaries are not supported yet; convert the boundaries to straight segments first.");
    }
    match op {
        Op::Intersect => format!("{operation}: overlapping boundaries require convex polygon loops; concave or rotated/curved cases are not supported yet."),
        Op::Union | Op::Subtract => format!("{operation}: overlapping boundaries currently support axis-aligned rectangular regions only."),
    }
}

/// `a` op `b` for regions whose boundaries don't meet; INTERSECT additionally supports
/// overlapping convex polygon boundaries. Other crossing curves remain conservative.
fn combine(a: &Region, b: &Region, op: Op) -> Result<Option<Region>> {
    let size = a.bounds().union(&b.bounds());
    let tol = ((size.width() + size.height()) * 1e-6).max(1e-12);
    let (ta, tb) = (tess(a, tol), tess(b, tol));
    if boundaries_meet(&ta, &tb, tol * 10.0)? {
        if op == Op::Intersect {
            return convex_intersection(&ta, &tb).map(Some).ok_or_else(|| other(overlap_diagnostic(a, b, op)));
        }
        if matches!(op, Op::Union | Op::Subtract) {
            return rect_boolean(a, b, op).map(Some).ok_or_else(|| other(overlap_diagnostic(a, b, op)));
        }
        return Ok(None);
    }
    let keep = |loops: &[HatchLoop], pts: &[Vec<Vec2>], other: &[Vec<Vec2>], want_inside: bool| -> Vec<HatchLoop> {
        loops.iter().zip(pts).filter(|(_, p)| probe(p).is_some_and(|q| inside(other, q) == want_inside)).map(|(l, _)| l.clone()).collect()
    };
    let mut loops = match op {
        Op::Union => [keep(&a.loops, &ta, &tb, false), keep(&b.loops, &tb, &ta, false)].concat(),
        Op::Subtract => [keep(&a.loops, &ta, &tb, false), keep(&b.loops, &tb, &ta, true)].concat(),
        Op::Intersect => [keep(&a.loops, &ta, &tb, true), keep(&b.loops, &tb, &ta, true)].concat(),
    };
    if loops.len() > MAX_REGION_LOOPS {
        return Err(other("Too many loops."));
    }
    loops.retain(|l| l.vertices.len() >= 2);
    let mut r = Region { loops };
    r.renest();
    Ok(Some(r))
}

const OVERLAP: &str = "Combining regions whose boundaries overlap is not available yet.";

/// Replace the first region with `r` (or erase it when empty) and erase the others.
fn apply(s: &mut Session, keep: Handle, r: Region, erase: &[Handle]) -> Result<()> {
    let d = s.doc_mut()?;
    for h in erase.iter().filter(|h| **h != keep) {
        d.remove_entity(*h);
    }
    if r.loops.is_empty() {
        d.remove_entity(keep);
    } else {
        d.modify_entity(keep,|e|{
            e.kind=EntityKind::Region(r);
            crate::cmd::modify::bump_tbt360_bim_revision(&mut e.common,keep);
        })?;
    }
    Ok(())
}

fn fold(s: &mut Session, hs: &[Handle], op: Op) -> Result<Value> {
    let rs = regions(s, hs)?;
    let Some(((h0, first), rest)) = rs.split_first() else { return Err(other("Select at least one region.")) };
    let mut acc = first.clone();
    for (_, r) in rest {
        acc = combine(&acc, r, op)?.ok_or_else(|| other(OVERLAP))?;
    }
    let all: Vec<Handle> = rs.iter().map(|(h, _)| *h).collect();
    let empty = acc.loops.is_empty();
    apply(s, *h0, acc, &all)?;
    Ok(
        json!({ "handle": if empty { Value::Null } else { json!(h0.hex()) }, "message": if empty { "The result is empty: the regions were erased." } else { "" } }),
    )
}

fn run_union(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    fold(s, &hs, Op::Union)
}

fn run_intersect(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    fold(s, &hs, Op::Intersect)
}

fn handles(p: &Value, key: &str) -> Result<Vec<Handle>> {
    let a = p.get(key).and_then(Value::as_array).ok_or_else(|| bad("subtract", format!("`{key}` (array of handles) is required")))?;
    a.iter().map(|v| super::super::curves::handle_value(v).ok_or_else(|| bad("subtract", format!("`{key}`: {v} is not a handle")))).collect()
}

fn subtract(s: &mut Session, from: &[Handle], remove: &[Handle]) -> Result<Value> {
    let (a, b) = (regions(s, from)?, regions(s, remove)?);
    let Some(((h0, first), rest)) = a.split_first() else { return Err(other("Select regions to subtract from.")) };
    let mut acc = first.clone();
    for (_, r) in rest {
        acc = combine(&acc, r, Op::Union)?.ok_or_else(|| other(OVERLAP))?;
    }
    for (h, r) in &b {
        if a.iter().any(|(x, _)| x == h) {
            return Err(other("A region can't be subtracted from itself."));
        }
        acc = combine(&acc, r, Op::Subtract)?.ok_or_else(|| other(OVERLAP))?;
    }
    let all: Vec<Handle> = a.iter().chain(&b).map(|(h, _)| *h).collect();
    let empty = acc.loops.is_empty();
    apply(s, *h0, acc, &all)?;
    Ok(
        json!({ "handle": if empty { Value::Null } else { json!(h0.hex()) }, "message": if empty { "The result is empty: the regions were erased." } else { "" } }),
    )
}

fn run_subtract(s: &mut Session, p: &Value) -> Result<Value> {
    let (from, remove) = (handles(p, "from")?, handles(p, "remove")?);
    subtract(s, &from, &remove)
}

fn loop_self_intersection(points: &[Vec2]) -> (bool, bool) {
    let n = points.len();
    if n < 4 {
        return (false, false);
    }
    let mut pairs = 0usize;
    for i in 0..n {
        let a = Line::new(points[i], points[(i + 1) % n]);
        for j in (i + 1)..n {
            if j == i + 1 || (i == 0 && j + 1 == n) {
                continue;
            }
            pairs = pairs.saturating_add(1);
            if pairs > MAX_PAIRS {
                return (false, true);
            }
            let b = Line::new(points[j], points[(j + 1) % n]);
            if line_line(&a, &b).is_some() {
                return (true, false);
            }
        }
    }
    (false, false)
}

fn run_regioncheck(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    let d = s.doc()?;
    let mut out = Vec::new();
    for h in hs {
        let Some(e) = d.entity(h) else {
            out.push(json!({ "handle": h.hex(), "valid": false, "reason": "missing object" }));
            continue;
        };
        let EntityKind::Region(r) = &e.kind else {
            out.push(json!({ "handle": h.hex(), "valid": false, "reason": "not a REGION" }));
            continue;
        };
        if !r.is_valid() {
            out.push(json!({ "handle": h.hex(), "valid": false, "loopCount": r.loops.len(), "reason": "invalid loop vertices or non-finite geometry" }));
            continue;
        }
        let loops = tess(r, r.bounds().width().max(r.bounds().height()).max(1.0) * 1e-7);
        let mut crossing = false;
        let mut capped = false;
        for loop_points in loops {
            let (hit, cap) = loop_self_intersection(&loop_points);
            crossing |= hit;
            capped |= cap;
        }
        out.push(json!({
            "handle": h.hex(), "valid": true, "loopCount": r.loops.len(), "area": r.area(),
            "selfIntersection": crossing, "selfIntersectionCapped": capped,
            "bounds": [[r.bounds().min.x, r.bounds().min.y], [r.bounds().max.x, r.bounds().max.y]],
        }));
    }
    Ok(json!({ "schema": 1, "regions": out }))
}

#[derive(Default)]
struct SubtractM {
    sel: SelectPhase,
    from: Option<Vec<Handle>>,
}

impl Interactive for SubtractM {
    fn name(&self) -> &'static str {
        "SUBTRACT"
    }
    /// Tell the user what to select first: the regions to subtract from.
    fn begin(&mut self, s: &mut Session) -> Result<Step> {
        s.echo("Select regions to subtract from ..");
        Ok(Step::Continue)
    }
    /// Prompt for the current selection phase (regions to subtract from, then regions to remove).
    fn prompt(&self, _s: &Session) -> Prompt {
        self.sel.prompt()
    }
    /// The first selection is the regions to subtract from; the second is subtracted from them.
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        match self.sel.feed(s, &i)? {
            SelOutcome::More => Ok(Step::Continue),
            SelOutcome::Empty => Ok(Step::Done),
            SelOutcome::Done(hs) => match self.from.take() {
                None => {
                    self.from = Some(hs);
                    self.sel = SelectPhase::default();
                    s.set_selection(Vec::new());
                    s.echo("Select regions to subtract ..");
                    Ok(Step::Continue)
                }
                Some(from) => {
                    let r = subtract(s, &from, &hs)?;
                    if let Some(m) = r.get("message").and_then(Value::as_str).filter(|m| !m.is_empty()) {
                        s.echo(m.to_string());
                    }
                    s.set_selection(Vec::new());
                    Ok(Step::Done)
                }
            },
        }
    }
}

/// MASSPROP: area, perimeter, bounding box, centroid, moments of inertia and principal directions
/// for each selected region, plus a self-check of what was measured, skipped and found
/// implausible.
fn run_massprop(s: &mut Session, p: &Value) -> Result<Value> {
    let hs = targets(s, p)?;
    let rs = regions(s, &hs)?;
    if rs.is_empty() {
        return Err(other("Select one or more regions."));
    }
    let mut check = SelfCheck::new();
    let d = s.doc()?;
    let extents = d.extents(&s.space());
    // No visible extents means no bound to compare an area against (`implausible` skips an
    // infinite bound).
    let extents_area = if extents.is_empty() { f64::INFINITY } else { extents.width() * extents.height() };
    for h in &hs {
        check.found(1);
        match d.entity(*h).map(|e| &e.kind) {
            Some(EntityKind::Region(_)) => check.analyzed(1),
            Some(kind) => check.skip(kind.type_name(), "not a region", 1),
            None => check.skip("Missing", "no such object", 1),
        }
    }
    let f = |v: f64| super::fmt_d(s, v);
    let mut out = Vec::new();
    let mut lines = Vec::new();
    for (h, r) in &rs {
        let m = r.props();
        let c = m.centroid;
        let diagonal = if m.bounds.is_empty() { 0.0 } else { m.bounds.min.dist(m.bounds.max) };
        check.implausible(*h, "Region", "perimeter", m.perimeter, diagonal * 10.0, "perimeter is more than 10 x the bounding-box diagonal");
        check.implausible(*h, "Region", "area", m.area, extents_area, "area is larger than the drawing extents");
        // Moments about the centroid, and the principal moments and directions.
        let [ixc, iyc, ixyc] = m.centroid_moments;
        let mid = (ixc + iyc) / 2.0;
        let rad = (((ixc - iyc) / 2.0).powi(2) + ixyc * ixyc).sqrt();
        let (i1, i2) = (mid - rad, mid + rad);
        let ang = 0.5 * (-2.0 * ixyc).atan2(ixc - iyc);
        let gyr = |i: f64| if m.area > 0.0 { (i / m.area).abs().sqrt() } else { 0.0 };
        out.push(json!({
            "handle": h.hex(), "area": m.area, "perimeter": m.perimeter,
            "boundingBox": [[m.bounds.min.x, m.bounds.min.y], [m.bounds.max.x, m.bounds.max.y]],
            "centroid": [c.x, c.y], "momentsOfInertia": [m.ixx, m.iyy], "productOfInertia": m.ixy,
            "radiiOfGyration": [gyr(m.ixx), gyr(m.iyy)], "principalMoments": [i1, i2],
            "principalDirections": [[ang.cos(), ang.sin()], [-ang.sin(), ang.cos()]],
        }));
        lines.push(format!(
            "----------------   REGIONS   ----------------\nArea:                    {}\nPerimeter:               {}\nBounding box:         X: {}  --  {}\n                      Y: {}  --  {}\nCentroid:             X: {}\n                      Y: {}\nMoments of inertia:   X: {}\n                      Y: {}\nProduct of inertia:  XY: {}\nRadii of gyration:    X: {}\n                      Y: {}\nPrincipal moments and X-Y directions about centroid:\n                      I: {} along [{:.4} {:.4}]\n                      J: {} along [{:.4} {:.4}]",
            f(m.area),
            f(m.perimeter),
            f(m.bounds.min.x),
            f(m.bounds.max.x),
            f(m.bounds.min.y),
            f(m.bounds.max.y),
            f(c.x),
            f(c.y),
            f(m.ixx),
            f(m.iyy),
            f(m.ixy),
            f(gyr(m.ixx)),
            f(gyr(m.iyy)),
            f(i1),
            ang.cos(),
            ang.sin(),
            f(i2),
            -ang.sin(),
            ang.cos(),
        ));
    }
    Ok(json!({ "regions": out, "selfCheck": check.to_json(), "message": lines.join("\n") }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_supports_overlapping_convex_regions() {
        let mut s = Session::new();
        let a = s.execute("rectang", &json!({ "p1": [0, 0], "p2": [10, 10] })).unwrap()["handle"].as_str().unwrap().to_string();
        let b = s.execute("rectang", &json!({ "p1": [5, 5], "p2": [15, 15] })).unwrap()["handle"].as_str().unwrap().to_string();
        s.execute("region", &json!({ "handles": [a.clone(), b.clone()] })).unwrap();
        let result = s.execute("intersect", &json!({ "handles": [a.clone(), b] })).unwrap();
        assert_eq!(result["handle"], a);
        let h = Handle::parse_hex(result["handle"].as_str().unwrap()).unwrap();
        let EntityKind::Region(r) = &s.doc().unwrap().entity(h).unwrap().kind else { panic!("intersection remains a region") };
        assert!((r.area() - 25.0).abs() < 1e-8, "area = {}", r.area());
    }

    #[test]
    fn union_and_subtract_support_overlapping_axis_aligned_rectangles() {
        let mut s = Session::new();
        let a = s.execute("rectang", &json!({ "p1": [0, 0], "p2": [10, 10] })).unwrap()["handle"].as_str().unwrap().to_string();
        let b = s.execute("rectang", &json!({ "p1": [5, 5], "p2": [15, 15] })).unwrap()["handle"].as_str().unwrap().to_string();
        s.execute("region", &json!({ "handles": [a.clone(), b.clone()] })).unwrap();
        let result = s.execute("union", &json!({ "handles": [a.clone(), b] })).unwrap();
        let h = Handle::parse_hex(result["handle"].as_str().unwrap()).unwrap();
        let EntityKind::Region(r) = &s.doc().unwrap().entity(h).unwrap().kind else { panic!("union remains a region") };
        assert!((r.area() - 175.0).abs() < 1e-8, "union area = {}", r.area());

        let mut s = Session::new();
        let a = s.execute("rectang", &json!({ "p1": [0, 0], "p2": [10, 10] })).unwrap()["handle"].as_str().unwrap().to_string();
        let b = s.execute("rectang", &json!({ "p1": [5, 5], "p2": [15, 15] })).unwrap()["handle"].as_str().unwrap().to_string();
        s.execute("region", &json!({ "handles": [a.clone(), b.clone()] })).unwrap();
        let result = s.execute("subtract", &json!({ "from": [a.clone()], "remove": [b] })).unwrap();
        let h = Handle::parse_hex(result["handle"].as_str().unwrap()).unwrap();
        let EntityKind::Region(r) = &s.doc().unwrap().entity(h).unwrap().kind else { panic!("difference remains a region") };
        assert!((r.area() - 75.0).abs() < 1e-8, "difference area = {}", r.area());
    }

    #[test]
    fn unsupported_overlapping_region_reports_actionable_diagnostic() {
        let mut s = Session::new();
        let a = s.execute("rectang", &json!({ "p1": [0, 0], "p2": [10, 10], "rotation": 20 })).unwrap()["handle"].as_str().unwrap().to_string();
        let b = s.execute("rectang", &json!({ "p1": [5, 5], "p2": [15, 15] })).unwrap()["handle"].as_str().unwrap().to_string();
        s.execute("region", &json!({ "handles": [a.clone(), b.clone()] })).unwrap();
        let e = s.execute("union", &json!({ "handles": [a, b] })).unwrap_err();
        assert!(e.to_string().contains("axis-aligned rectangular"), "{e}");
    }

    #[test]
    fn regioncheck_reports_loops_and_validity() {
        let mut s = Session::new();
        let h = s.execute("rectang", &json!({ "p1": [0, 0], "p2": [10, 5] })).unwrap()["handle"].as_str().unwrap().to_string();
        s.execute("region", &json!({ "handles": [h.clone()] })).unwrap();
        let r = s.execute("regioncheck", &json!({ "handles": [h] })).unwrap();
        let item = &r["regions"][0];
        assert_eq!(item["valid"], true);
        assert_eq!(item["loopCount"], 1);
        assert_eq!(item["selfIntersection"], false);
    }
}
