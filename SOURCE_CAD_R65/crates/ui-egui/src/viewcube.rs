//! The navigation overlays on the drawing area: the ViewCube (Top face, Home, compass ring and
//! the WCS menu) and the UCS icon.
//!
//! Every overlay registers a widget that senses clicks *and* drags, so egui hands the pointer to
//! the overlay instead of the canvas underneath (a click on the ViewCube must never also pick or
//! start a selection window in the drawing). What the overlays do goes through commands:
//! `zoom.extents` (Top face, Home), `ui.toggle.viewcube`, `ui.toggle.ucsicon`.
//!
//! The renderer keeps 2D drafting entities in exact XY while projecting genuine 3D entities with
//! the drawing's VIEWYAW/VIEWPITCH. Named UCS definitions are exposed through the same engine
//! command used by scripts, and the compass ring drives 3DORBIT directly.

use egui::{Color32, CursorIcon, Pos2, Rect, Sense, Stroke, pos2, vec2};
use serde_json::json;

use crate::CadApp;
use crate::theme::Tokens;

/// Shown when the compass ring is clicked or dragged.
pub const NO_TWIST: &str = "ViewCube orbit changes the orthographic projection of 3D entities; 2D drafting entities remain in their exact XY plane.";

/// Where the ViewCube's parts are on the canvas.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub center: Pos2,
    /// Radius of the compass ring's centre line.
    pub ring: f32,
    pub face: Rect,
    pub home: Rect,
    /// The current coordinate-system menu under the cube.
    pub pill: Rect,
    /// Everything that belongs to the cube (the area kept away from the canvas).
    pub bounds: Rect,
}

impl Layout {
    pub fn new(canvas: Rect) -> Self {
        let center = pos2(canvas.right() - 92.0, canvas.top() + 82.0);
        let ring = 58.0;
        Layout {
            center,
            ring,
            face: Rect::from_center_size(center, vec2(44.0, 44.0)),
            home: Rect::from_center_size(center + vec2(-ring + 2.0, -ring + 2.0), vec2(20.0, 20.0)),
            pill: Rect::from_center_size(center + vec2(0.0, ring + 26.0), vec2(56.0, 16.0)),
            bounds: Rect::from_center_size(center, vec2(2.0 * (ring + 10.0), 2.0 * (ring + 10.0))),
        }
    }

    /// The part of the cube at `p` (not the pill, which is its own widget).
    pub fn part_at(&self, p: Pos2) -> Option<Part> {
        if self.home.contains(p) {
            return Some(Part::Home);
        }
        if self.face.contains(p) {
            return Some(Part::Face);
        }
        let d = p.distance(self.center);
        ((self.ring - 10.0..=self.ring + 10.0).contains(&d)).then_some(Part::Ring)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Face,
    Home,
    Ring,
}

impl Part {
    fn tip(self) -> &'static str {
        match self {
            Part::Face => "Top: reset 3D projection to plan view",
            Part::Home => "Home: SE isometric 3D projection + zoom extents",
            Part::Ring => "Compass: drag to orbit 3D entities; click for SE isometric",
        }
    }
}

/// Paint the ViewCube's ring, compass letters, Top face and Home button, and handle the pointer.
/// The canvas paints the UCS pill itself; this returns the pointer's part for its highlight.
pub fn show(app: &mut CadApp, ui: &mut egui::Ui, l: &Layout) {
    let resp = ui.interact(l.bounds, ui.scope_id().with("viewcube"), Sense::click_and_drag());
    let part = resp.hover_pos().and_then(|p| l.part_at(p));
    let p = ui.painter_at(l.bounds.union(l.pill));
    let ring_col = if part == Some(Part::Ring) { Color32::from_rgb(0x5c, 0x67, 0x7c) } else { Color32::from_rgb(0x48, 0x50, 0x5c) };
    p.circle_stroke(l.center, l.ring, Stroke::new(9.0, ring_col));
    p.circle_stroke(l.center, l.ring + 4.5, Stroke::new(1.0, Color32::from_rgb(0x5c, 0x65, 0x72)));
    let f = egui::FontId::proportional(17.0);
    let lc = Color32::from_rgb(0xc8, 0xcc, 0xd2);
    let c = l.center;
    let ring = l.ring;
    p.text(c + vec2(0.0, -ring - 1.0), egui::Align2::CENTER_CENTER, crate::tl!("N"), f.clone(), lc);
    p.text(c + vec2(0.0, ring + 1.0), egui::Align2::CENTER_CENTER, crate::tl!("S"), f.clone(), lc);
    p.text(c + vec2(ring + 1.0, 0.0), egui::Align2::CENTER_CENTER, crate::tl!("E"), f.clone(), lc);
    p.text(c + vec2(-ring - 1.0, 0.0), egui::Align2::CENTER_CENTER, crate::tl!("W"), f, lc);
    let face_col = if part == Some(Part::Face) { Color32::from_rgb(0xb8, 0xbc, 0xc2) } else { Color32::from_rgb(0x9a, 0x9e, 0xa4) };
    p.rect_filled(l.face, 2.0, face_col);
    p.rect_stroke(l.face, 2.0, Stroke::new(1.0, Color32::from_rgb(0x6c, 0x70, 0x76)), egui::StrokeKind::Inside);
    p.text(c, egui::Align2::CENTER_CENTER, crate::tl!("TOP"), egui::FontId::proportional(13.0), Color32::from_rgb(0x50, 0x54, 0x5a));
    paint_home(&p, l.home, part == Some(Part::Home));

    let resp = match part {
        Some(part) => resp.on_hover_cursor(CursorIcon::PointingHand).on_hover_text(part.tip()),
        None => resp,
    };
    let pressed_on = resp.interact_pointer_pos().and_then(|o| l.part_at(o));
    if resp.clicked() {
        match pressed_on {
            Some(Part::Face) => {
                let _ = app.run("3dorbit", json!({"preset":"top"}));
                let _ = app.run("zoom.extents", json!({}));
            }
            Some(Part::Home) => {
                let _ = app.run("3dorbit", json!({"preset":"seiso"}));
                let _ = app.run("zoom.extents", json!({}));
            }
            Some(Part::Ring) => { let _ = app.run("3dorbit", json!({"preset":"seiso"})); }
            None => {}
        }
    }
    if resp.dragged() && pressed_on == Some(Part::Ring) {
        let delta = ui.input(|i| i.pointer.delta());
        if delta.length_sq() > 0.0 {
            let _ = app.run("3dorbit", json!({"deltaYaw": f64::from(delta.x) * 0.45, "deltaPitch": f64::from(-delta.y) * 0.35}));
        }
    }
    let mut action = None;
    resp.context_menu(|ui| {
        if ui.button("Top").clicked() { action = Some(("3dorbit", json!({"preset":"top"}))); ui.close(); }
        if ui.button("SE Isometric").clicked() { action = Some(("3dorbit", json!({"preset":"seiso"}))); ui.close(); }
        if ui.button("SW Isometric").clicked() { action = Some(("3dorbit", json!({"preset":"swiso"}))); ui.close(); }
        if ui.button("NE Isometric").clicked() { action = Some(("3dorbit", json!({"preset":"neiso"}))); ui.close(); }
        if ui.button("NW Isometric").clicked() { action = Some(("3dorbit", json!({"preset":"nwiso"}))); ui.close(); }
        if ui.button("Zoom Extents").clicked() { action = Some(("zoom.extents", json!({}))); ui.close(); }
        if ui.button("Hide ViewCube").clicked() {
            action = Some(("ui.toggle.viewcube", json!({"on": false})));
            ui.close();
        }
    });
    if let Some((id, params)) = action {
        let _ = app.run(id, params);
    }
    // The UCS pill (painted by the canvas) opens the coordinate-system menu.
    let pill = ui.interact(l.pill, ui.scope_id().with("viewcube_wcs"), Sense::click_and_drag()).on_hover_cursor(CursorIcon::PointingHand);
    let current = app.session.doc().ok().map(|d| d.header.str("UCSNAME", "World")).unwrap_or_else(|| "World".into());
    let pill = pill.on_hover_text(format!("Coordinate system: {current}"));
    ucs_menu(app, egui::Popup::menu(&pill));
}

/// A small house, drawn in code.
fn paint_home(p: &egui::Painter, r: Rect, hot: bool) {
    let t = Tokens::get();
    if hot {
        p.rect_filled(r, 3.0, t.control_hover);
    }
    let st = Stroke::new(1.3, if hot { t.text } else { t.text_dim });
    let c = r.center();
    let roof = [c + vec2(-6.5, 0.5), c + vec2(0.0, -6.0), c + vec2(6.5, 0.5)];
    p.line(roof.to_vec(), st);
    p.line(vec![c + vec2(-4.5, -1.0), c + vec2(-4.5, 6.0), c + vec2(4.5, 6.0), c + vec2(4.5, -1.0)], st);
    p.line_segment([c + vec2(-1.2, 6.0), c + vec2(-1.2, 2.5)], st);
    p.line_segment([c + vec2(1.2, 6.0), c + vec2(1.2, 2.5)], st);
    p.line_segment([c + vec2(-1.2, 2.5), c + vec2(1.2, 2.5)], st);
}

/// The UCS icon's arms (as drawn by the canvas at the lower-left corner): the X arm and the Y arm.
pub fn ucs_arms(canvas: Rect) -> [Rect; 2] {
    let o = pos2(canvas.left() + 36.0, canvas.bottom() - 34.0);
    [Rect::from_min_max(o + vec2(-8.0, -8.0), o + vec2(80.0, 8.0)), Rect::from_min_max(o + vec2(-8.0, -82.0), o + vec2(8.0, 8.0))]
}

/// Make the UCS icon clickable: hovering highlights it, a click (either button) opens the
/// coordinate-system menu.
pub fn ucs_icon(app: &mut CadApp, ui: &mut egui::Ui, canvas: Rect) {
    let t = Tokens::get();
    let [x, y] = ucs_arms(canvas);
    let rx = ui.interact(x, ui.scope_id().with("ucs_x"), Sense::click_and_drag());
    let ry = ui.interact(y, ui.scope_id().with("ucs_y"), Sense::click_and_drag());
    let resp = (rx | ry).on_hover_cursor(CursorIcon::PointingHand).on_hover_text("UCS icon: World Coordinate System");
    if resp.hovered() {
        let o = pos2(x.left() + 8.0, x.center().y);
        let st = Stroke::new(2.0, t.hover);
        let p = ui.painter_at(canvas);
        p.line_segment([o, o + vec2(60.0, 0.0)], st);
        p.line_segment([o, o + vec2(0.0, -60.0)], st);
    }
    // Opened with `Bool(true)`: a pointer-anchored popup records its position only then.
    let open = (resp.clicked() || resp.secondary_clicked()).then_some(egui::SetOpenCommand::Bool(true));
    ucs_menu(app, egui::Popup::menu(&resp).open_memory(open).at_pointer_fixed());
}

/// Coordinate-system menu backed by the drawing's named UCS table. Setting a UCS here changes
/// the canonical UCSNAME through the engine. 2D drafting geometry remains XY-exact while the
/// dedicated 3D entity projection uses VIEWYAW/VIEWPITCH.
fn ucs_menu(app: &mut CadApp, popup: egui::Popup<'_>) {
    let (current, mut names) = app
        .session
        .doc()
        .map(|d| {
            let current = d.header.str("UCSNAME", "World");
            let mut names: Vec<_> = d.ucss.iter().map(|u| u.name.clone()).collect();
            if !names.iter().any(|n| n.eq_ignore_ascii_case("World")) {
                names.insert(0, "World".into());
            }
            (current, names)
        })
        .unwrap_or_else(|_| ("World".into(), vec!["World".into()]));
    names.sort_by_key(|n| (!n.eq_ignore_ascii_case("World"), n.to_ascii_lowercase()));
    let mut action: Option<(&'static str, serde_json::Value)> = None;
    popup.show(|ui| {
        ui.strong("Coordinate System");
        for name in &names {
            if ui.radio(name.eq_ignore_ascii_case(&current), name).clicked() {
                action = Some(("ucs", if name.eq_ignore_ascii_case("World") { json!({"mode":"world"}) } else { json!({"mode":"set", "name":name}) }));
                ui.close();
            }
        }
        ui.separator();
        if ui.button("UCS Manager...").clicked() {
            action = Some(("ucs", serde_json::Value::Null));
            ui.close();
        }
        if ui.button("Hide UCS Icon").clicked() {
            action = Some(("ui.toggle.ucsicon", json!({"on": false})));
            ui.close();
        }
    });
    if let Some((cmd, params)) = action {
        if params.is_null() {
            app.start(cmd);
        } else {
            let _ = app.run(cmd, params);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadcraft_engine::Session;

    /// The app with the sample drawing, laid out once in a 1400×900 window.
    fn app_and_ctx() -> (CadApp, egui::Context) {
        let mut app = CadApp::new(Session::empty(), crate::Services::default());
        app.ui.in_window_menu = true;
        app.start("ui.sample");
        let ctx = egui::Context::default();
        frame(&mut app, &ctx, Vec::new());
        frame(&mut app, &ctx, Vec::new());
        (app, ctx)
    }

    fn frame(app: &mut CadApp, ctx: &egui::Context, events: Vec<egui::Event>) {
        let raw = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1400.0, 900.0))), events, ..Default::default() };
        let mut out = ctx.run_ui(raw, |ui| {
            app.logic(ui.ctx());
            app.ui(ui);
        });
        out.textures_delta.clear();
    }

    /// A real pointer click (move, press, release in separate frames, like the control channel).
    pub(crate) fn click(app: &mut CadApp, ctx: &egui::Context, at: Pos2, button: egui::PointerButton) {
        let m = egui::Modifiers::default();
        frame(app, ctx, vec![egui::Event::PointerMoved(at)]);
        frame(app, ctx, vec![egui::Event::PointerButton { pos: at, button, pressed: true, modifiers: m }]);
        frame(app, ctx, vec![egui::Event::PointerButton { pos: at, button, pressed: false, modifiers: m }]);
        frame(app, ctx, Vec::new());
    }

    /// A menu popup was shown in the last frame.
    fn menu_shown(ctx: &egui::Context) -> bool {
        ctx.memory(|m| m.areas().visible_layer_ids().iter().any(|l| l.order == egui::Order::Foreground))
    }

    fn canvas(app: &CadApp) -> Rect {
        app.canvas.rect.unwrap_or(Rect::NOTHING)
    }

    fn view(app: &CadApp) -> (f64, f64, f64) {
        let v = app.session.state().map(|s| s.view()).unwrap_or_default();
        (v.center.x, v.center.y, v.height)
    }

    #[test]
    fn parts_are_found_where_they_are_drawn() {
        let l = Layout::new(Rect::from_min_size(pos2(200.0, 80.0), vec2(1000.0, 800.0)));
        assert_eq!(l.part_at(l.center), Some(Part::Face));
        assert_eq!(l.part_at(l.home.center()), Some(Part::Home));
        assert_eq!(l.part_at(l.center + vec2(0.0, -l.ring)), Some(Part::Ring), "N");
        assert_eq!(l.part_at(l.center + vec2(l.ring + 3.0, 0.0)), Some(Part::Ring), "E");
        assert_eq!(l.part_at(l.center + vec2(30.0, 30.0)), None, "between the face and the ring");
        assert!(l.bounds.contains_rect(l.home) && l.bounds.contains_rect(l.face));
    }

    #[test]
    fn viewcube_clicks_reach_the_cube_not_the_drawing() {
        let (mut app, ctx) = app_and_ctx();
        let l = Layout::new(canvas(&app));
        let fitted = view(&app);
        // Zoomed in, the Top face fits the drawing again (zoom extents) and nothing is picked.
        let _ = app.run("zoom.in", json!({}));
        assert_ne!(view(&app), fitted);
        click(&mut app, &ctx, l.center, egui::PointerButton::Primary);
        assert_eq!(view(&app), fitted, "Top face zooms to extents");
        assert!(app.session.pending_window.is_none(), "the click didn't start a selection window");
        assert!(app.session.selection().is_empty());
        // Home does the same.
        let _ = app.run("zoom.in", json!({}));
        click(&mut app, &ctx, l.home.center(), egui::PointerButton::Primary);
        assert_eq!(view(&app), fitted, "Home zooms to extents");
        // The ring explains that the 2D view can't rotate, and doesn't touch the drawing.
        click(&mut app, &ctx, l.center + vec2(0.0, -l.ring), egui::PointerButton::Primary);
        assert_eq!(app.session.log.last().map(String::as_str), Some(NO_TWIST));
        assert!(app.session.pending_window.is_none());
        assert_eq!(view(&app), fitted);
        // A click in the drawing still reaches the drawing.
        let empty = canvas(&app).center() + vec2(0.0, -300.0);
        click(&mut app, &ctx, empty, egui::PointerButton::Primary);
        assert!(app.session.pending_window.is_some(), "an empty-space click starts a selection window");
    }

    #[test]
    fn ucs_icon_and_wcs_pill_open_the_coordinate_system_menu() {
        let (mut app, ctx) = app_and_ctx();
        let [x, _] = ucs_arms(canvas(&app));
        click(&mut app, &ctx, x.center(), egui::PointerButton::Primary);
        assert!(menu_shown(&ctx), "UCS icon menu");
        assert!(app.session.pending_window.is_none(), "the click stayed on the icon");
        let l = Layout::new(canvas(&app));
        click(&mut app, &ctx, l.pill.center(), egui::PointerButton::Primary);
        assert!(menu_shown(&ctx), "WCS menu");
        assert!(app.session.pending_window.is_none());
    }
}
