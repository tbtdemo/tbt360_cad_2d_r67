//! Associative-array parameter grips on the canvas.
//!
//! The engine owns all array metadata and mutation (`ARRAYGRIPS`/`ARRAYGRIPSET`).  This module is
//! only the egui adapter: selected associative arrays expose parameter grips directly on the
//! canvas and a compact editor applies the new value through `ARRAYGRIPSET`, preserving the same
//! validation, rebuild and undo path as ARRAYEDIT.

use std::sync::Arc;

use cadcraft_engine::doc::{ArrayKind, EntityKind, Handle};
use cadcraft_geom::Vec2;
use egui::{Color32, Pos2, Rect, Stroke, vec2};
use serde_json::json;

use crate::CadApp;
use crate::canvas::Xf;
use crate::theme::Tokens;

const MAX_ARRAYS: usize = 200;

#[derive(Clone, Debug)]
pub(crate) struct ArrayGrip {
    pub handle: Handle,
    pub parameter: &'static str,
    pub at: Vec2,
    pub value: f64,
    pub integer: bool,
}

#[derive(Clone, Default)]
pub(crate) struct Cache {
    key: (u64, u64, Vec<Handle>),
    grips: Arc<Vec<ArrayGrip>>,
}

#[derive(Clone, Debug)]
pub(crate) struct Editor {
    pub grip: ArrayGrip,
    pub text: String,
}

fn world(ins: &cadcraft_engine::doc::Insert, q: Vec2) -> Vec2 {
    ins.transform(Vec2::ZERO).apply(q)
}

/// Array grips for the current selection, cached per drawing revision and selection.
pub(crate) fn of_selection(app: &mut CadApp) -> Arc<Vec<ArrayGrip>> {
    let sel = app.session.selection();
    let Ok(st) = app.session.state() else { return Arc::default() };
    let key = (st.uid, st.revision, sel.clone());
    if app.canvas.array_grips.key != key {
        let mut out = Vec::new();
        for h in sel.iter().take(MAX_ARRAYS) {
            let Some(e) = st.doc.entity(*h) else { continue };
            if st.doc.layer(&e.common.layer).is_some_and(|l| l.locked) {
                continue;
            }
            let EntityKind::Insert(ins) = &e.kind else { continue };
            let Some(a) = st.doc.array_params(e) else { continue };
            if !a.is_valid() {
                continue;
            }
            match &a.kind {
                ArrayKind::Rect(r) => {
                    let ux = Vec2::from_angle(r.axis_angle);
                    let uy = ux.perp();
                    if r.cols > 1 {
                        out.push(ArrayGrip {
                            handle: *h,
                            parameter: "cols",
                            at: world(ins, ux * (r.col_spacing * f64::from(r.cols - 1))),
                            value: f64::from(r.cols),
                            integer: true,
                        });
                    }
                    if r.rows > 1 {
                        out.push(ArrayGrip {
                            handle: *h,
                            parameter: "rows",
                            at: world(ins, uy * (r.row_spacing * f64::from(r.rows - 1))),
                            value: f64::from(r.rows),
                            integer: true,
                        });
                    }
                }
                ArrayKind::Polar(p) => {
                    let radius = Vec2::ZERO - p.center;
                    let endpoint = p.center + radius.rotate(p.fill.to_radians());
                    if p.items > 1 {
                        out.push(ArrayGrip {
                            handle: *h,
                            parameter: "items",
                            at: world(ins, endpoint),
                            value: f64::from(p.items),
                            integer: true,
                        });
                        out.push(ArrayGrip {
                            handle: *h,
                            parameter: "fill",
                            at: world(ins, endpoint + radius.normalized() * 0.001),
                            value: p.fill,
                            integer: false,
                        });
                    }
                }
                ArrayKind::Path(_) => {}
            }
        }
        app.canvas.array_grips = Cache { key, grips: Arc::new(out) };
    }
    app.canvas.array_grips.grips.clone()
}

fn grip_rect(xf: &Xf, g: &ArrayGrip, size: f32) -> Rect {
    Rect::from_center_size(xf.to_screen(g.at), vec2(size * 2.2, size * 2.2))
}

pub(crate) fn at(app: &mut CadApp, xf: &Xf, hover: Option<Pos2>) -> Option<ArrayGrip> {
    let p = hover?;
    let gs = app.session.settings.gripsize as f32;
    of_selection(app).iter().find(|g| grip_rect(xf, g, gs).expand(3.0).contains(p)).cloned()
}


/// Ctrl+click a visible item of the single selected associative array to suppress that item.
/// Returns true when the click belonged to the selected array, so the canvas does not also run
/// normal object-selection logic for the same click.
pub(crate) fn ctrl_remove_item(app: &mut CadApp, at: Vec2, scale: f64) -> bool {
    let hover = app.canvas.hover;
    let tolerance = app.session.settings.pickbox.max(1.0) * 1.5 / scale.max(1e-12);
    let sel = app.session.selection();
    let [h] = sel.as_slice() else { return false };
    if hover != Some(*h) {
        return false;
    }
    let is_array = app
        .session
        .state()
        .ok()
        .and_then(|st| st.doc.entity(*h).and_then(|e| st.doc.array_params(e)))
        .is_some();
    if !is_array {
        return false;
    }
    if let Err(e) = app.run(
        "arrayitemremove",
        json!({ "handle": h.hex(), "point": [at.x, at.y], "tolerance": tolerance.max(0.0) }),
    ) {
        app.session.echo(e);
    }
    true
}

pub(crate) fn click(app: &mut CadApp, g: ArrayGrip) {
    let text = if g.integer { format!("{:.0}", g.value) } else { format!("{:.3}", g.value) };
    app.canvas.array_grip_editor = Some(Editor { grip: g, text });
}

pub(crate) fn draw(app: &mut CadApp, painter: &egui::Painter, xf: &Xf, rect: Rect, hover: Option<Pos2>) {
    let t = Tokens::get();
    let gs = app.session.settings.gripsize as f32;
    for g in of_selection(app).iter() {
        let r = grip_rect(xf, g, gs);
        if !rect.intersects(r) {
            continue;
        }
        let warm = hover.is_some_and(|p| r.expand(3.0).contains(p));
        let active = app
            .canvas
            .array_grip_editor
            .as_ref()
            .is_some_and(|e| e.grip.handle == g.handle && e.grip.parameter == g.parameter);
        let fill = if active {
            t.grip_hot
        } else if warm {
            Color32::from_rgb(0xff, 0x7f, 0x9f)
        } else {
            t.grip_dyn
        };
        painter.rect_filled(r, 2.0, fill);
        painter.rect_stroke(r, 2.0, Stroke::new(1.0, Color32::from_rgb(0x10, 0x30, 0x80)), egui::StrokeKind::Middle);
        let label = match g.parameter {
            "cols" => "C",
            "rows" => "R",
            "items" => "#",
            "fill" => "°",
            _ => "A",
        };
        painter.text(r.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::monospace(gs.max(8.0)), Color32::BLACK);
    }
}

/// Compact numeric editor opened from an array parameter grip.
pub(crate) fn menu(app: &mut CadApp, ctx: &egui::Context, xf: &Xf) {
    let Some(mut ed) = app.canvas.array_grip_editor.clone() else { return };
    let pos = xf.to_screen(ed.grip.at) + vec2(10.0, 10.0);
    let mut apply = false;
    let mut cancel = false;
    egui::Area::new(egui::Id::new(("array-grip-editor", ed.grip.handle, ed.grip.parameter)))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.label(format!("Array {}", ed.grip.parameter));
                let response = ui.add(egui::TextEdit::singleline(&mut ed.text).desired_width(90.0));
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    apply = true;
                }
                ui.horizontal(|ui| {
                    if ui.small_button("Apply").clicked() {
                        apply = true;
                    }
                    if ui.small_button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        cancel = true;
    }
    if cancel {
        app.canvas.array_grip_editor = None;
        return;
    }
    if apply {
        let parsed = if ed.grip.integer {
            ed.text.trim().parse::<u32>().ok().filter(|v| *v >= 1).map(|v| json!(v))
        } else {
            ed.text.trim().parse::<f64>().ok().filter(|v| v.is_finite()).map(|v| json!(v))
        };
        match parsed {
            Some(value) => {
                if let Err(e) = app.run(
                    "arraygripset",
                    json!({ "handle": ed.grip.handle.hex(), "parameter": ed.grip.parameter, "value": value }),
                ) {
                    app.session.echo(e);
                }
                app.canvas.array_grip_editor = None;
            }
            None => {
                app.session.echo("Array grip value is invalid.");
                app.canvas.array_grip_editor = Some(ed);
            }
        }
    } else {
        app.canvas.array_grip_editor = Some(ed);
    }
}
