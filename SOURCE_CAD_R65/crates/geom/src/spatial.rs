//! Rebuildable, immutable acceleration structures for CAD viewport queries.
//! No persistent entity state, hidden entities must be filtered by the caller.
use crate::{closest_point_segment_2d, ray_triangle_3d, Vec2, Vec3};

/// Snap query result: original segment index, nearest point and squared distance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentHit {
    pub index: usize,
    pub point: Vec2,
    pub parameter: f64,
    pub distance: f64,
}

#[derive(Debug, Clone)]
struct SegmentEntry {
    index: usize,
    a: Vec2,
    b: Vec2,
    min_x: f64,
    max_x: f64,
}

/// Sorted sweep index for proximity-based 2D edge snapping.
/// `new` excludes invalid segments. Radius must be finite and nonnegative.
/// Stable tie-break uses original insertion order.
#[derive(Debug, Clone, Default)]
pub struct SegmentIndex2D {
    entries: Vec<SegmentEntry>,
    /// Monotonic prefix maximum enables logarithmic elimination of intervals
    /// entirely left of the query, even for long overlapping segments.
    prefix_max_x: Vec<f64>,
}

impl SegmentIndex2D {
    pub fn new(segments: &[(Vec2, Vec2)]) -> Self {
        let mut entries: Vec<_> = segments.iter().enumerate().filter_map(|(index, &(a, b))| {
            (a.is_finite() && b.is_finite()).then_some(SegmentEntry {
                index, a, b, min_x: a.x.min(b.x), max_x: a.x.max(b.x),
            })
        }).collect();
        entries.sort_by(|a, b| a.min_x.total_cmp(&b.min_x).then(a.index.cmp(&b.index)));
        let mut prefix_max_x = Vec::with_capacity(entries.len());
        let mut running = f64::NEG_INFINITY;
        for entry in &entries {
            running = running.max(entry.max_x);
            prefix_max_x.push(running);
        }
        Self { entries, prefix_max_x }
    }

    pub fn nearest(&self, point: Vec2, radius: f64) -> Option<SegmentHit> {
        if !point.is_finite() || !radius.is_finite() || radius < 0.0 { return None; }
        self.nearest_filtered(point, radius, |_| true)
    }

    /// Query selectable geometry only. The predicate receives the original
    /// segment index; callers can filter hidden, locked or isolated entities.
    pub fn nearest_filtered(
        &self,
        point: Vec2,
        radius: f64,
        mut selectable: impl FnMut(usize) -> bool,
    ) -> Option<SegmentHit> {
        if !point.is_finite() || !radius.is_finite() || radius < 0.0 { return None; }
        let left = point.x - radius;
        let right = point.x + radius;
        if !left.is_finite() || !right.is_finite() { return None; }
        let start = self.prefix_max_x.partition_point(|&max_x| max_x < left);
        let end = self.entries.partition_point(|e| e.min_x <= right);
        if start >= end { return None; }
        let mut best: Option<SegmentHit> = None;
        for entry in &self.entries[start..end] {
            if !selectable(entry.index) || entry.max_x < left || point.y < entry.a.y.min(entry.b.y) - radius ||
                point.y > entry.a.y.max(entry.b.y) + radius { continue; }
            if let Some((nearest, parameter)) = closest_point_segment_2d(point, entry.a, entry.b) {
                let distance = point.dist(nearest);
                if !distance.is_finite() || distance > radius { continue; }
                if best.is_none_or(|b| distance < b.distance ||
                    (distance == b.distance && entry.index < b.index)) {
                    best = Some(SegmentHit { index: entry.index, point: nearest, parameter, distance });
                }
            }
        }
        best
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

/// Supported bounded OSNAP targets on indexed line segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentSnapMode {
    Endpoint,
    Midpoint,
    Nearest,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentSnapHit {
    pub index: usize,
    pub point: Vec2,
    pub mode: SegmentSnapMode,
    pub distance: f64,
}

impl SegmentIndex2D {
    /// Snap to endpoints/midpoints/nearest points of selectable segments.
    /// Each mode has an independent toggle; stable ties prefer the input order
    /// and then Endpoint > Midpoint > Nearest for equal-distance candidates.
    pub fn snap_filtered(
        &self,
        point: Vec2,
        radius: f64,
        endpoint: bool,
        midpoint: bool,
        nearest: bool,
        mut selectable: impl FnMut(usize) -> bool,
    ) -> Option<SegmentSnapHit> {
        if !point.is_finite() || !radius.is_finite() || radius < 0.0 ||
            !(endpoint || midpoint || nearest) { return None; }
        let left = point.x - radius;
        let right = point.x + radius;
        if !left.is_finite() || !right.is_finite() { return None; }
        let start = self.prefix_max_x.partition_point(|&max_x| max_x < left);
        let end = self.entries.partition_point(|e| e.min_x <= right);
        if start >= end { return None; }
        let mut best: Option<(SegmentSnapHit, u8)> = None;
        for entry in &self.entries[start..end] {
            if entry.max_x < left || point.y < entry.a.y.min(entry.b.y) - radius ||
                point.y > entry.a.y.max(entry.b.y) + radius || !selectable(entry.index) { continue; }
            let mut consider = |candidate: Vec2, mode: SegmentSnapMode, priority: u8| {
                let distance = point.dist(candidate);
                if !distance.is_finite() || distance > radius { return; }
                let hit = SegmentSnapHit { index: entry.index, point: candidate, mode, distance };
                if best.is_none_or(|(old, old_priority)| distance < old.distance ||
                    (distance == old.distance && (entry.index < old.index ||
                    (entry.index == old.index && priority < old_priority)))) {
                    best = Some((hit, priority));
                }
            };
            if endpoint {
                consider(entry.a, SegmentSnapMode::Endpoint, 0);
                consider(entry.b, SegmentSnapMode::Endpoint, 0);
            }
            if midpoint {
                let middle = entry.a + (entry.b - entry.a) * 0.5;
                if middle.is_finite() { consider(middle, SegmentSnapMode::Midpoint, 1); }
            }
            if nearest {
                if let Some((candidate, _)) = closest_point_segment_2d(point, entry.a, entry.b) {
                    consider(candidate, SegmentSnapMode::Nearest, 2);
                }
            }
        }
        best.map(|(hit, _)| hit)
    }
}

/// Intersection snapping result. Original segment IDs are preserved.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentIntersectionHit {
    pub first_index: usize,
    pub second_index: usize,
    pub point: Vec2,
    pub distance: f64,
}

/// Finite segments only: a parallel or collinear pair has no unique crossing.
/// Shared endpoints of nonparallel segments are supported.
fn finite_segment_crossing(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Option<Vec2> {
    let r = b - a;
    let s = d - c;
    let determinant = r.cross(s);
    if !determinant.is_finite() || determinant == 0.0 { return None; }
    let t = (c - a).cross(s) / determinant;
    let u = (c - a).cross(r) / determinant;
    if !t.is_finite() || !u.is_finite() || !(0.0..=1.0).contains(&t) ||
        !(0.0..=1.0).contains(&u) { return None; }
    let point = a + r * t;
    point.is_finite().then_some(point)
}

impl SegmentIndex2D {
    /// Finds the closest unique crossing within the cursor aperture. Restricts
    /// candidates by X/Y bounds before checking pairs; pairs of collinear
    /// overlapping segments are intentionally excluded (ambiguous snap point).
    /// Caller supplies visibility/locked/isolation filtering by original index.
    pub fn intersection_filtered(
        &self, point: Vec2, radius: f64,
        mut selectable: impl FnMut(usize) -> bool,
    ) -> Option<SegmentIntersectionHit> {
        if !point.is_finite() || !radius.is_finite() || radius < 0.0 { return None; }
        let left = point.x - radius;
        let right = point.x + radius;
        let bottom = point.y - radius;
        let top = point.y + radius;
        if !left.is_finite() || !right.is_finite() || !bottom.is_finite() || !top.is_finite() {
            return None;
        }
        let start = self.prefix_max_x.partition_point(|&x| x < left);
        let end = self.entries.partition_point(|e| e.min_x <= right);
        if start >= end { return None; }
        let candidates: Vec<_> = self.entries[start..end].iter().filter(|e| {
            e.max_x >= left && e.a.y.min(e.b.y) <= top &&
                e.a.y.max(e.b.y) >= bottom && selectable(e.index)
        }).collect();
        let mut best: Option<SegmentIntersectionHit> = None;
        for (i, a) in candidates.iter().enumerate() {
            for b in candidates.iter().skip(i + 1) {
                if a.max_x < b.min_x || b.max_x < a.min_x ||
                    a.a.y.max(a.b.y) < b.a.y.min(b.b.y) ||
                    b.a.y.max(b.b.y) < a.a.y.min(a.b.y) { continue; }
                let Some(crossing) = finite_segment_crossing(a.a, a.b, b.a, b.b) else { continue; };
                let distance = point.dist(crossing);
                if !distance.is_finite() || distance > radius { continue; }
                let ids = (a.index.min(b.index), a.index.max(b.index));
                if best.is_none_or(|old| distance < old.distance ||
                    (distance == old.distance && ids < (old.first_index, old.second_index))) {
                    best = Some(SegmentIntersectionHit {
                        first_index: ids.0, second_index: ids.1, point: crossing, distance,
                    });
                }
            }
        }
        best
    }
}

/// Facet and user-controlled identity. IDs are not regenerated by index building.
#[derive(Debug, Clone, Copy)]
pub struct PickTriangle3D {
    pub entity_id: u64,
    pub vertices: [Vec3; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FaceHit3D {
    pub triangle_index: usize,
    pub entity_id: u64,
    pub parameter: f64,
    pub barycentric: [f64; 3],
}

#[derive(Debug, Clone, Copy)]
struct Box3 {
    lo: Vec3,
    hi: Vec3,
}
impl Box3 {
    fn of_triangle(t: &PickTriangle3D) -> Self {
        let [a,b,c] = t.vertices;
        Self {
            lo: Vec3::new(a.x.min(b.x).min(c.x), a.y.min(b.y).min(c.y), a.z.min(b.z).min(c.z)),
            hi: Vec3::new(a.x.max(b.x).max(c.x), a.y.max(b.y).max(c.y), a.z.max(b.z).max(c.z)),
        }
    }
    fn union(self, other: Self) -> Self {
        Self { lo: Vec3::new(self.lo.x.min(other.lo.x), self.lo.y.min(other.lo.y), self.lo.z.min(other.lo.z)),
            hi: Vec3::new(self.hi.x.max(other.hi.x), self.hi.y.max(other.hi.y), self.hi.z.max(other.hi.z)) }
    }
    fn centroid(self, axis: usize) -> f64 {
        let (a, b) = match axis { 0 => (self.lo.x, self.hi.x), 1 => (self.lo.y, self.hi.y), _ => (self.lo.z, self.hi.z) };
        a + (b - a) * 0.5
    }
    fn hits(self, origin: Vec3, direction: Vec3, maximum: f64) -> bool {
        let mut near: f64 = 0.0;
        let mut far: f64 = maximum;
        for (o, d, lo, hi) in [
            (origin.x, direction.x, self.lo.x, self.hi.x),
            (origin.y, direction.y, self.lo.y, self.hi.y),
            (origin.z, direction.z, self.lo.z, self.hi.z),
        ] {
            if d == 0.0 {
                if o < lo || o > hi { return false; }
            } else {
                let t0 = (lo - o) / d;
                let t1 = (hi - o) / d;
                near = near.max(t0.min(t1));
                far = far.min(t0.max(t1));
                if near > far { return false; }
            }
        }
        true
    }
}

#[derive(Debug)]
struct Node {
    bounds: Box3,
    start: usize,
    end: usize,
    children: Option<(usize, usize)>,
}

/// Median-split BVH over nondegenerate input facets. Build costs O(n log²n)
/// in the current bounded implementation; picking prunes branches by ray AABB.
#[derive(Debug, Default)]
pub struct FaceIndex3D {
    triangles: Vec<PickTriangle3D>,
    order: Vec<usize>,
    original_indices: Vec<usize>,
    nodes: Vec<Node>,
}
impl FaceIndex3D {
    pub fn new(input: &[PickTriangle3D]) -> Self {
        let accepted: Vec<_> = input.iter().copied().enumerate().filter(|(_, t)| {
            t.vertices.iter().all(|v| v.is_finite()) &&
                (t.vertices[1] - t.vertices[0]).cross(t.vertices[2] - t.vertices[0]).len() > 0.0
        }).collect();
        let original_indices = accepted.iter().map(|(i, _)| *i).collect();
        let triangles: Vec<_> = accepted.into_iter().map(|(_, t)| t).collect();
        let mut index = Self { order: (0..triangles.len()).collect(), triangles, original_indices, nodes: Vec::new() };
        if !index.order.is_empty() { index.build(0, index.order.len()); }
        index
    }
    fn build(&mut self, start: usize, end: usize) -> usize {
        let mut bounds = Box3::of_triangle(&self.triangles[self.order[start]]);
        for &i in &self.order[start + 1..end] { bounds = bounds.union(Box3::of_triangle(&self.triangles[i])); }
        let node_id = self.nodes.len();
        self.nodes.push(Node { bounds, start, end, children: None });
        if end - start > 8 {
            let spans = [bounds.hi.x - bounds.lo.x, bounds.hi.y - bounds.lo.y, bounds.hi.z - bounds.lo.z];
            let axis = if spans[1] > spans[0] && spans[1] >= spans[2] { 1 } else if spans[2] > spans[0] { 2 } else { 0 };
            self.order[start..end].sort_by(|&a, &b| {
                Box3::of_triangle(&self.triangles[a]).centroid(axis)
                    .total_cmp(&Box3::of_triangle(&self.triangles[b]).centroid(axis)).then(a.cmp(&b))
            });
            let mid = start + (end - start) / 2;
            let l = self.build(start, mid);
            let r = self.build(mid, end);
            self.nodes[node_id].children = Some((l, r));
        }
        node_id
    }
    pub fn nearest(&self, origin: Vec3, direction: Vec3) -> Option<FaceHit3D> {
        if !origin.is_finite() || !direction.is_finite() || direction.len() == 0.0 { return None; }
        self.nearest_filtered(origin, direction, f64::INFINITY, |_| true)
    }

    /// Pick within a camera clipping interval. The interval uses ray parameters;
    /// callers using normalized camera rays can pass world-space distances.
    /// This deliberately searches past hits before `min_parameter`, rather than
    /// choosing the nearest hit and rejecting it after the fact.
    pub fn nearest_filtered_range(
        &self,
        origin: Vec3,
        direction: Vec3,
        min_parameter: f64,
        max_parameter: f64,
        selectable: impl FnMut(u64) -> bool,
    ) -> Option<FaceHit3D> {
        if !min_parameter.is_finite() || min_parameter < 0.0 ||
            max_parameter.is_nan() || max_parameter < min_parameter {
            return None;
        }
        self.nearest_filtered_impl(origin, direction, min_parameter, max_parameter, selectable)
    }

    /// Ray-pick within a bounded parameter range, excluding hidden or locked
    /// entities without rebuilding the immutable acceleration structure.
    /// `max_parameter` is ray parameter (not world distance for non-unit rays).
    pub fn nearest_filtered(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_parameter: f64,
        selectable: impl FnMut(u64) -> bool,
    ) -> Option<FaceHit3D> {
        self.nearest_filtered_impl(origin, direction, 0.0, max_parameter, selectable)
    }

    fn nearest_filtered_impl(
        &self,
        origin: Vec3,
        direction: Vec3,
        min_parameter: f64,
        max_parameter: f64,
        mut selectable: impl FnMut(u64) -> bool,
    ) -> Option<FaceHit3D> {
        if !origin.is_finite() || !direction.is_finite() || direction.len() == 0.0 ||
            max_parameter.is_nan() || max_parameter < min_parameter { return None; }
        let mut best: Option<FaceHit3D> = None;
        if self.nodes.is_empty() { return None; }
        let mut stack = vec![0];
        while let Some(node_id) = stack.pop() {
            let node = &self.nodes[node_id];
            if !node.bounds.hits(origin, direction, best.map_or(max_parameter, |b| b.parameter)) { continue; }
            if let Some((l, r)) = node.children {
                stack.push(r);
                stack.push(l);
            } else {
                for &i in &self.order[node.start..node.end] {
                    let triangle = self.triangles[i];
                    if !selectable(triangle.entity_id) { continue; }
                    if let Some((parameter, barycentric)) = ray_triangle_3d(origin, direction,
                        triangle.vertices[0], triangle.vertices[1], triangle.vertices[2]) {
                        if parameter < min_parameter || parameter > max_parameter { continue; }
                        if best.is_none_or(|b| parameter < b.parameter ||
                            (parameter == b.parameter && self.original_indices[i] < b.triangle_index)) {
                            best = Some(FaceHit3D { triangle_index: self.original_indices[i], entity_id: triangle.entity_id, parameter, barycentric });
                        }
                    }
                }
            }
        }
        best
    }
    pub fn len(&self) -> usize { self.triangles.len() }
    pub fn is_empty(&self) -> bool { self.triangles.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexed_snap_modes_respect_selection_and_stable_ties() {
        let segments = [(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0)),
            (Vec2::new(0.0, 2.0), Vec2::new(10.0, 2.0))];
        let index = SegmentIndex2D::new(&segments);
        let q = Vec2::new(5.0, 0.2);
        let midpoint = index.snap_filtered(q, 1.0, false, true, false, |_| true).unwrap();
        assert_eq!(midpoint.mode, SegmentSnapMode::Midpoint);
        assert_eq!(midpoint.point, Vec2::new(5.0, 0.0));
        assert!(index.snap_filtered(q, 1.0, true, false, false, |_| true).is_none());
        let only_second = index.snap_filtered(Vec2::new(5.0, 1.0), 2.0,
            false, true, false, |id| id == 1).unwrap();
        assert_eq!(only_second.index, 1);
        assert!(index.snap_filtered(q, 1.0, true, true, true, |_| false).is_none());
        assert!(index.snap_filtered(q, f64::NAN, true, true, true, |_| true).is_none());
        assert!(index.snap_filtered(q, 1.0, false, false, false, |_| true).is_none());
        let tie = index.snap_filtered(Vec2::new(5.0, 1.0), 2.0,
            false, true, false, |_| true).unwrap();
        assert_eq!(tie.index, 0);
    }
    #[test]
    fn snap_index_matches_brute_force_and_preserves_original_index() {
        let segments = [(Vec2::ZERO, Vec2::new(10.0, 0.0)),
            (Vec2::new(0.0, 4.0), Vec2::new(10.0, 4.0)),
            (Vec2::new(f64::NAN, 0.0), Vec2::X)];
        let index = SegmentIndex2D::new(&segments);
        assert_eq!(index.len(), 2);
        let hit = index.nearest(Vec2::new(6.0, 3.0), 2.0);
        assert_eq!(hit.map(|h| h.index), Some(1));
        assert!(index.nearest(Vec2::new(6.0, 8.0), 2.0).is_none());
        assert!(index.nearest(Vec2::ZERO, f64::NAN).is_none());
    }
    #[test]
    fn bvh_selects_nearest_surface_and_preserves_entity_id() {
        let make = |z, id| PickTriangle3D { entity_id: id,
            vertices: [Vec3::new(0.0,0.0,z), Vec3::new(2.0,0.0,z), Vec3::new(0.0,2.0,z)] };
        let mut triangles = vec![make(0.0, 80), make(3.0, 90)];
        for i in 0..20 { triangles.push(make(-1.0 - f64::from(i), 100 + i as u64)); }
        let index = FaceIndex3D::new(&triangles);
        let hit = index.nearest(Vec3::new(0.25, 0.25, 5.0), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(hit.map(|h| h.entity_id), Some(90));
        assert_eq!(hit.map(|h| h.triangle_index), Some(1));
        assert!(index.nearest(Vec3::new(5.0, 5.0, 5.0), Vec3::new(0.0,0.0,-1.0)).is_none());
    }
    #[test]
    fn filtered_queries_respect_hidden_identity_and_pick_depth() {
        let segments = [(Vec2::ZERO, Vec2::new(2.0, 0.0)),
            (Vec2::new(0.0, 1.0), Vec2::new(2.0, 1.0))];
        let two_d = SegmentIndex2D::new(&segments);
        let query = Vec2::new(1.0, 0.1);
        assert_eq!(two_d.nearest(query, 2.0).map(|h| h.index), Some(0));
        assert_eq!(two_d.nearest_filtered(query, 2.0, |id| id == 1)
            .map(|h| h.index), Some(1));
        assert!(two_d.nearest_filtered(query, 2.0, |_| false).is_none());
        let face = |z, id| PickTriangle3D { entity_id: id, vertices:
            [Vec3::new(0.0, 0.0, z), Vec3::new(2.0, 0.0, z), Vec3::new(0.0, 2.0, z)] };
        let three_d = FaceIndex3D::new(&[face(3.0, 17), face(1.0, 27)]);
        let origin = Vec3::new(0.25, 0.25, 5.0);
        let ray = Vec3::new(0.0, 0.0, -1.0);
        assert_eq!(three_d.nearest(origin, ray).map(|h| h.entity_id), Some(17));
        assert_eq!(three_d.nearest_filtered(origin, ray, 5.0, |id| id != 17)
            .map(|h| h.entity_id), Some(27));
        assert!(three_d.nearest_filtered(origin, ray, 3.0, |id| id != 17).is_none());
        assert!(three_d.nearest_filtered(origin, ray, f64::NAN, |_| true).is_none());
    }
    #[test]
    fn indexed_picking_matches_linear_over_many_triangles() {
        let mut triangles = Vec::new();
        for i in 0..100 {
            let z = f64::from(i) / 4.0;
            triangles.push(PickTriangle3D { entity_id: i as u64,
                vertices: [Vec3::new(0.0,0.0,z), Vec3::new(3.0,0.0,z), Vec3::new(0.0,3.0,z)] });
        }
        let index = FaceIndex3D::new(&triangles);
        let origin = Vec3::new(0.5, 0.5, 30.0);
        let direction = Vec3::new(0.0, 0.0, -2.0);
        let linear = triangles.iter().enumerate().filter_map(|(i,t)|
            ray_triangle_3d(origin,direction,t.vertices[0],t.vertices[1],t.vertices[2])
                .map(|h| (i,h.0))).min_by(|a,b| a.1.total_cmp(&b.1));
        assert_eq!(index.nearest(origin,direction).map(|h| h.triangle_index), linear.map(|h| h.0));
    }
}

#[cfg(test)]
mod r57_clip_tests {
    use super::*;

    #[test]
    fn near_clip_skips_front_face_without_discarding_rear_face() {
        let face = |z: f64, id| PickTriangle3D {
            entity_id: id,
            vertices: [Vec3::new(-2.0, -2.0, z), Vec3::new(2.0, -2.0, z),
                Vec3::new(0.0, 2.0, z)],
        };
        let index = FaceIndex3D::new(&[face(4.0, 11), face(1.0, 22)]);
        let origin = Vec3::new(0.0, 0.0, 5.0);
        let dir = Vec3::new(0.0, 0.0, -1.0);
        assert_eq!(index.nearest_filtered_range(origin, dir, 0.0, 10.0, |_| true)
            .map(|h| h.entity_id), Some(11));
        assert_eq!(index.nearest_filtered_range(origin, dir, 2.0, 10.0, |_| true)
            .map(|h| h.entity_id), Some(22));
        assert!(index.nearest_filtered_range(origin, dir, 2.0, 3.0, |_| true).is_none());
        assert!(index.nearest_filtered_range(origin, dir, 0.0, 10.0, |_| false).is_none());
        assert!(index.nearest_filtered_range(origin, dir, -1.0, 10.0, |_| true).is_none());
        assert!(index.nearest_filtered_range(origin, dir, 3.0, 2.0, |_| true).is_none());
    }
}

#[cfg(test)]
mod sweep_regression_tests {
    use super::*;
    #[test]
    fn prefix_interval_pruning_preserves_long_crossing_segments() {
        let mut segments = Vec::new();
        for x in -200..200 {
            let x = x as f64;
            segments.push((Vec2::new(x, 40.0), Vec2::new(x + 0.5, 40.0)));
        }
        segments.push((Vec2::new(-300.0, 0.0), Vec2::new(300.0, 0.0)));
        let index = SegmentIndex2D::new(&segments);
        let hit = index.nearest(Vec2::new(180.0, 0.1), 1.0).unwrap();
        assert_eq!(hit.index, segments.len() - 1);
        let snap = index.snap_filtered(Vec2::new(0.0, 0.1), 1.0,
            false, false, true, |_| true).unwrap();
        assert_eq!(snap.index, segments.len() - 1);
    }
    #[test]
    fn indexed_nearest_matches_brute_force_for_many_queries() {
        let segments: Vec<_> = (0..700).map(|i| {
            let x = (i as f64) * 0.3 - 100.0;
            (Vec2::new(x, (i % 13) as f64),
             Vec2::new(x + (i % 17) as f64 * 0.9, (i % 13) as f64 + 0.2))
        }).collect();
        let index = SegmentIndex2D::new(&segments);
        for q in 0..100 {
            let point = Vec2::new(q as f64 * 2.0 - 90.0, (q % 13) as f64);
            let indexed = index.nearest(point, 1.5);
            let mut brute: Option<(usize, f64)> = None;
            for (i, &(a,b)) in segments.iter().enumerate() {
                if let Some((p,_)) = closest_point_segment_2d(point,a,b) {
                    let distance = point.dist(p);
                    if distance <= 1.5 && brute.is_none_or(|(bi,bd)| distance < bd ||
                        (distance == bd && i < bi)) { brute = Some((i,distance)); }
                }
            }
            assert_eq!(indexed.map(|hit| hit.index), brute.map(|(i,_)| i));
        }
    }
}

#[cfg(test)]
mod r60_intersection_tests {
    use super::*;

    #[test]
    fn crossing_endpoint_filter_and_parallel_cases() {
        let segments = [
            (Vec2::new(-2.0, 0.0), Vec2::new(2.0, 0.0)),
            (Vec2::new(0.0, -2.0), Vec2::new(0.0, 2.0)),
            (Vec2::new(-2.0, 1.0), Vec2::new(2.0, 1.0)),
            (Vec2::new(2.0, 0.0), Vec2::new(2.0, 2.0)),
        ];
        let index = SegmentIndex2D::new(&segments);
        let hit = index.intersection_filtered(Vec2::new(0.1, 0.1), 0.5, |_| true).unwrap();
        assert_eq!((hit.first_index, hit.second_index), (0, 1));
        assert!(hit.point.near(Vec2::ZERO, 1e-12));
        assert!(index.intersection_filtered(Vec2::new(0.1, 0.1), 0.5, |i| i != 1).is_none());
        let endpoint = index.intersection_filtered(Vec2::new(2.0, 0.0), 0.01, |_| true).unwrap();
        assert_eq!((endpoint.first_index, endpoint.second_index), (0, 3));
        assert!(index.intersection_filtered(Vec2::new(f64::NAN, 0.0), 1.0, |_| true).is_none());
        assert!(index.intersection_filtered(Vec2::ZERO, -1.0, |_| true).is_none());
    }

    #[test]
    fn collinear_pairs_do_not_produce_ambiguous_intersections() {
        let index = SegmentIndex2D::new(&[
            (Vec2::new(0.0, 0.0), Vec2::new(3.0, 0.0)),
            (Vec2::new(1.0, 0.0), Vec2::new(4.0, 0.0)),
        ]);
        assert!(index.intersection_filtered(Vec2::new(2.0, 0.0), 1.0, |_| true).is_none());
    }
}
