//! Pure, immutable viewport query adapters. These functions do not mutate CAD
//! documents, select UI widgets, or bypass document visibility/locking rules.
use crate::{FaceHit3D, FaceIndex3D, SegmentHit, SegmentIndex2D, SegmentSnapHit, SegmentIntersectionHit, Vec2, Vec3};

/// Pixel coordinates: origin at the top-left, +Y downward.
/// Drawing coordinates: +Y upward. `drawing_center` is at the viewport center.
#[derive(Debug, Clone, Copy)]
pub struct Viewport2D {
    pub width_px: f64,
    pub height_px: f64,
    pub drawing_center: Vec2,
    pub drawing_units_per_pixel: f64,
}

impl Viewport2D {
    pub fn screen_to_world(&self, pixel: Vec2) -> Option<Vec2> {
        if !self.valid() || !pixel.is_finite() || !self.pixel_inside(pixel) { return None; }
        let result = self.drawing_center + Vec2::new(
            (pixel.x - self.width_px * 0.5) * self.drawing_units_per_pixel,
            (self.height_px * 0.5 - pixel.y) * self.drawing_units_per_pixel,
        );
        result.is_finite().then_some(result)
    }

    pub fn world_to_screen(&self, point: Vec2) -> Option<Vec2> {
        if !self.valid() || !point.is_finite() { return None; }
        let delta = (point - self.drawing_center) / self.drawing_units_per_pixel;
        let result = Vec2::new(self.width_px * 0.5 + delta.x, self.height_px * 0.5 - delta.y);
        result.is_finite().then_some(result)
    }

    pub fn nearest_segment(
        &self, index: &SegmentIndex2D, pixel: Vec2, tolerance_px: f64,
        selectable: impl FnMut(usize) -> bool,
    ) -> Option<SegmentHit> {
        if !tolerance_px.is_finite() || tolerance_px < 0.0 { return None; }
        let world = self.screen_to_world(pixel)?;
        let tolerance = tolerance_px * self.drawing_units_per_pixel;
        if !tolerance.is_finite() { return None; }
        index.nearest_filtered(world, tolerance, selectable)
    }

    /// Pixel-tolerance OSNAP query, preserving original segment indices.
    pub fn snap_segment(
        &self, index: &SegmentIndex2D, pixel: Vec2, tolerance_px: f64,
        endpoint: bool, midpoint: bool, nearest: bool,
        selectable: impl FnMut(usize) -> bool,
    ) -> Option<SegmentSnapHit> {
        if !tolerance_px.is_finite() || tolerance_px < 0.0 { return None; }
        let world = self.screen_to_world(pixel)?;
        let radius = tolerance_px * self.drawing_units_per_pixel;
        if !radius.is_finite() { return None; }
        index.snap_filtered(world, radius, endpoint, midpoint, nearest, selectable)
    }

    /// Intersection OSNAP using the same world/pixel aperture and caller filter.
    pub fn snap_intersection(
        &self, index: &SegmentIndex2D, pixel: Vec2, tolerance_px: f64,
        selectable: impl FnMut(usize) -> bool,
    ) -> Option<SegmentIntersectionHit> {
        if !tolerance_px.is_finite() || tolerance_px < 0.0 { return None; }
        let world = self.screen_to_world(pixel)?;
        let radius = tolerance_px * self.drawing_units_per_pixel;
        if !radius.is_finite() { return None; }
        index.intersection_filtered(world, radius, selectable)
    }

    /// Reject events outside the viewport rather than selecting off-canvas entities.
    pub fn pixel_inside(&self, pixel: Vec2) -> bool {
        pixel.is_finite() && pixel.x >= 0.0 && pixel.y >= 0.0 &&
            pixel.x <= self.width_px && pixel.y <= self.height_px
    }

    fn valid(&self) -> bool {
        self.width_px.is_finite() && self.width_px > 0.0 &&
            self.height_px.is_finite() && self.height_px > 0.0 &&
            self.drawing_units_per_pixel.is_finite() && self.drawing_units_per_pixel > 0.0 &&
            self.drawing_center.is_finite()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Projection3D {
    Perspective { vertical_fov_rad: f64 },
    Orthographic { vertical_span_world: f64 },
}

/// Camera forward points into the scene. The supplied up vector may be
/// nonorthogonal; it is corrected to form a right-handed camera basis.
#[derive(Debug, Clone, Copy)]
pub struct ViewportCamera3D {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub width_px: f64,
    pub height_px: f64,
    pub projection: Projection3D,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraRay3D {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl ViewportCamera3D {
    pub fn ray_from_pixel(&self, pixel: Vec2) -> Option<CameraRay3D> {
        if !self.position.is_finite() || !self.forward.is_finite() ||
            !self.up.is_finite() || !pixel.is_finite() ||
            pixel.x < 0.0 || pixel.y < 0.0 ||
            pixel.x > self.width_px || pixel.y > self.height_px ||
            !self.width_px.is_finite() || self.width_px <= 0.0 ||
            !self.height_px.is_finite() || self.height_px <= 0.0 { return None; }
        let forward = self.forward.normalized();
        let right = forward.cross(self.up).normalized();
        let true_up = right.cross(forward).normalized();
        if forward == Vec3::ZERO || right == Vec3::ZERO || true_up == Vec3::ZERO { return None; }
        let nx = (2.0 * pixel.x / self.width_px) - 1.0;
        let ny = 1.0 - (2.0 * pixel.y / self.height_px);
        let aspect = self.width_px / self.height_px;
        let ray = match self.projection {
            Projection3D::Perspective { vertical_fov_rad } => {
                if !vertical_fov_rad.is_finite() || vertical_fov_rad <= 0.0 ||
                    vertical_fov_rad >= std::f64::consts::PI { return None; }
                let scale = (vertical_fov_rad * 0.5).tan();
                CameraRay3D { origin: self.position,
                    direction: (forward + right * (nx * aspect * scale) + true_up * (ny * scale)).normalized() }
            },
            Projection3D::Orthographic { vertical_span_world } => {
                if !vertical_span_world.is_finite() || vertical_span_world <= 0.0 { return None; }
                CameraRay3D { origin: self.position + right * (nx * aspect * vertical_span_world * 0.5)
                    + true_up * (ny * vertical_span_world * 0.5), direction: forward }
            },
        };
        (ray.origin.is_finite() && ray.direction.is_finite() && ray.direction != Vec3::ZERO).then_some(ray)
    }

    /// Pick a face between camera *view-depth* near/far planes (world units).
    /// Unlike distance along an oblique ray, view depth is measured along the
    /// camera forward axis. This matters at the edge of a perspective viewport:
    /// a ray reaches the near/far planes farther from the eye than it does at
    /// the viewport center. The immutable query does not mutate selection.
    pub fn pick_face_clipped(
        &self, index: &FaceIndex3D, pixel: Vec2,
        near_distance: f64, far_distance: f64,
        selectable: impl FnMut(u64) -> bool,
    ) -> Option<FaceHit3D> {
        if !near_distance.is_finite() || near_distance < 0.0 ||
            far_distance.is_nan() || far_distance < near_distance { return None; }
        let ray = self.ray_from_pixel(pixel)?;
        let forward = self.forward.normalized();
        let axial = ray.direction.dot(forward);
        if !axial.is_finite() || axial <= 0.0 { return None; }
        // Orthographic rays are parallel to forward; perspective rays have an
        // oblique direction and must convert axial depths to ray parameters.
        let near_parameter = near_distance / axial;
        let far_parameter = far_distance / axial;
        if !near_parameter.is_finite() || far_parameter.is_nan() { return None; }
        index.nearest_filtered_range(ray.origin, ray.direction,
            near_parameter, far_parameter, selectable)
    }

    /// Maximum distance is measured in world units; generated rays are unit length.
    pub fn pick_face(&self, index: &FaceIndex3D, pixel: Vec2,
        max_distance: f64, selectable: impl FnMut(u64) -> bool) -> Option<FaceHit3D> {
        if max_distance.is_nan() || max_distance < 0.0 { return None; }
        let ray = self.ray_from_pixel(pixel)?;
        index.nearest_filtered(ray.origin, ray.direction, max_distance, selectable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PickTriangle3D;
    #[test]
    fn screen_world_round_trip_and_filtered_snap() {
        let view = Viewport2D { width_px: 800.0, height_px: 600.0,
            drawing_center: Vec2::new(100.0, 200.0), drawing_units_per_pixel: 0.5 };
        let world = Vec2::new(110.0, 220.0);
        let pixel = view.world_to_screen(world).unwrap();
        assert!(view.screen_to_world(pixel).unwrap().near(world, 1e-10));
        let segments = [(Vec2::new(100.0, 200.0), Vec2::new(120.0, 200.0)),
            (Vec2::new(100.0, 205.0), Vec2::new(120.0, 205.0))];
        let index = SegmentIndex2D::new(&segments);
        assert_eq!(view.nearest_segment(&index, Vec2::new(420.0, 298.0), 12.0,
            |id| id == 1).map(|hit| hit.index), Some(1));
        assert!(view.nearest_segment(&index, Vec2::new(420.0, 298.0), 2.0,
            |id| id == 1).is_none());
    }
    #[test]
    fn perspective_and_orthographic_pick_preserve_entity() {
        let face = PickTriangle3D { entity_id: 42,
            vertices: [Vec3::new(-2.0,-2.0,0.0), Vec3::new(2.0,-2.0,0.0), Vec3::new(0.0,2.0,0.0)] };
        let index = FaceIndex3D::new(&[face]);
        let mut camera = ViewportCamera3D { position: Vec3::new(0.0,0.0,5.0),
            forward: Vec3::new(0.0,0.0,-1.0), up: Vec3::new(0.0,1.0,0.0), width_px: 800.0,
            height_px: 600.0, projection: Projection3D::Perspective { vertical_fov_rad: 1.0 } };
        assert_eq!(camera.pick_face(&index, Vec2::new(400.0,300.0), 6.0, |_| true)
            .map(|hit| hit.entity_id), Some(42));
        assert!(camera.pick_face(&index, Vec2::new(400.0,300.0), 4.0, |_| true).is_none());
        assert!(camera.pick_face(&index, Vec2::new(400.0,300.0), 6.0, |_| false).is_none());
        camera.projection = Projection3D::Orthographic { vertical_span_world: 10.0 };
        assert_eq!(camera.pick_face(&index, Vec2::new(400.0,300.0), 6.0, |_| true)
            .map(|hit| hit.entity_id), Some(42));
        assert!(camera.pick_face(&index, Vec2::new(799.0,599.0), 6.0, |_| true).is_none());
    }
    #[test]
    fn osnap_pixel_tolerance_endpoint_midpoint_nearest() {
        let view = Viewport2D { width_px: 800.0, height_px: 600.0,
            drawing_center: Vec2::ZERO, drawing_units_per_pixel: 0.5 };
        let index = SegmentIndex2D::new(&[(Vec2::new(-10.0, 0.0), Vec2::new(10.0, 0.0))]);
        let center = Vec2::new(400.0, 301.0);
        let hit = view.snap_segment(&index, center, 3.0, false, true, false, |_| true).unwrap();
        assert_eq!(hit.point, Vec2::ZERO);
        assert!(view.snap_segment(&index, center, 3.0, true, false, false, |_| true).is_none());
        assert!(view.snap_segment(&index, Vec2::new(-1.0, 300.0), 500.0,
            true, true, true, |_| true).is_none());
    }
    #[test]
    fn outside_viewport_never_selects_objects() {
        let view = Viewport2D { width_px: 800.0, height_px: 600.0,
            drawing_center: Vec2::ZERO, drawing_units_per_pixel: 1.0 };
        let segments = [(Vec2::new(-400.0, 0.0), Vec2::new(400.0, 0.0))];
        let index = SegmentIndex2D::new(&segments);
        for pixel in [Vec2::new(-0.1, 300.0), Vec2::new(800.1, 300.0),
            Vec2::new(400.0, -1.0), Vec2::new(400.0, 601.0)] {
            assert!(view.screen_to_world(pixel).is_none());
            assert!(view.nearest_segment(&index, pixel, 100.0, |_| true).is_none());
        }
        let camera = ViewportCamera3D { position: Vec3::new(0.0, 0.0, 5.0),
            forward: Vec3::new(0.0, 0.0, -1.0), up: Vec3::new(0.0, 1.0, 0.0),
            width_px: 800.0, height_px: 600.0,
            projection: Projection3D::Perspective { vertical_fov_rad: 1.0 } };
        assert!(camera.ray_from_pixel(Vec2::new(-1.0, 300.0)).is_none());
        assert!(camera.ray_from_pixel(Vec2::new(400.0, 601.0)).is_none());
        assert!(camera.ray_from_pixel(Vec2::new(400.0, 300.0)).is_some());
    }
    #[test]
    fn invalid_camera_and_viewport_are_rejected() {
        let cam = ViewportCamera3D { position: Vec3::ZERO, forward: Vec3::new(0.0,1.0,0.0),
            up: Vec3::new(0.0,1.0,0.0), width_px: 800.0, height_px: 600.0,
            projection: Projection3D::Perspective { vertical_fov_rad: 1.0 } };
        assert!(cam.ray_from_pixel(Vec2::ZERO).is_none());
        let view = Viewport2D { width_px: 0.0, height_px: 20.0,
            drawing_center: Vec2::ZERO, drawing_units_per_pixel: 1.0 };
        assert!(view.screen_to_world(Vec2::ZERO).is_none());
    }
}

#[cfg(test)]
mod r57_camera_clip_tests {
    use super::*;
    use crate::PickTriangle3D;
    #[test]
    fn perspective_camera_respects_near_clip_and_far_clip() {
        let face = |z, id| PickTriangle3D { entity_id: id, vertices: [
            Vec3::new(-1.0,-1.0,z), Vec3::new(1.0,-1.0,z), Vec3::new(0.0,1.0,z)] };
        let index = FaceIndex3D::new(&[face(4.0,1),face(1.0,2)]);
        let camera = ViewportCamera3D { position: Vec3::new(0.0,0.0,5.0),
            forward: Vec3::new(0.0,0.0,-1.0), up: Vec3::new(0.0,1.0,0.0),
            width_px: 800.0, height_px: 600.0,
            projection: Projection3D::Perspective { vertical_fov_rad: 1.0 } };
        let pixel = Vec2::new(400.0,300.0);
        assert_eq!(camera.pick_face_clipped(&index,pixel,2.0,5.0, |_| true)
            .map(|h| h.entity_id),Some(2));
        assert!(camera.pick_face_clipped(&index,pixel,2.0,3.0, |_| true).is_none());
    }
}

#[cfg(test)]
mod r58_camera_depth_tests {
    use super::*;
    use crate::PickTriangle3D;

    #[test]
    fn off_axis_perspective_clip_uses_view_depth_not_ray_length() {
        // At an off-centre pixel, z=2.5 is inside camera view-depth [2,3].
        // A naïve ray-distance clip may incorrectly discard this valid face.
        let camera = ViewportCamera3D { position: Vec3::new(0.0,0.0,5.0),
            forward: Vec3::new(0.0,0.0,-1.0), up: Vec3::new(0.0,1.0,0.0),
            width_px: 800.0, height_px: 600.0,
            projection: Projection3D::Perspective { vertical_fov_rad: 1.5 } };
        let pixel = Vec2::new(760.0, 300.0);
        let ray = camera.ray_from_pixel(pixel).unwrap();
        let t = (5.0 - 2.5) / -ray.direction.z;
        assert!(t > 3.0, "oblique ray intersection must exceed far ray-distance");
        let center = ray.origin + ray.direction * t;
        let triangle = PickTriangle3D { entity_id: 901, vertices: [
            center + Vec3::new(-1.0,-1.0,0.0),
            center + Vec3::new(1.0,-1.0,0.0),
            center + Vec3::new(0.0,1.0,0.0)] };
        let index = FaceIndex3D::new(&[triangle]);
        assert_eq!(camera.pick_face_clipped(&index, pixel, 2.0, 3.0, |_| true)
            .map(|h| h.entity_id), Some(901));
        assert!(camera.pick_face_clipped(&index, pixel, 0.0, 2.0, |_| true).is_none());
        assert!(camera.pick_face_clipped(&index, pixel, 3.0, 4.0, |_| true).is_none());
        assert!(camera.pick_face_clipped(&index, pixel, 2.0, 3.0, |_| false).is_none());
    }

    #[test]
    fn invalid_depth_ranges_do_not_pick() {
        let camera = ViewportCamera3D { position: Vec3::ZERO,
            forward: Vec3::new(0.0,0.0,-1.0), up: Vec3::new(0.0,1.0,0.0),
            width_px: 800.0, height_px: 600.0,
            projection: Projection3D::Perspective { vertical_fov_rad: 1.0 } };
        let index = FaceIndex3D::default();
        let p = Vec2::new(400.0,300.0);
        for (near, far) in [(f64::NAN,4.0),(-1.0,4.0),(4.0,3.0),
            (1.0,f64::NAN),(f64::INFINITY, f64::INFINITY)] {
            assert!(camera.pick_face_clipped(&index,p,near,far, |_| true).is_none());
        }
    }
}

#[cfg(test)]
mod r60_viewport_intersection_tests {
    use super::*;
    #[test]
    fn pixel_intersection_snap_filters_and_rejects_off_canvas() {
        let view = Viewport2D { width_px: 800.0, height_px: 600.0,
            drawing_center: Vec2::ZERO, drawing_units_per_pixel: 0.1 };
        let index = SegmentIndex2D::new(&[
            (Vec2::new(-3.0, 0.0), Vec2::new(3.0, 0.0)),
            (Vec2::new(0.0, -3.0), Vec2::new(0.0, 3.0)),
        ]);
        let hit = view.snap_intersection(&index, Vec2::new(401.0, 301.0), 3.0,
            |_| true).unwrap();
        assert_eq!((hit.first_index, hit.second_index), (0, 1));
        assert!(view.snap_intersection(&index, Vec2::new(401.0, 301.0), 3.0,
            |i| i == 0).is_none());
        assert!(view.snap_intersection(&index, Vec2::new(-1.0, 300.0), 500.0,
            |_| true).is_none());
    }
}
