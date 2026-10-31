//! Shared f64 picking primitives for 2D drafting and 3D faceted geometry.
//! Queries never modify canonical document entities.
use crate::{Vec2, Vec3};

/// Nearest point on a finite 2D segment. Degenerate segments resolve to `a`.
/// Rejects invalid geometry or non-finite query coordinates.
pub fn closest_point_segment_2d(p: Vec2, a: Vec2, b: Vec2) -> Option<(Vec2, f64)> {
    if !p.is_finite() || !a.is_finite() || !b.is_finite() {
        return None;
    }
    let d = b - a;
    let scale = d.x.abs().max(d.y.abs());
    if scale == 0.0 {
        return Some((a, 0.0));
    }
    if !scale.is_finite() {
        return None;
    }
    let scaled = d * (1.0 / scale);
    let offset = (p - a) * (1.0 / scale);
    if !offset.is_finite() {
        return None;
    }
    let den = scaled.dot(scaled);
    let t = (offset.dot(scaled) / den).clamp(0.0, 1.0);
    let nearest = a * (1.0 - t) + b * t;
    nearest.is_finite().then_some((nearest, t))
}

/// Double-sided Möller–Trumbore ray/triangle hit, with relative degeneracy
/// tolerance. Returns distance along ray (ray direction need not be unit) and
/// barycentric coordinates (w, u, v). No hit for zero direction or bad inputs.
pub fn ray_triangle_3d(
    origin: Vec3,
    direction: Vec3,
    a: Vec3,
    b: Vec3,
    c: Vec3,
) -> Option<(f64, [f64; 3])> {
    if !origin.is_finite() || !direction.is_finite() || !a.is_finite() || !b.is_finite() || !c.is_finite() {
        return None;
    }
    let e1 = b - a;
    let e2 = c - a;
    let length = direction.len();
    let s1 = e1.len();
    let s2 = e2.len();
    if length == 0.0 || s1 == 0.0 || s2 == 0.0 || !length.is_finite() || !s1.is_finite() || !s2.is_finite() {
        return None;
    }
    let rd = direction * (1.0 / length);
    let u1 = e1 * (1.0 / s1);
    let u2 = e2 * (1.0 / s2);
    let pvec = rd.cross(u2);
    let det = u1.dot(pvec);
    if !det.is_finite() || det.abs() <= 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let tvec = origin - a;
    let bary_u = (tvec * (1.0 / s1)).dot(pvec) * inv;
    if !(-1e-12..=1.0 + 1e-12).contains(&bary_u) { return None; }
    let qvec = tvec.cross(u1);
    // qvec uses the original origin offset; normalize by the second edge length.
    let bary_v = rd.dot(qvec) * inv / s2;
    let normalized_t = u2.dot(qvec) * inv;
    if !(-1e-12..=1.0 + 1e-12).contains(&bary_v) || bary_u + bary_v > 1.0 + 1e-12 { return None; }
    if !normalized_t.is_finite() || normalized_t < 0.0 { return None; }
    let t = normalized_t / length;
    t.is_finite().then_some((t, [1.0 - bary_u - bary_v, bary_u, bary_v]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_projection_and_degenerate() {
        assert_eq!(closest_point_segment_2d(Vec2::new(4.0, 3.0), Vec2::ZERO, Vec2::new(10.0, 0.0)), Some((Vec2::new(4.0, 0.0), 0.4)));
        assert_eq!(closest_point_segment_2d(Vec2::new(50.0, 2.0), Vec2::ZERO, Vec2::new(10.0, 0.0)), Some((Vec2::new(10.0, 0.0), 1.0)));
        assert_eq!(closest_point_segment_2d(Vec2::new(1.0, 1.0), Vec2::ZERO, Vec2::ZERO), Some((Vec2::ZERO, 0.0)));
        assert!(closest_point_segment_2d(Vec2::new(f64::NAN, 0.0), Vec2::ZERO, Vec2::X).is_none());
    }

    #[test]
    fn triangle_hit_and_misses() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 2.0, 0.0);
        let hit = ray_triangle_3d(Vec3::new(0.5, 0.5, 5.0), Vec3::new(0.0, 0.0, -2.0), a, b, c);
        assert!(hit.is_some());
        let (t, bary) = hit.unwrap_or((0.0, [0.0; 3]));
        assert!((t - 2.5).abs() < 1e-10);
        assert!((bary[0] - 0.5).abs() < 1e-10);
        assert!((bary[1] - 0.25).abs() < 1e-10);
        assert!((bary[2] - 0.25).abs() < 1e-10);
        assert!(ray_triangle_3d(Vec3::new(3.0, 3.0, 5.0), Vec3::new(0.0, 0.0, -1.0), a, b, c).is_none());
        assert!(ray_triangle_3d(Vec3::new(0.5, 0.5, 5.0), Vec3::Z, a, b, c).is_none());
        assert!(ray_triangle_3d(Vec3::ZERO, Vec3::ZERO, a, b, c).is_none());
        assert!(ray_triangle_3d(Vec3::ZERO, Vec3::Z, a, b, b).is_none());
    }
}

#[cfg(test)]
mod hardening_tests {
    use super::*;

    #[test]
    fn reversed_triangle_and_parallel_ray() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 2.0, 0.0);
        assert!(ray_triangle_3d(Vec3::new(0.5, 0.5, 1.0), Vec3::new(0.0, 0.0, -1.0), a, c, b).is_some());
        assert!(ray_triangle_3d(Vec3::new(0.5, 0.5, 1.0), Vec3::new(1.0, 0.0, 0.0), a, b, c).is_none());
    }

    #[test]
    fn huge_vector_normalizes_without_overflow() {
        let v = Vec3::new(1e308, 1e308, 0.0).normalized();
        assert!(v.is_finite());
        assert!((v.len() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn invalid_tessellation_tolerance_is_bounded() {
        assert!((1..=4096).contains(&crate::arc_segments(10.0, crate::TAU, f64::NAN)));
        assert!((1..=4096).contains(&crate::arc_segments(10.0, crate::TAU, f64::INFINITY)));
    }
}
