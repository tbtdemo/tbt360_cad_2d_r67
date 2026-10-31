//! Style and settings dialogs opened from the menus: Linetype Manager, Point Style, Text Style,
//! Table Style, Multileader Style, Page Setup Manager, Plot and Constraint Settings.
//!
//! Typed or chosen from a menu (no parameters) these commands open their dialog; JSON calls run
//! the command. The dialogs read the drawing and change it only through the same commands
//! (`app.run`), so every change is undoable and scriptable.

use std::sync::atomic::{AtomicU32, Ordering};

use egui::{RichText, Sense, Stroke, vec2};
use serde_json::{Value, json};

use crate::CadApp;
use crate::parametric::expr_field;
use crate::theme::Tokens;

/// The Block Attribute Manager (BATTMAN).
mod battman;

/// Open the dialog for a typed or menu-invoked command. `None` if `id` has no dialog here or the
/// call carries parameters.
pub fn route(app: &mut CadApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    if id == "ui.tolerance.place" {
        let characteristic = params.get("characteristic").and_then(Value::as_str).unwrap_or("Position");
        let tol1 = params.get("tol1").and_then(Value::as_str).unwrap_or("");
        let tol2 = params.get("tol2").and_then(Value::as_str).unwrap_or("");
        let datums = params.get("datums").and_then(Value::as_array).cloned().unwrap_or_default();
        if let Err(e) = app.session.start("tolerance") {
            return Some(Err(e.to_string()));
        }
        for line in [characteristic, tol1, tol2] {
            if let Err(e) = app.session.cmdline(line) {
                return Some(Err(e.to_string()));
            }
        }
        for i in 0..3 {
            let line = datums.get(i).and_then(Value::as_str).unwrap_or("");
            if let Err(e) = app.session.cmdline(line) {
                return Some(Err(e.to_string()));
            }
            if line.trim().is_empty() {
                break;
            }
        }
        app.ui.dialog = None;
        return Some(Ok(Value::Null));
    }
    if !params.is_null() {
        return None;
    }
    if matches!(id, "blocks.dynamic" | "blocks.dynamic.validate" | "blocks.reference" | "blocks.reference.grips" | "blocks.reference.set" | "blocks.reference.grip") {
        app.ui.dialog = Some("dynamicblock".into());
        return Some(Ok(Value::Null));
    }
    if matches!(id, "box" | "pyramid" | "cylinder" | "cone" | "sphere" | "torus") {
        app.ui.dialog = Some(format!("primitive3d:{id}"));
        return Some(Ok(Value::Null));
    }
    if matches!(id, "extrude" | "revolve" | "sweep" | "loft") {
        app.ui.dialog = Some(format!("solid3d:{id}"));
        return Some(Ok(Value::Null));
    }
    if id == "sectionplane" || id == "section" {
        app.ui.dialog = Some("sectionplane".into());
        return Some(Ok(Value::Null));
    }
    if id == "pointcloud" || id == "pointcloud.style" {
        app.ui.dialog = Some("pointcloud".into());
        return Some(Ok(Value::Null));
    }
    if matches!(id, "attach" | "pdfattach" | "imageattach") {
        app.ui.dialog = Some("underlay".into());
        return Some(Ok(Value::Null));
    }
    if matches!(id, "imageadjust" | "imagetransform") {
        app.ui.dialog = Some("imageedit".into());
        return Some(Ok(Value::Null));
    }
    if matches!(id, "facettransform3d" | "3dtransform" | "facettransform" | "facetmirror3d" | "3dmirror" | "facetmirror" | "facetoffset3d" | "3doffsetfacet" | "facetoffset") {
        app.ui.dialog = Some("facettransform3d".into());
        return Some(Ok(Value::Null));
    }
    if id == "text3d" || id == "3dtext" {
        app.ui.dialog = Some("text3d".into());
        return Some(Ok(Value::Null));
    }
    if id == "dim3d" || id == "3ddim" || id == "dimension3d" {
        app.ui.dialog = Some("dim3d".into());
        return Some(Ok(Value::Null));
    }
    if id == "leader3d" || id == "3dleader" { app.ui.dialog = Some("leader3d".into()); return Some(Ok(Value::Null)); }
    if id == "dim3d.angle" || id == "3dangle" { app.ui.dialog = Some("dim3d_angle".into()); return Some(Ok(Value::Null)); }
    if id == "dim3d.radius" || id == "3dradius" { app.ui.dialog = Some("dim3d_radius".into()); return Some(Ok(Value::Null)); }
    if id == "3dorbit" || id == "orbit" || id == "3do" {
        app.ui.dialog = Some("view3d".into());
        return Some(Ok(Value::Null));
    }
    let dialog = match id {
        "battman" => "battman",
        "linetype" | "lt" | "ltype" => "linetype",
        "ddptype" | "ptype" => "ptype",
        "dimstyle" | "d" => "dimstyle",
        "tolerance" | "tol" => "tolerance",
        "ucs" | "uc" => "ucs",
        "visualstyle" | "vscurrent" | "vstyle" => "visualstyle",
        "material" | "materials" => "material",
        "light" | "lights" => "light",
        "render" => "render",
        "sectionplane" | "section" => "sectionplane",
        "pointcloud" => "pointcloud",
        "facettransform3d" | "3dtransform" | "facettransform" | "facetmirror3d" | "3dmirror" | "facetmirror" | "facetoffset3d" | "3doffsetfacet" | "facetoffset" => "facettransform3d",
        "text3d" | "3dtext" => "text3d",
        "dim3d" | "3ddim" | "dimension3d" => "dim3d",
        "leader3d" | "3dleader" => "leader3d",
        "dim3d.angle" | "3dangle" => "dim3d_angle",
        "dim3d.radius" | "3dradius" => "dim3d_radius",
        "3dorbit" | "orbit" | "3do" => "view3d",
        "style" | "st" => "textstyle",
        "tablestyle" | "ts" => "tablestyle",
        "mleaderstyle" | "mls" => "mleaderstyle",
        "pagesetup" => "pagesetup",
        "constraintsettings" | "csettings" => "csettings",
        "plot" | "print" => {
            PLOT_GEN.fetch_add(1, Ordering::Relaxed);
            "plot"
        }
        _ => return None,
    };
    app.ui.dialog = Some(dialog.into());
    Some(Ok(Value::Null))
}

/// Show the dialog named `name`; closes it (`open = false`) if it isn't one of these.
pub fn dialog(app: &mut CadApp, ctx: &egui::Context, name: &str, open: &mut bool) {
    if let Some(kind) = name.strip_prefix("primitive3d:") {
        if let Some((cmd, p)) = primitive_3d(app, ctx, open, kind) {
            let _ = app.run(cmd, p);
        }
        return;
    }
    if let Some(kind) = name.strip_prefix("solid3d:") {
        if let Some((cmd, p)) = solid_3d(app, ctx, open, kind) {
            let _ = app.run(cmd, p);
        }
        return;
    }
    let action = match name {
        "3dcenter" => control_center_3d(app, ctx, open),
        "dynamicblock" => dynamic_block_dialog(ctx, open),
        "battman" => battman::dialog(app, ctx, open),
        "linetype" => linetype(app, ctx, open),
        "ptype" => point_style(app, ctx, open),
        "dimstyle" => dimension_style(app, ctx, open),
        "tolerance" => tolerance_dialog(app, ctx, open),
        "ucs" => ucs_manager(app, ctx, open),
        "visualstyle" => visual_style(app, ctx, open),
        "material" => material_dialog(app, ctx, open),
        "light" => light_dialog(app, ctx, open),
        "render" => render_dialog(app, ctx, open),
        "sectionplane" => section_plane(app, ctx, open),
        "pointcloud" => point_cloud_dialog(app, ctx, open),
        "underlay" => underlay_attach_dialog(ctx, open),
        "imageedit" => image_edit_dialog(ctx, open),
        "facettransform3d" => facet_transform_dialog(ctx, open),
        "text3d" => text3d_dialog(app, ctx, open),
        "dim3d" => dim3d_dialog(app, ctx, open),
        "leader3d" => leader3d_dialog(app, ctx, open),
        "dim3d_angle" => dim3d_angle_dialog(app, ctx, open),
        "dim3d_radius" => dim3d_radius_dialog(app, ctx, open),
        "view3d" => view3d_dialog(app, ctx, open),
        "textstyle" => text_style(app, ctx, open),
        "tablestyle" => table_style(app, ctx, open),
        "mleaderstyle" => mleader_style(app, ctx, open),
        "pagesetup" => page_setup(app, ctx, open),
        "csettings" => constraint_settings(app, ctx, open),
        "plot" => plot(app, ctx, open),
        _ => {
            *open = false;
            None
        }
    };
    if let Some((cmd, p)) = action {
        // Errors are echoed on the command line by `run`.
        let _ = app.run(cmd, p);
    }
}

/// A command to run after the dialog is drawn.
type Action = Option<(&'static str, Value)>;

fn fmt_num(v: f64) -> String {
    let s = format!("{v:.4}");
    if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s }
}

/// A number field that commits on Enter or focus loss.
///
/// `label` is the visible label's widget id (see [`expr_field`]).
fn num_field(ui: &mut egui::Ui, id: impl egui::AsIdSalt, v: f64, label: Option<egui::Id>) -> Option<f64> {
    expr_field(ui, ui.make_persistent_id(id), &fmt_num(v), label).and_then(|t| t.parse::<f64>().ok()).filter(|v| v.is_finite())
}

fn temp<T: Clone + Send + Sync + 'static>(ctx: &egui::Context, id: &str) -> Option<T> {
    ctx.data_mut(|d| d.get_temp::<T>(egui::Id::unique(id)))
}

fn set_temp<T: Clone + Send + Sync + 'static>(ctx: &egui::Context, id: &str, v: T) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique(id), v));
}

fn no_drawing(ctx: &egui::Context, title: &str, open: &mut bool) {
    egui::Window::new(title).open(open).collapsible(false).resizable(false).show(ctx, |ui| {
        ui.label("Open a drawing first.");
    });
}




fn control_center_3d(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "3D Control Center", open);
        return None;
    };
    let visual = d.header.str("VISUALSTYLE", "2DWireframe").to_string();
    let ucs = d.header.str("UCSNAME", "World").to_string();
    let yaw = d.header.f64("VIEWYAW", 0.0);
    let pitch = d.header.f64("VIEWPITCH", 0.0);
    let projection = d.header.str("VIEWPROJ", "orthographic").to_string();
    let entities = d.model.iter().count();
    let selected = app.session.selection().len();
    let mut action = None;
    egui::Window::new("3D Control Center")
        .open(open)
        .default_width(620.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Visible 3D workflow");
            ui.label("Everything here routes to the same engine commands used by menus and Tool Sets.");
            ui.separator();
            egui::Grid::new("3d_center_status").num_columns(2).striped(true).show(ui, |ui| {
                ui.label("Visual style"); ui.strong(&visual); ui.end_row();
                ui.label("Projection"); ui.strong(&projection); ui.end_row();
                ui.label("View yaw / pitch"); ui.label(format!("{yaw:.1}° / {pitch:.1}°")); ui.end_row();
                ui.label("Current UCS"); ui.strong(&ucs); ui.end_row();
                ui.label("Entities / selected"); ui.label(format!("{entities} / {selected}")); ui.end_row();
            });
            ui.separator();
            ui.strong("Quick start");
            ui.horizontal_wrapped(|ui| {
                if ui.button("Create 3D Demo Scene").clicked() { action = Some(("ui.demo.3d", Value::Null)); }
                if ui.button("Architectural 3D Demo").clicked() { action = Some(("ui.demo.3d.arch", Value::Null)); }
                if ui.button("SE Isometric").clicked() { action = Some(("3dorbit", json!({"preset":"seiso","projection":"perspective","distance":80.0}))); }
                if ui.button("Top").clicked() { action = Some(("3dorbit", json!({"preset":"top","projection":"orthographic"}))); }
                if ui.button("Zoom Extents").clicked() { action = Some(("zoom.extents", json!({}))); }
            });
            ui.separator();
            ui.strong("Modeling");
            ui.horizontal_wrapped(|ui| {
                for (label, cmd) in [("Box","box"),("Cylinder","cylinder"),("Sphere","sphere"),("Torus","torus"),("Extrude","extrude"),("Revolve","revolve"),("Sweep","sweep"),("Loft","loft")] {
                    if ui.button(label).clicked() { app.start(cmd); }
                }
            });
            ui.separator();
            ui.strong("Inspect / view / reality");
            ui.horizontal_wrapped(|ui| {
                if ui.button("Visual Styles").clicked() { app.start("visualstyle"); }
                if ui.button("UCS Manager").clicked() { app.start("ucs"); }
                if ui.button("Orbit / Camera").clicked() { app.start("3dorbit"); }
                if ui.button("Section Plane").clicked() { app.start("sectionplane"); }
                if ui.button("Point Cloud").clicked() { app.start("pointcloud"); }
                if ui.button("Facet Statistics").clicked() { app.start("facetstats3d"); }
                if ui.button("Shell Check").clicked() { app.start("facetcheck3d"); }
            });
            ui.separator();
            ui.small("Current production representation is faceted DXF-compatible geometry. ACIS/B-Rep solid and surface kernels remain separate capabilities and are not implied by this panel.");
        });
    action
}

fn underlay_attach_dialog(ctx: &egui::Context, open: &mut bool) -> Action {
    let mut mode = temp::<String>(ctx, "underlay_mode").unwrap_or_else(|| "PDF".into());
    let mut path = temp::<String>(ctx, "underlay_path").unwrap_or_default();
    let mut page = temp::<u64>(ctx, "underlay_page").unwrap_or(1);
    let mut at = temp::<[f64; 2]>(ctx, "underlay_at").unwrap_or([0.0, 0.0]);
    let mut size = temp::<[f64; 2]>(ctx, "underlay_size").unwrap_or([100.0, 100.0]);
    let mut rotation = temp::<f64>(ctx, "underlay_rotation").unwrap_or(0.0);
    let mut action = None;
    egui::Window::new("Underlay / Raster Attach").open(open).default_width(520.0).resizable(false).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Type");
            egui::ComboBox::from_id_salt("underlay_type").selected_text(&mode).show_ui(ui, |ui| {
                for v in ["PDF", "Raster Image", "Auto by extension"] { ui.selectable_value(&mut mode, v.into(), v); }
            });
        });
        ui.horizontal(|ui| { ui.label("Path"); ui.text_edit_singleline(&mut path); });
        ui.small("No extra file-picker dependency is required. Paste or type the local file path. PDF/DWF/DGN/WMF/EPS currently show a placement/clip frame, not decoded page contents.");
        egui::Grid::new("underlay_attach_grid").num_columns(3).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
            ui.label("Insert X / Y"); ui.add(egui::DragValue::new(&mut at[0]).speed(0.5)); ui.add(egui::DragValue::new(&mut at[1]).speed(0.5)); ui.end_row();
            ui.label("Width / Height"); ui.add(egui::DragValue::new(&mut size[0]).speed(0.5).range(0.000001..=1.0e12)); ui.add(egui::DragValue::new(&mut size[1]).speed(0.5).range(0.000001..=1.0e12)); ui.end_row();
            ui.label("Rotation°"); ui.add(egui::DragValue::new(&mut rotation).speed(1.0)); ui.label(""); ui.end_row();
            if mode == "PDF" { ui.label("PDF page"); ui.add(egui::DragValue::new(&mut page).range(1..=100_000)); ui.label(""); ui.end_row(); }
        });
        let valid = !path.trim().is_empty() && size[0].is_finite() && size[1].is_finite() && size[0] > 0.0 && size[1] > 0.0 && rotation.is_finite();
        ui.horizontal(|ui| {
            if ui.add_enabled(valid, egui::Button::new("Attach to drawing")).clicked() {
                let cmd = match mode.as_str() { "PDF" => "pdfattach", "Raster Image" => "imageattach", _ => "attach" };
                let mut p = json!({"path":path.trim(),"at":at,"width":size[0],"height":size[1],"rotation":rotation});
                if cmd == "pdfattach" { p["page"] = json!(page); }
                action = Some((cmd, p));
            }
            if ui.button("Reset placement").clicked() { at=[0.0,0.0]; size=[100.0,100.0]; rotation=0.0; page=1; }
        });
        ui.separator();
        ui.small("R32 renders stable frames for metadata underlays so placement is visible on canvas. Raster IMAGE entities retain their existing frame. Full PDF/raster pixel decoding is still a separate renderer capability.");
    });
    set_temp(ctx,"underlay_mode",mode); set_temp(ctx,"underlay_path",path); set_temp(ctx,"underlay_page",page);
    set_temp(ctx,"underlay_at",at); set_temp(ctx,"underlay_size",size); set_temp(ctx,"underlay_rotation",rotation);
    action
}


fn image_edit_dialog(ctx: &egui::Context, open: &mut bool) -> Action {
    let mut handle=temp::<String>(ctx,"imageedit_handle").unwrap_or_default();
    let mut brightness=temp::<u64>(ctx,"imageedit_brightness").unwrap_or(50);
    let mut contrast=temp::<u64>(ctx,"imageedit_contrast").unwrap_or(50);
    let mut fade=temp::<u64>(ctx,"imageedit_fade").unwrap_or(0);
    let mut transparency=temp::<bool>(ctx,"imageedit_transparency").unwrap_or(false);
    let mut at=temp::<[f64;2]>(ctx,"imageedit_at").unwrap_or([0.0,0.0]);
    let mut size=temp::<[f64;2]>(ctx,"imageedit_size").unwrap_or([100.0,100.0]);
    let mut rotation=temp::<f64>(ctx,"imageedit_rotation").unwrap_or(0.0);
    let mut action=None;
    egui::Window::new("Raster Image Adjust / Transform").open(open).default_width(480.0).resizable(false).show(ctx,|ui|{
        ui.horizontal(|ui|{ui.label("Image handle");ui.text_edit_singleline(&mut handle);});
        ui.small("Leave handle blank to use the currently selected IMAGE entity.");
        egui::Grid::new("image_adjust_grid").num_columns(2).spacing(vec2(10.0,6.0)).show(ui,|ui|{
            ui.label("Brightness");ui.add(egui::Slider::new(&mut brightness,0..=100));ui.end_row();
            ui.label("Contrast");ui.add(egui::Slider::new(&mut contrast,0..=100));ui.end_row();
            ui.label("Fade");ui.add(egui::Slider::new(&mut fade,0..=100));ui.end_row();
            ui.label("Transparency");ui.checkbox(&mut transparency,"Enable");ui.end_row();
        });
        if ui.button("Apply image appearance").clicked(){
            let mut p=json!({"brightness":brightness,"contrast":contrast,"fade":fade,"transparency":transparency});
            if !handle.trim().is_empty(){p["handle"]=json!(handle.trim());}
            action=Some(("imageadjust",p));
        }
        ui.separator();
        egui::Grid::new("image_transform_grid").num_columns(3).spacing(vec2(10.0,6.0)).show(ui,|ui|{
            ui.label("Insert X / Y");ui.add(egui::DragValue::new(&mut at[0]).speed(0.5));ui.add(egui::DragValue::new(&mut at[1]).speed(0.5));ui.end_row();
            ui.label("Width / Height");ui.add(egui::DragValue::new(&mut size[0]).speed(0.5).range(0.000001..=1.0e12));ui.add(egui::DragValue::new(&mut size[1]).speed(0.5).range(0.000001..=1.0e12));ui.end_row();
            ui.label("Rotation°");ui.add(egui::DragValue::new(&mut rotation).speed(1.0));ui.label("");ui.end_row();
        });
        let valid=size[0].is_finite()&&size[1].is_finite()&&size[0]>0.0&&size[1]>0.0&&rotation.is_finite();
        if ui.add_enabled(valid,egui::Button::new("Apply placement / size / rotation")).clicked(){
            let mut p=json!({"at":at,"width":size[0],"height":size[1],"rotation":rotation});
            if !handle.trim().is_empty(){p["handle"]=json!(handle.trim());}
            action=Some(("imagetransform",p));
        }
        ui.small("R33 edits native IMAGE metadata through the engine. Pixel decoding/rendering remains a separate viewport capability.");
    });
    set_temp(ctx,"imageedit_handle",handle);set_temp(ctx,"imageedit_brightness",brightness);set_temp(ctx,"imageedit_contrast",contrast);set_temp(ctx,"imageedit_fade",fade);set_temp(ctx,"imageedit_transparency",transparency);set_temp(ctx,"imageedit_at",at);set_temp(ctx,"imageedit_size",size);set_temp(ctx,"imageedit_rotation",rotation);
    action
}

// ---------- 3D primitive / visual-style foundation ----------

/// Build a faceted 3D primitive through the engine. These are intentionally represented by
/// DXF-compatible 3DFACE entities until a true ACIS/B-Rep kernel is integrated.
fn primitive_3d(_app: &mut CadApp, ctx: &egui::Context, open: &mut bool, kind: &str) -> Action {
    let title = format!("{} - Faceted 3D", kind.to_ascii_uppercase());
    let mut action = None;
    let key = |suffix: &str| egui::Id::new(format!("primitive3d_{kind}_{suffix}"));
    let mut x = ctx.data_mut(|d| d.get_temp::<String>(key("x"))).unwrap_or_else(|| "0".into());
    let mut y = ctx.data_mut(|d| d.get_temp::<String>(key("y"))).unwrap_or_else(|| "0".into());
    let mut z = ctx.data_mut(|d| d.get_temp::<String>(key("z"))).unwrap_or_else(|| "0".into());
    let mut a = ctx.data_mut(|d| d.get_temp::<String>(key("a"))).unwrap_or_else(|| if matches!(kind, "cylinder"|"cone"|"sphere") { "5".into() } else if kind == "torus" { "7".into() } else { "10".into() });
    let mut b = ctx.data_mut(|d| d.get_temp::<String>(key("b"))).unwrap_or_else(|| if kind == "torus" { "2".into() } else { "10".into() });
    let mut h = ctx.data_mut(|d| d.get_temp::<String>(key("h"))).unwrap_or_else(|| "10".into());
    let mut use_ucs = ctx.data_mut(|d| d.get_temp::<bool>(key("use_ucs"))).unwrap_or(true);
    let parse = |v: &str| v.trim().parse::<f64>().ok().filter(|v| v.is_finite());
    egui::Window::new(title).open(open).default_width(390.0).resizable(false).show(ctx, |ui| {
        ui.label("Creates real drawing geometry as 3DFACE facets. This is not an ACIS solid.");
        ui.separator();
        egui::Grid::new(format!("primitive3d_grid_{kind}")).num_columns(2).spacing(vec2(12.0, 7.0)).show(ui, |ui| {
            ui.label(if matches!(kind, "box"|"pyramid") { "Base X" } else { "Center X" }); ui.text_edit_singleline(&mut x); ui.end_row();
            ui.label(if matches!(kind, "box"|"pyramid") { "Base Y" } else { "Center Y" }); ui.text_edit_singleline(&mut y); ui.end_row();
            ui.label(if matches!(kind, "box"|"pyramid") { "Base Z" } else { "Center Z" }); ui.text_edit_singleline(&mut z); ui.end_row();
            match kind {
                "box" | "pyramid" => {
                    ui.label("Length"); ui.text_edit_singleline(&mut a); ui.end_row();
                    ui.label("Width"); ui.text_edit_singleline(&mut b); ui.end_row();
                    ui.label("Height"); ui.text_edit_singleline(&mut h); ui.end_row();
                }
                "cylinder" | "cone" => {
                    ui.label("Radius"); ui.text_edit_singleline(&mut a); ui.end_row();
                    ui.label("Height"); ui.text_edit_singleline(&mut h); ui.end_row();
                }
                "sphere" => { ui.label("Radius"); ui.text_edit_singleline(&mut a); ui.end_row(); }
                "torus" => {
                    ui.label("Major radius"); ui.text_edit_singleline(&mut a); ui.end_row();
                    ui.label("Minor radius"); ui.text_edit_singleline(&mut b); ui.end_row();
                }
                _ => {}
            }
        });
        ui.checkbox(&mut use_ucs, "Interpret XYZ in current UCS");
        let pos = [parse(&x), parse(&y), parse(&z)];
        let aa = parse(&a); let bb = parse(&b); let hh = parse(&h);
        let valid = pos.iter().all(Option::is_some) && aa.is_some_and(|v| v > 0.0)
            && (!matches!(kind, "box"|"pyramid"|"torus") || bb.is_some_and(|v| v > 0.0))
            && (!matches!(kind, "box"|"pyramid"|"cylinder"|"cone") || hh.is_some_and(|v| v > 0.0))
            && (kind != "torus" || aa.zip(bb).is_some_and(|(major, minor)| minor < major));
        if ui.add_enabled(valid, egui::Button::new("Create in drawing")).clicked() {
            let pos = [pos[0].unwrap_or(0.0), pos[1].unwrap_or(0.0), pos[2].unwrap_or(0.0)];
            let params = match kind {
                "box" | "pyramid" => json!({"base":pos, "length":aa, "width":bb, "height":hh, "useUcs":use_ucs}),
                "cylinder" | "cone" => json!({"center":pos, "radius":aa, "height":hh, "segments":24, "useUcs":use_ucs}),
                "sphere" => json!({"center":pos, "radius":aa, "segments":24, "rings":12, "useUcs":use_ucs}),
                "torus" => json!({"center":pos, "majorRadius":aa, "minorRadius":bb, "segments":24, "tubeSegments":12, "useUcs":use_ucs}),
                _ => Value::Null,
            };
            action = Some((match kind { "box"=>"box", "pyramid"=>"pyramid", "cylinder"=>"cylinder", "cone"=>"cone", "sphere"=>"sphere", "torus"=>"torus", _=>"mesh" }, params));
        }
        ui.small("Tip: define a named UCS for 3D work, then use Orbit/View presets to inspect the generated facets.");
    });
    ctx.data_mut(|d| {
        d.insert_temp(key("x"), x); d.insert_temp(key("y"), y); d.insert_temp(key("z"), z);
        d.insert_temp(key("a"), a); d.insert_temp(key("b"), b); d.insert_temp(key("h"), h); d.insert_temp(key("use_ucs"), use_ucs);
    });
    action
}


/// Selection-based faceted 3D modeling. The engine owns all geometry creation; this dialog only
/// collects bounded parameters and shows the exact selection contract to the user.
fn solid_3d(app: &mut CadApp, ctx: &egui::Context, open: &mut bool, kind: &str) -> Action {
    if app.session.doc().is_err() { no_drawing(ctx, "3D Modeling", open); return None; }
    let selection = app.session.selection();
    let id = |suffix: &str| egui::Id::new(format!("solid3d_{kind}_{suffix}"));
    let mut height = ctx.data_mut(|d| d.get_temp::<String>(id("height"))).unwrap_or_else(|| "10".into());
    let mut angle = ctx.data_mut(|d| d.get_temp::<String>(id("angle"))).unwrap_or_else(|| "360".into());
    let mut segments = ctx.data_mut(|d| d.get_temp::<String>(id("segments"))).unwrap_or_else(|| "32".into());
    let mut samples = ctx.data_mut(|d| d.get_temp::<String>(id("samples"))).unwrap_or_else(|| "32".into());
    let parse = |v: &str| v.trim().parse::<f64>().ok().filter(|x| x.is_finite());
    let parse_u = |v: &str| v.trim().parse::<usize>().ok();
    let mut action=None;
    egui::Window::new(format!("3D {}", kind.to_ascii_uppercase())).open(open).default_width(430.0).resizable(false).show(ctx, |ui| {
        ui.label(format!("Selected objects: {}", selection.len()));
        match kind {
            "extrude" => {
                ui.label("Select one closed polyline or circle before running EXTRUDE.");
                ui.horizontal(|ui| { ui.label("Height"); ui.text_edit_singleline(&mut height); });
                let ok=selection.len()>=1 && parse(&height).is_some_and(|v| v>0.0);
                if ui.add_enabled(ok, egui::Button::new("Extrude selected profile")).clicked() {
                    action=Some(("extrude",json!({"height":parse(&height)})));
                }
            }
            "revolve" => {
                ui.label("Select one profile. R13 revolves it around the world Z axis.");
                ui.horizontal(|ui| { ui.label("Angle (deg)"); ui.text_edit_singleline(&mut angle); });
                ui.horizontal(|ui| { ui.label("Segments"); ui.text_edit_singleline(&mut segments); });
                let ok=selection.len()>=1 && parse(&angle).is_some_and(|v| v.abs()>1e-6 && v.abs()<=360.0)
                    && parse_u(&segments).is_some_and(|v| (8..=128).contains(&v));
                if ui.add_enabled(ok, egui::Button::new("Revolve selected profile")).clicked() {
                    action=Some(("revolve",json!({"angle":parse(&angle),"segments":parse_u(&segments)})));
                }
            }
            "sweep" => {
                ui.label("Selection order: closed profile first, path second (Line/2D Polyline/3D Polyline).");
                ui.small("Current faceted sweep translates the section along the path without twisting it.");
                if ui.add_enabled(selection.len()>=2, egui::Button::new("Sweep profile along path")).clicked() {
                    action=Some(("sweep",json!({})));
                }
            }
            "loft" => {
                ui.label("Select two or more closed profiles in section order.");
                ui.horizontal(|ui| { ui.label("Samples/profile"); ui.text_edit_singleline(&mut samples); });
                let ok=selection.len()>=2 && parse_u(&samples).is_some_and(|v| (8..=128).contains(&v));
                if ui.add_enabled(ok, egui::Button::new("Loft selected profiles")).clicked() {
                    action=Some(("loft",json!({"samples":parse_u(&samples)})));
                }
            }
            _ => { ui.label("Unsupported modeling command."); }
        }
        ui.separator();
        ui.small("R13 creates DXF-compatible 3DFACE facets. It is editable/saveable geometry, not an ACIS/B-Rep solid kernel.");
    });
    ctx.data_mut(|d| { d.insert_temp(id("height"),height); d.insert_temp(id("angle"),angle); d.insert_temp(id("segments"),segments); d.insert_temp(id("samples"),samples); });
    action
}


fn dynamic_block_dialog(ctx: &egui::Context, open: &mut bool) -> Action {
    let id=|n:&str| egui::Id::new(("dynamic_block_dialog",n));
    let mut name=ctx.data_mut(|d|d.get_temp::<String>(id("name"))).unwrap_or_default();
    let mut handle=ctx.data_mut(|d|d.get_temp::<String>(id("handle"))).unwrap_or_default();
    let mut property=ctx.data_mut(|d|d.get_temp::<String>(id("property"))).unwrap_or_default();
    let mut value=ctx.data_mut(|d|d.get_temp::<String>(id("value"))).unwrap_or_default();
    let mut grip_node=ctx.data_mut(|d|d.get_temp::<String>(id("grip_node"))).unwrap_or_default();
    let mut grip_part=ctx.data_mut(|d|d.get_temp::<String>(id("grip_part"))).unwrap_or_else(||"end".into());
    let mut grip_to=ctx.data_mut(|d|d.get_temp::<[f64;2]>(id("grip_to"))).unwrap_or([0.0,0.0]);
    let mut action=None;
    egui::Window::new("Dynamic Block Reference Editor").open(open).default_width(520.0).resizable(false).show(ctx,|ui|{
        ui.label("Inspects existing dynamic-block graphs and edits reference properties/grips through the canonical dynamic-block engine.");
        ui.horizontal(|ui|{ui.label("Block definition");ui.text_edit_singleline(&mut name);});
        ui.horizontal_wrapped(|ui|{
            if ui.add_enabled(!name.trim().is_empty(),egui::Button::new("Inspect definition")).clicked(){action=Some(("blocks.dynamic",json!({"name":name.trim()})));}
            if ui.add_enabled(!name.trim().is_empty(),egui::Button::new("Validate graph")).clicked(){action=Some(("blocks.dynamic.validate",json!({"name":name.trim()})));}
        });
        ui.separator();
        ui.horizontal(|ui|{ui.label("Block reference handle");ui.text_edit_singleline(&mut handle);});
        ui.horizontal_wrapped(|ui|{
            if ui.add_enabled(!handle.trim().is_empty(),egui::Button::new("Inspect reference properties")).clicked(){action=Some(("blocks.reference",json!({"handle":handle.trim()})));}
            if ui.add_enabled(!handle.trim().is_empty(),egui::Button::new("List dynamic grips")).clicked(){action=Some(("blocks.reference.grips",json!({"handle":handle.trim()})));}
        });
        ui.separator(); ui.strong("Set dynamic property");
        ui.horizontal_wrapped(|ui|{ ui.label("Property");ui.text_edit_singleline(&mut property);ui.label("Value");ui.text_edit_singleline(&mut value);
            let ok=!handle.trim().is_empty()&&!property.trim().is_empty()&&!value.trim().is_empty();
            if ui.add_enabled(ok,egui::Button::new("Apply property")).clicked(){
                let v=value.trim().parse::<f64>().map_or_else(|_|json!(value.trim()),|n|json!(n));
                action=Some(("blocks.reference.set",json!({"handle":handle.trim(),"name":property.trim(),"value":v})));
            }
        });
        ui.separator(); ui.strong("Move dynamic grip");
        ui.horizontal_wrapped(|ui|{
            ui.label("Node");ui.text_edit_singleline(&mut grip_node);
            ui.label("Part");egui::ComboBox::from_id_salt("dyn_grip_part").selected_text(&grip_part).show_ui(ui,|ui|{
                ui.selectable_value(&mut grip_part,"base".into(),"base"); ui.selectable_value(&mut grip_part,"end".into(),"end"); ui.selectable_value(&mut grip_part,"at".into(),"at");
            });
            ui.label("To X/Y");ui.add(egui::DragValue::new(&mut grip_to[0]).speed(0.5));ui.add(egui::DragValue::new(&mut grip_to[1]).speed(0.5));
        });
        let node=grip_node.trim().parse::<i32>().ok();
        if ui.add_enabled(!handle.trim().is_empty()&&node.is_some(),egui::Button::new("Move grip through engine")).clicked(){
            action=Some(("blocks.reference.grip",json!({"handle":handle.trim(),"node":node,"part":grip_part,"to":grip_to})));
        }
        ui.small("R27 edits existing dynamic-block reference values and grips. Creating new parameter/action graphs is still draft/API-level until DWG write-back is implemented and verified.");
    });
    ctx.data_mut(|d|{d.insert_temp(id("name"),name);d.insert_temp(id("handle"),handle);d.insert_temp(id("property"),property);d.insert_temp(id("value"),value);d.insert_temp(id("grip_node"),grip_node);d.insert_temp(id("grip_part"),grip_part);d.insert_temp(id("grip_to"),grip_to);});
    action
}

fn section_plane(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    if app.session.doc().is_err() { no_drawing(ctx, "Section Plane", open); return None; }
    let id = |suffix: &str| egui::Id::new(format!("sectionplane_{suffix}"));
    let get = |ctx: &egui::Context, key: egui::Id, default: &str| ctx.data_mut(|d| d.get_temp::<String>(key)).unwrap_or_else(|| default.into());
    let mut name = get(ctx, id("name"), "Section A");
    let mut ox = get(ctx, id("ox"), "0"); let mut oy = get(ctx, id("oy"), "0"); let mut oz = get(ctx, id("oz"), "0");
    let mut nx = get(ctx, id("nx"), "0"); let mut ny = get(ctx, id("ny"), "0"); let mut nz = get(ctx, id("nz"), "1");
    let mut width = get(ctx, id("width"), "20"); let mut height = get(ctx, id("height"), "12");
    let parse = |v: &str| v.trim().parse::<f64>().ok().filter(|x| x.is_finite());
    let mut action = None;
    egui::Window::new("Section Plane").open(open).default_width(430.0).resizable(false).show(ctx, |ui| {
        ui.label("Persistent 3D section marker with viewport half-space clipping foundation.");
        egui::Grid::new("sectionplane_grid").num_columns(4).striped(true).show(ui, |ui| {
            ui.label("Name"); ui.text_edit_singleline(&mut name); ui.end_row();
            ui.label("Origin X/Y/Z"); ui.text_edit_singleline(&mut ox); ui.text_edit_singleline(&mut oy); ui.text_edit_singleline(&mut oz); ui.end_row();
            ui.label("Normal X/Y/Z"); ui.text_edit_singleline(&mut nx); ui.text_edit_singleline(&mut ny); ui.text_edit_singleline(&mut nz); ui.end_row();
            ui.label("Width / Height"); ui.text_edit_singleline(&mut width); ui.text_edit_singleline(&mut height); ui.end_row();
        });
        let o = [parse(&ox), parse(&oy), parse(&oz)];
        let n = [parse(&nx), parse(&ny), parse(&nz)];
        let w = parse(&width); let h = parse(&height);
        let n2 = n.iter().flatten().map(|v| v*v).sum::<f64>();
        let ok = !name.trim().is_empty() && o.iter().all(Option::is_some) && n.iter().all(Option::is_some)
            && n2 > 1e-12 && w.is_some_and(|v| v > 0.0) && h.is_some_and(|v| v > 0.0);
        if ui.add_enabled(ok, egui::Button::new("Create section plane in drawing")).clicked() {
            action = Some(("sectionplane", json!({
                "name": name.trim(),
                "origin": [o[0], o[1], o[2]],
                "normal": [n[0], n[1], n[2]],
                "width": w,
                "height": h
            })));
        }
        ui.horizontal(|ui| {
            if ui.button("Toggle clipping").clicked() { action = Some(("sectionclip", json!({}))); }
            if ui.button("Keep + side").clicked() { action = Some(("sectionclip", json!({"on":true,"side":"positive"}))); }
            if ui.button("Keep - side").clicked() { action = Some(("sectionclip", json!({"on":true,"side":"negative"}))); }
            if ui.button("Clipping off").clicked() { action = Some(("sectionclip", json!({"on":false}))); }
            if ui.button("Toggle cut edges").clicked() { action = Some(("sectioncutstyle", json!({}))); }
            if ui.button("Generate section lines").clicked() { action = Some(("sectiongeometry3d", json!({}))); }
        });
        ui.small("Section clipping polygon-slices 3DFACE geometry and clips 3D segments. Cut-edge display can be toggled independently. This is viewport sectioning, not ACIS section-body/cap generation.");
    });
    ctx.data_mut(|d| {
        d.insert_temp(id("name"), name); d.insert_temp(id("ox"), ox); d.insert_temp(id("oy"), oy); d.insert_temp(id("oz"), oz);
        d.insert_temp(id("nx"), nx); d.insert_temp(id("ny"), ny); d.insert_temp(id("nz"), nz); d.insert_temp(id("width"), width); d.insert_temp(id("height"), height);
    });
    action
}


fn point_cloud_dialog(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(doc)=app.session.doc() else { no_drawing(ctx, "Point Cloud", open); return None; };
    let mut groups: std::collections::BTreeMap<String,(usize,usize,cadcraft_engine::geom::Vec3)>=std::collections::BTreeMap::new();
    for e in doc.model.iter(){
        let mut tagged=false; let mut group=None;
        for t in &e.common.xdata {
            if t.code==1001 && t.value=="TBT360_POINTCLOUD" { tagged=true; continue; }
            if tagged && t.code==1000 { group=Some(t.value.clone()); break; }
        }
        if let (Some(name),cadcraft_doc::EntityKind::Point(pt))=(group,&e.kind) { let v=groups.entry(name).or_default(); v.0+=1; if e.common.visible {v.1+=1;} v.2=v.2+pt.p; }
    }
    let id=|n:&str| egui::Id::new(("pointcloud_dialog",n));
    let mut name=ctx.data_mut(|d| d.get_temp::<String>(id("name"))).unwrap_or_else(|| "PointCloud".into());
    let mut text=ctx.data_mut(|d| d.get_temp::<String>(id("xyz"))).unwrap_or_else(|| "0,0,0\n5,0,1\n5,5,2\n0,5,1".into());
    let mut action=None;
    let mut point_size=doc.header.f64("POINTCLOUDSIZE",1.5).clamp(0.1,1000.0);
    let mut point_mode=doc.header.i64("POINTCLOUDMODE",0).clamp(0,99);
    let mut crop_min=ctx.data_mut(|d| d.get_temp::<[f64;3]>(id("crop_min"))).unwrap_or([-10.0,-10.0,-10.0]);
    let mut crop_max=ctx.data_mut(|d| d.get_temp::<[f64;3]>(id("crop_max"))).unwrap_or([10.0,10.0,10.0]);
    let mut crop_invert=ctx.data_mut(|d| d.get_temp::<bool>(id("crop_invert"))).unwrap_or(false);
    let mut move_delta=ctx.data_mut(|d| d.get_temp::<[f64;3]>(id("move_delta"))).unwrap_or([0.0,0.0,0.0]);
    let mut scale_factor=ctx.data_mut(|d| d.get_temp::<f64>(id("scale_factor"))).unwrap_or(1.0).clamp(0.000001,1_000_000.0);
    let mut rotate_angle=ctx.data_mut(|d| d.get_temp::<f64>(id("rotate_angle"))).unwrap_or(0.0);
    let mut recenter_target=ctx.data_mut(|d| d.get_temp::<[f64;3]>(id("recenter_target"))).unwrap_or([0.0,0.0,0.0]);
    let mut rename_target=ctx.data_mut(|d| d.get_temp::<String>(id("rename_target"))).unwrap_or_default();
    let mut merge_target=ctx.data_mut(|d| d.get_temp::<String>(id("merge_target"))).unwrap_or_default();
    let mut copy_visible_target=ctx.data_mut(|d| d.get_temp::<String>(id("copy_visible_target"))).unwrap_or_default();
    let mut cloud_color=ctx.data_mut(|d| d.get_temp::<String>(id("cloud_color"))).unwrap_or_else(|| "7".into());
    let mut export_visible_only=ctx.data_mut(|d| d.get_temp::<bool>(id("export_visible_only"))).unwrap_or(false);
    let mut export_delimiter=ctx.data_mut(|d| d.get_temp::<String>(id("export_delimiter"))).unwrap_or_else(|| "comma".into());
    let mut rotate3d_axis=ctx.data_mut(|d| d.get_temp::<[f64;3]>(id("rotate3d_axis"))).unwrap_or([0.0,0.0,1.0]);
    let mut rotate3d_angle=ctx.data_mut(|d| d.get_temp::<f64>(id("rotate3d_angle"))).unwrap_or(0.0);
    let mut filter_z=ctx.data_mut(|d| d.get_temp::<[f64;2]>(id("filter_z"))).unwrap_or([-10.0,10.0]);
    let mut filter_z_invert=ctx.data_mut(|d| d.get_temp::<bool>(id("filter_z_invert"))).unwrap_or(false);
    let mut nearest_at=ctx.data_mut(|d| d.get_temp::<[f64;3]>(id("nearest_at"))).unwrap_or([0.0,0.0,0.0]);
    let mut nearest_count=ctx.data_mut(|d| d.get_temp::<u64>(id("nearest_count"))).unwrap_or(1).clamp(1,64);
    let mut nearest_visible_only=ctx.data_mut(|d| d.get_temp::<bool>(id("nearest_visible_only"))).unwrap_or(false);
    let mut radius_query=ctx.data_mut(|d| d.get_temp::<f64>(id("radius_query"))).unwrap_or(10.0).max(0.000001);
    let mut radius_count=ctx.data_mut(|d| d.get_temp::<u64>(id("radius_count"))).unwrap_or(256).clamp(1,4096);
    let mut voxel_size=ctx.data_mut(|d| d.get_temp::<f64>(id("voxel_size"))).unwrap_or(1.0).max(0.000001);
    let mut dedup_tolerance=ctx.data_mut(|d| d.get_temp::<f64>(id("dedup_tolerance"))).unwrap_or(0.001).max(0.000000001);
    let mut clip_origin=ctx.data_mut(|d| d.get_temp::<[f64;3]>(id("clip_origin"))).unwrap_or([0.0,0.0,0.0]);
    let mut clip_normal=ctx.data_mut(|d| d.get_temp::<[f64;3]>(id("clip_normal"))).unwrap_or([0.0,0.0,1.0]);
    let mut clip_positive=ctx.data_mut(|d| d.get_temp::<bool>(id("clip_positive"))).unwrap_or(true);
    let mut slice_half=ctx.data_mut(|d| d.get_temp::<f64>(id("slice_half"))).unwrap_or(0.5).max(0.000001);
    let mut slice_invert=ctx.data_mut(|d| d.get_temp::<bool>(id("slice_invert"))).unwrap_or(false);
    let mut grid_cell=ctx.data_mut(|d| d.get_temp::<f64>(id("grid_cell"))).unwrap_or(1.0).max(0.000001);
    let mut classify_breaks=ctx.data_mut(|d| d.get_temp::<String>(id("classify_breaks"))).unwrap_or_else(|| "0,3,10".into());
    let mut icp_target=ctx.data_mut(|d| d.get_temp::<String>(id("icp_target"))).unwrap_or_default();
    let mut icp_apply=ctx.data_mut(|d| d.get_temp::<bool>(id("icp_apply"))).unwrap_or(false);
    let mut zstats_bins=ctx.data_mut(|d| d.get_temp::<u64>(id("zstats_bins"))).unwrap_or(10).clamp(2,100);
    egui::Window::new("Point Cloud (XYZ)").open(open).default_width(540.0).resizable(true).show(ctx,|ui|{
        ui.label("Paste XYZ points: one point per line, separated by comma, semicolon, tab, or spaces.");
        ui.horizontal(|ui|{ ui.label("Name"); ui.text_edit_singleline(&mut name); });
        ui.horizontal(|ui|{
            ui.label("Display size");
            ui.add(egui::DragValue::new(&mut point_size).speed(0.1).range(0.1..=1000.0));
            ui.label("Point mode");
            ui.add(egui::DragValue::new(&mut point_mode).range(0..=99));
            if ui.small_button("Apply display style").clicked(){ action=Some(("pointcloud.style",json!({"size":point_size,"mode":point_mode}))); }
        });
        ui.add(egui::TextEdit::multiline(&mut text).desired_rows(12).desired_width(f32::INFINITY));
        let mut points=Vec::<[f64;3]>::new(); let mut invalid=0usize;
        for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let vals:Vec<_>=line.split(|c:char| c==','||c==';'||c=='\t'||c==' ').filter(|v|!v.is_empty()).collect();
            if vals.len()<3 { invalid+=1; continue; }
            let parsed=(vals[0].parse::<f64>(),vals[1].parse::<f64>(),vals[2].parse::<f64>());
            match parsed { (Ok(x),Ok(y),Ok(z)) if x.is_finite()&&y.is_finite()&&z.is_finite()=>points.push([x,y,z]), _=>invalid+=1 }
        }
        ui.label(format!("Valid points: {}   Invalid lines: {}",points.len(),invalid));
        if ui.add_enabled(!points.is_empty() && invalid==0 && points.len()<=200_000 && !name.trim().is_empty(), egui::Button::new("Import into drawing")).clicked() {
            action=Some(("pointcloud",json!({"name":name.trim(),"points":points,"maxPoints":200000})));
        }
        if !groups.is_empty(){
            ui.separator(); ui.strong("Point-cloud groups in drawing");
            for (group,(count,visible_count,sum)) in &groups {
                ui.horizontal_wrapped(|ui|{
                    let c=if *count>0 {*sum/(*count as f64)} else {cadcraft_engine::geom::Vec3::ZERO};
                    ui.label(format!("{group} — {count} pts"));
                    ui.small(format!("C=({:.2}, {:.2}, {:.2})",c.x,c.y,c.z));
                    let shown=*visible_count>0;
                    if ui.small_button(if shown {"Hide"} else {"Show"}).clicked(){ action=Some(("pointcloud.visible",json!({"name":group,"visible":!shown}))); }
                    if ui.small_button("Isolate").clicked(){ action=Some(("pointcloud.isolate",json!({"name":group}))); }
                    if ui.small_button("LOD 1/5").clicked(){ action=Some(("pointcloud.sample",json!({"name":group,"keepEvery":5}))); }
                    if ui.small_button("LOD 1/20").clicked(){ action=Some(("pointcloud.sample",json!({"name":group,"keepEvery":20}))); }
                    if ui.small_button("Full").clicked(){ action=Some(("pointcloud.sample",json!({"name":group,"keepEvery":1}))); }
                    if ui.small_button("Select visible").clicked(){ action=Some(("pointcloud.selectvisible",json!({"name":group}))); }
                    if ui.small_button("Crop box").clicked(){ action=Some(("pointcloud.crop",json!({"name":group,"min":crop_min,"max":crop_max,"invert":crop_invert}))); }
                    if ui.small_button("Translate").clicked(){ action=Some(("pointcloud.translate",json!({"name":group,"delta":move_delta}))); }
                    if ui.small_button("Scale").clicked(){ action=Some(("pointcloud.scale",json!({"name":group,"factor":scale_factor}))); }
                    if ui.small_button("Rotate Z").clicked(){ action=Some(("pointcloud.rotate",json!({"name":group,"angle":rotate_angle}))); }
                    if ui.small_button("Recenter").clicked(){ action=Some(("pointcloud.recenter",json!({"name":group,"target":recenter_target}))); }
                    if ui.small_button("Rename").clicked() && !rename_target.trim().is_empty(){ action=Some(("pointcloud.rename",json!({"name":group,"newName":rename_target.trim()}))); }
                    if ui.small_button("Merge").clicked() && !merge_target.trim().is_empty(){ action=Some(("pointcloud.merge",json!({"source":group,"target":merge_target.trim()}))); }
                    if ui.small_button("Bounds").clicked(){ action=Some(("pointcloud.bounds",json!({"name":group}))); }
                    if ui.small_button("3D Bounds").clicked(){ action=Some(("pointcloud.box3d",json!({"name":group}))); }
                    if ui.small_button("Density").clicked(){ action=Some(("pointcloud.density",json!({"name":group,"visibleOnly":nearest_visible_only}))); }
                    if ui.small_button("Fit plane").clicked(){ action=Some(("pointcloud.planefit",json!({"name":group,"visibleOnly":nearest_visible_only}))); }
                    if ui.small_button("Create fit plane").clicked(){ action=Some(("pointcloud.plane3d",json!({"name":group,"visibleOnly":nearest_visible_only,"margin":0.0}))); }
                    if ui.small_button("Grid index").clicked(){ action=Some(("pointcloud.gridindex",json!({"name":group,"cellSize":grid_cell,"visibleOnly":nearest_visible_only}))); }
                    if ui.small_button("Octree stats").clicked(){ action=Some(("pointcloud.octreestats",json!({"name":group,"maxDepth":8,"leafSize":128,"visibleOnly":nearest_visible_only}))); }
                    if ui.small_button("Build octree").clicked(){ action=Some(("pointcloud.octreebuild",json!({"name":group,"maxDepth":8,"leafSize":128,"visibleOnly":nearest_visible_only}))); }
                    if ui.small_button("Classify Z").clicked(){ let breaks=classify_breaks.split(',').filter_map(|v|v.trim().parse::<f64>().ok()).collect::<Vec<_>>(); if !breaks.is_empty(){action=Some(("pointcloud.classifyz",json!({"name":group,"breaks":breaks,"visibleOnly":nearest_visible_only})));}}
                    if ui.small_button("ICP translate").clicked() && !icp_target.trim().is_empty(){ action=Some(("pointcloud.icptranslate",json!({"source":group,"target":icp_target.trim(),"visibleOnly":nearest_visible_only,"apply":icp_apply}))); }
                    if ui.small_button("ICP rigid 6DoF").clicked() && !icp_target.trim().is_empty(){ action=Some(("pointcloud.icprigid",json!({"source":group,"target":icp_target.trim(),"visibleOnly":nearest_visible_only,"apply":icp_apply}))); }
                    if ui.small_button("Copy visible").clicked() && !copy_visible_target.trim().is_empty(){ action=Some(("pointcloud.copyvisible",json!({"name":group,"newName":copy_visible_target.trim()}))); }
                    if ui.small_button("Color").clicked() && !cloud_color.trim().is_empty(){ action=Some(("pointcloud.color",json!({"name":group,"color":cloud_color.trim()}))); }
                    if ui.small_button("Export XYZ").clicked(){ action=Some(("pointcloud.exportxyz",json!({"name":group,"visibleOnly":export_visible_only,"delimiter":export_delimiter}))); }
                    if ui.small_button("Rotate 3D").clicked(){ action=Some(("pointcloud.rotate3d",json!({"name":group,"axis":rotate3d_axis,"angle":rotate3d_angle}))); }
                    if ui.small_button("Filter Z").clicked(){ action=Some(("pointcloud.filterz",json!({"name":group,"minZ":filter_z[0],"maxZ":filter_z[1],"invert":filter_z_invert}))); }
                    if ui.small_button("Nearest").clicked(){ action=Some(("pointcloud.nearest",json!({"name":group,"at":nearest_at,"maxResults":nearest_count,"visibleOnly":nearest_visible_only}))); }
                    if ui.small_button("Radius query").clicked(){ action=Some(("pointcloud.radius",json!({"name":group,"at":nearest_at,"radius":radius_query,"maxResults":radius_count,"visibleOnly":nearest_visible_only}))); }
                    ui.label("Voxel"); ui.add(egui::DragValue::new(&mut voxel_size).speed(0.1).range(0.000001..=1.0e12));
                    if ui.small_button("Voxel display").clicked(){ action=Some(("pointcloud.voxel",json!({"name":group,"size":voxel_size}))); }
                    ui.label("Dedup tol"); ui.add(egui::DragValue::new(&mut dedup_tolerance).speed(0.0001).range(0.000000001..=1.0e12));
                    if ui.small_button("Find dupes").clicked(){ action=Some(("pointcloud.dedup",json!({"name":group,"tolerance":dedup_tolerance}))); }
                    if ui.small_button("Remove dupes").clicked(){ action=Some(("pointcloud.dedup",json!({"name":group,"tolerance":dedup_tolerance,"delete":true}))); }
                    if ui.small_button("Delete").clicked(){ action=Some(("pointcloud.delete",json!({"name":group}))); }
                });
            }
            ui.separator(); ui.strong("3D crop box (non-destructive visibility crop)");
            ui.horizontal_wrapped(|ui|{
                ui.label("Min"); for v in &mut crop_min { ui.add(egui::DragValue::new(v).speed(0.5)); }
                ui.label("Max"); for v in &mut crop_max { ui.add(egui::DragValue::new(v).speed(0.5)); }
                ui.checkbox(&mut crop_invert,"Invert");
            });
            ui.separator(); ui.strong("Cloud transform (persistent XYZ edit)");
            ui.horizontal_wrapped(|ui|{
                ui.label("Move dXYZ"); for v in &mut move_delta { ui.add(egui::DragValue::new(v).speed(0.5)); }
                ui.label("Scale about centroid"); ui.add(egui::DragValue::new(&mut scale_factor).speed(0.05).range(0.000001..=1_000_000.0));
                ui.label("Rotate Z°"); ui.add(egui::DragValue::new(&mut rotate_angle).speed(1.0));
            });
            ui.horizontal_wrapped(|ui|{
                ui.label("Recenter target XYZ"); for v in &mut recenter_target { ui.add(egui::DragValue::new(v).speed(0.5)); }
            });
            ui.horizontal_wrapped(|ui|{
                ui.label("Rotate 3D axis"); for v in &mut rotate3d_axis { ui.add(egui::DragValue::new(v).speed(0.1)); }
                ui.label("Angle°"); ui.add(egui::DragValue::new(&mut rotate3d_angle).speed(1.0));
            });
            ui.horizontal_wrapped(|ui|{
                ui.label("Elevation filter Z"); ui.add(egui::DragValue::new(&mut filter_z[0]).speed(0.5)); ui.label("to"); ui.add(egui::DragValue::new(&mut filter_z[1]).speed(0.5));
                ui.checkbox(&mut filter_z_invert,"Invert");
            });
            ui.horizontal_wrapped(|ui|{
                ui.label("Nearest query XYZ"); for v in &mut nearest_at { ui.add(egui::DragValue::new(v).speed(0.5)); }
                ui.label("Nearest count"); ui.add(egui::DragValue::new(&mut nearest_count).range(1..=64));
                ui.label("Radius"); ui.add(egui::DragValue::new(&mut radius_query).speed(0.5).range(0.000001..=1.0e12));
                ui.label("Radius max"); ui.add(egui::DragValue::new(&mut radius_count).range(1..=4096));
                ui.checkbox(&mut nearest_visible_only,"Visible only");
            });
            ui.horizontal_wrapped(|ui|{
                ui.label("Clip plane origin"); for v in &mut clip_origin { ui.add(egui::DragValue::new(v).speed(0.5)); }
                ui.label("normal"); for v in &mut clip_normal { ui.add(egui::DragValue::new(v).speed(0.1)); }
                ui.checkbox(&mut clip_positive,"Keep + side");
                let clip_ok=clip_normal.iter().map(|v|v*v).sum::<f64>()>1e-12;
                if ui.add_enabled(clip_ok,egui::Button::new("Clip by plane")).clicked(){ action=Some(("pointcloud.clipplane",json!({"name":name.trim(),"origin":clip_origin,"normal":clip_normal,"keepPositive":clip_positive}))); }
                ui.label("Slice ±"); ui.add(egui::DragValue::new(&mut slice_half).speed(0.1).range(0.000001..=1.0e12));
                ui.checkbox(&mut slice_invert,"Outside band");
                if ui.add_enabled(clip_ok,egui::Button::new("Slice band")).clicked(){ action=Some(("pointcloud.sliceplane",json!({"name":name.trim(),"origin":clip_origin,"normal":clip_normal,"halfThickness":slice_half,"invert":slice_invert}))); }
            });
            ui.horizontal_wrapped(|ui|{
                ui.label("Z stats bins"); ui.add(egui::DragValue::new(&mut zstats_bins).range(2..=100));
                if ui.button("Elevation statistics").clicked(){action=Some(("pointcloud.zstats",json!({"name":name.trim(),"bins":zstats_bins})));}
            });
            ui.horizontal_wrapped(|ui|{
                ui.label("Grid cell"); ui.add(egui::DragValue::new(&mut grid_cell).speed(0.1).range(0.000001..=1.0e12));
                ui.label("Class Z breaks"); ui.text_edit_singleline(&mut classify_breaks);
                ui.label("ICP target"); ui.text_edit_singleline(&mut icp_target); ui.checkbox(&mut icp_apply,"Apply registration");
            });
            ui.horizontal_wrapped(|ui|{
                ui.label("Rename to"); ui.text_edit_singleline(&mut rename_target);
                ui.label("Merge into"); ui.text_edit_singleline(&mut merge_target);
                ui.label("Copy visible as"); ui.text_edit_singleline(&mut copy_visible_target);
            });
            ui.horizontal_wrapped(|ui|{
                ui.label("Cloud color"); ui.text_edit_singleline(&mut cloud_color);
                ui.checkbox(&mut export_visible_only,"Export visible only");
                ui.label("Delimiter"); egui::ComboBox::from_id_salt("pointcloud_export_delimiter").selected_text(&export_delimiter).show_ui(ui,|ui|{
                    ui.selectable_value(&mut export_delimiter,"comma".into(),"Comma");
                    ui.selectable_value(&mut export_delimiter,"space".into(),"Space");
                    ui.selectable_value(&mut export_delimiter,"tab".into(),"Tab");
                });
            });
            if ui.button("Show all point clouds").clicked(){ action=Some(("pointcloud.showall",json!({}))); }
        }
        ui.small("R45 adds bounded translation-only ICP registration, uniform-grid index statistics and persistent elevation classification. R31 keeps native XYZ POINT data and adds density statistics plus copy-visible-to-new-group alongside nearest/radius queries, voxel display filtering, arbitrary clip-plane filtering, plane-slice bands, transforms, bounds/color/export, LOD and crop. Filters remain non-destructive. This is still not LAS/LAZ/E57/RCP indexing.");
    });
    ctx.data_mut(|d|{d.insert_temp(id("name"),name);d.insert_temp(id("xyz"),text);d.insert_temp(id("crop_min"),crop_min);d.insert_temp(id("crop_max"),crop_max);d.insert_temp(id("crop_invert"),crop_invert);d.insert_temp(id("move_delta"),move_delta);d.insert_temp(id("scale_factor"),scale_factor);d.insert_temp(id("rotate_angle"),rotate_angle);d.insert_temp(id("recenter_target"),recenter_target);d.insert_temp(id("rename_target"),rename_target);d.insert_temp(id("merge_target"),merge_target);d.insert_temp(id("copy_visible_target"),copy_visible_target);d.insert_temp(id("cloud_color"),cloud_color);d.insert_temp(id("export_visible_only"),export_visible_only);d.insert_temp(id("export_delimiter"),export_delimiter);d.insert_temp(id("rotate3d_axis"),rotate3d_axis);d.insert_temp(id("rotate3d_angle"),rotate3d_angle);d.insert_temp(id("filter_z"),filter_z);d.insert_temp(id("filter_z_invert"),filter_z_invert);d.insert_temp(id("nearest_at"),nearest_at);d.insert_temp(id("nearest_count"),nearest_count);d.insert_temp(id("nearest_visible_only"),nearest_visible_only);d.insert_temp(id("radius_query"),radius_query);d.insert_temp(id("radius_count"),radius_count);d.insert_temp(id("voxel_size"),voxel_size);d.insert_temp(id("dedup_tolerance"),dedup_tolerance);d.insert_temp(id("clip_origin"),clip_origin);d.insert_temp(id("clip_normal"),clip_normal);d.insert_temp(id("clip_positive"),clip_positive);d.insert_temp(id("slice_half"),slice_half);d.insert_temp(id("slice_invert"),slice_invert);d.insert_temp(id("grid_cell"),grid_cell);d.insert_temp(id("classify_breaks"),classify_breaks);d.insert_temp(id("icp_target"),icp_target);d.insert_temp(id("icp_apply"),icp_apply);d.insert_temp(id("zstats_bins"),zstats_bins);});
    action
}


fn facet_transform_dialog(ctx: &egui::Context, open: &mut bool) -> Action {
    let id=|n:&str| egui::Id::new(("facet_transform_dialog",n));
    let mut delta=ctx.data_mut(|d|d.get_temp::<[f64;3]>(id("delta"))).unwrap_or([0.0,0.0,0.0]);
    let mut factor=ctx.data_mut(|d|d.get_temp::<f64>(id("factor"))).unwrap_or(1.0).max(0.000001);
    let mut origin=ctx.data_mut(|d|d.get_temp::<[f64;3]>(id("origin"))).unwrap_or([0.0,0.0,0.0]);
    let mut use_centroid=ctx.data_mut(|d|d.get_temp::<bool>(id("use_centroid"))).unwrap_or(true);
    let mut axis=ctx.data_mut(|d|d.get_temp::<[f64;3]>(id("axis"))).unwrap_or([0.0,0.0,1.0]);
    let mut angle=ctx.data_mut(|d|d.get_temp::<f64>(id("angle"))).unwrap_or(0.0);
    let mut mirror_origin=ctx.data_mut(|d|d.get_temp::<[f64;3]>(id("mirror_origin"))).unwrap_or([0.0,0.0,0.0]);
    let mut mirror_normal=ctx.data_mut(|d|d.get_temp::<[f64;3]>(id("mirror_normal"))).unwrap_or([1.0,0.0,0.0]);
    let mut offset_distance=ctx.data_mut(|d|d.get_temp::<f64>(id("offset_distance"))).unwrap_or(0.0);
    let mut action=None;
    egui::Window::new("Transform Selected 3D Facets").open(open).default_width(470.0).resizable(false).show(ctx,|ui|{
        ui.label("Transforms selected native 3DFACE geometry through the engine; Undo/Redo and Save/Reopen use the normal document transaction path.");
        ui.horizontal_wrapped(|ui|{ui.label("Move dXYZ"); for v in &mut delta{ui.add(egui::DragValue::new(v).speed(0.5));} if ui.button("Move").clicked(){action=Some(("facettransform3d",json!({"mode":"move","delta":delta})));}});
        ui.separator();
        ui.horizontal_wrapped(|ui|{ui.label("Scale");ui.add(egui::DragValue::new(&mut factor).speed(0.05).range(0.000001..=1_000_000.0));ui.checkbox(&mut use_centroid,"Use selection centroid");});
        if !use_centroid{ui.horizontal_wrapped(|ui|{ui.label("Origin XYZ");for v in &mut origin{ui.add(egui::DragValue::new(v).speed(0.5));}});}
        if ui.button("Scale selected facets").clicked(){let mut p=json!({"mode":"scale","factor":factor});if !use_centroid{p["origin"]=json!(origin);}action=Some(("facettransform3d",p));}
        ui.separator();
        ui.horizontal_wrapped(|ui|{ui.label("Rotate axis XYZ");for v in &mut axis{ui.add(egui::DragValue::new(v).speed(0.1));}ui.label("Angle°");ui.add(egui::DragValue::new(&mut angle).speed(1.0));});
        if ui.button("Rotate selected facets").clicked(){let mut p=json!({"mode":"rotate","axis":axis,"angle":angle});if !use_centroid{p["origin"]=json!(origin);}action=Some(("facettransform3d",p));}
        ui.separator(); ui.strong("Mirror across arbitrary 3D plane");
        ui.horizontal_wrapped(|ui|{ui.label("Plane origin");for v in &mut mirror_origin{ui.add(egui::DragValue::new(v).speed(0.5));}});
        ui.horizontal_wrapped(|ui|{ui.label("Plane normal");for v in &mut mirror_normal{ui.add(egui::DragValue::new(v).speed(0.1));}});
        let mirror_ok=mirror_normal.iter().map(|v|v*v).sum::<f64>()>1e-12;
        if ui.add_enabled(mirror_ok,egui::Button::new("Mirror selected facets")).clicked(){action=Some(("facetmirror3d",json!({"origin":mirror_origin,"normal":mirror_normal})));}
        ui.separator(); ui.strong("Offset facets along own normals");
        ui.horizontal_wrapped(|ui|{
            ui.label("Distance"); ui.add(egui::DragValue::new(&mut offset_distance).speed(0.1));
            if ui.button("Offset selected facets").clicked(){action=Some(("facetoffset3d",json!({"distance":offset_distance})));}
        });
        ui.small("Per-face normal offset for faceted repair/modeling; this is not a B-Rep shell offset.");
        ui.separator(); ui.strong("Facet cleanup");
        ui.horizontal_wrapped(|ui|{
            if ui.button("Report duplicate facets").clicked(){action=Some(("facetdedup3d",json!({"delete":false})));}
            if ui.button("Delete duplicate facets").clicked(){action=Some(("facetdedup3d",json!({"delete":true})));}
        });
        ui.small("R30 transforms/mirrors/offsets native 3DFACE and can detect/remove geometric duplicate facets. It does not claim ACIS/B-Rep topology healing.");
    });
    ctx.data_mut(|d|{d.insert_temp(id("delta"),delta);d.insert_temp(id("factor"),factor);d.insert_temp(id("origin"),origin);d.insert_temp(id("use_centroid"),use_centroid);d.insert_temp(id("axis"),axis);d.insert_temp(id("angle"),angle);d.insert_temp(id("mirror_origin"),mirror_origin);d.insert_temp(id("mirror_normal"),mirror_normal);d.insert_temp(id("offset_distance"),offset_distance);});
    action
}

fn view3d_dialog(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(doc)=app.session.doc() else { no_drawing(ctx, "3D View / Camera", open); return None; };
    let mut yaw=doc.header.f64("VIEWYAW",0.0);
    let mut pitch=doc.header.f64("VIEWPITCH",0.0);
    let mut distance=doc.header.f64("VIEWDIST",100.0).max(0.001);
    let mut projection=doc.header.str("VIEWPROJ","orthographic").to_string();
    let mut action=None;
    egui::Window::new("3D View / Camera").open(open).default_width(440.0).resizable(false).show(ctx,|ui|{
        ui.horizontal(|ui|{
            ui.label("Projection");
            egui::ComboBox::from_id_salt("view3d_projection").selected_text(&projection).show_ui(ui,|ui|{
                ui.selectable_value(&mut projection,"orthographic".into(),"Orthographic");
                ui.selectable_value(&mut projection,"perspective".into(),"Perspective");
            });
        });
        ui.add(egui::Slider::new(&mut yaw,-180.0..=180.0).text("Yaw"));
        ui.add(egui::Slider::new(&mut pitch,-89.0..=89.0).text("Pitch"));
        if projection=="perspective" { ui.add(egui::Slider::new(&mut distance,1.0..=10000.0).logarithmic(true).text("Camera distance")); }
        ui.horizontal_wrapped(|ui|{
            for (label,preset) in [("Top","top"),("Bottom","bottom"),("Front","front"),("Back","back"),("Right","right"),("Left","left"),("SE Iso","seiso"),("SW Iso","swiso"),("NE Iso","neiso"),("NW Iso","nwiso")] {
                if ui.small_button(label).clicked(){ action=Some(("3dorbit",json!({"preset":preset,"projection":projection,"distance":distance}))); }
            }
        });
        if ui.button("Apply view").clicked(){ action=Some(("3dorbit",json!({"yaw":yaw,"pitch":pitch,"projection":projection,"distance":distance}))); }
        ui.small("Perspective currently applies to native 3D entities/facets and 3D labels. 2D drafting geometry remains exact in XY.");
    });
    action
}

fn text3d_dialog(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    if app.session.doc().is_err() { no_drawing(ctx, "3D Text Label", open); return None; }
    let id=|n:&str| egui::Id::new(("text3d_dialog",n));
    let mut x=ctx.data_mut(|d| d.get_temp::<String>(id("x"))).unwrap_or_else(|| "0".into());
    let mut y=ctx.data_mut(|d| d.get_temp::<String>(id("y"))).unwrap_or_else(|| "0".into());
    let mut z=ctx.data_mut(|d| d.get_temp::<String>(id("z"))).unwrap_or_else(|| "0".into());
    let mut height=ctx.data_mut(|d| d.get_temp::<String>(id("height"))).unwrap_or_else(|| "2.5".into());
    let mut text=ctx.data_mut(|d| d.get_temp::<String>(id("text"))).unwrap_or_else(|| "3D Label".into());
    let parse=|v:&str| v.trim().parse::<f64>().ok().filter(|v| v.is_finite());
    let mut action=None;
    egui::Window::new("3D Text Label").open(open).default_width(440.0).resizable(false).show(ctx,|ui|{
        ui.label("Creates a screen-facing TEXT entity anchored at a WCS XYZ point. It follows 3D orbit, perspective and section clipping.");
        egui::Grid::new("text3d_grid").num_columns(2).show(ui,|ui|{
            ui.label("X"); ui.text_edit_singleline(&mut x); ui.end_row();
            ui.label("Y"); ui.text_edit_singleline(&mut y); ui.end_row();
            ui.label("Z"); ui.text_edit_singleline(&mut z); ui.end_row();
            ui.label("Height"); ui.text_edit_singleline(&mut height); ui.end_row();
            ui.label("Text"); ui.text_edit_singleline(&mut text); ui.end_row();
        });
        let vals=(parse(&x),parse(&y),parse(&z),parse(&height));
        let ok=vals.0.is_some()&&vals.1.is_some()&&vals.2.is_some()&&vals.3.is_some_and(|v|v>0.0)&&!text.trim().is_empty();
        if ui.add_enabled(ok,egui::Button::new("Place 3D label")).clicked(){
            action=Some(("text3d",json!({"at":[vals.0.unwrap_or(0.0),vals.1.unwrap_or(0.0),vals.2.unwrap_or(0.0)],"height":vals.3,"text":text.trim()})));
        }
    });
    ctx.data_mut(|d|{d.insert_temp(id("x"),x);d.insert_temp(id("y"),y);d.insert_temp(id("z"),z);d.insert_temp(id("height"),height);d.insert_temp(id("text"),text);});
    action
}


fn dim3d_dialog(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    if app.session.doc().is_err() { no_drawing(ctx, "3D Distance Dimension", open); return None; }
    let id=|n:&str| egui::Id::new(("dim3d_dialog",n));
    let mut values: Vec<String>=(0..9).map(|i|ctx.data_mut(|d|d.get_temp::<String>(id(&format!("v{i}")))).unwrap_or_else(|| if i==3 {"10".into()} else if i==8 {"1".into()} else {"0".into()})).collect();
    let parse=|v:&str|v.trim().parse::<f64>().ok().filter(|v|v.is_finite());
    let mut action=None;
    egui::Window::new("3D Distance Dimension").open(open).default_width(470.0).resizable(false).show(ctx,|ui|{
        ui.label("Creates native LINE/TEXT entities anchored in WCS 3D. The annotation follows Orbit, Perspective and Section Clip.");
        egui::Grid::new("dim3d_grid").num_columns(4).show(ui,|ui|{
            ui.label(""); ui.label("X"); ui.label("Y"); ui.label("Z"); ui.end_row();
            for (row,base) in [("A",0usize),("B",3usize),("Offset",6usize)] { ui.label(row); for i in 0..3 { ui.text_edit_singleline(&mut values[base+i]); } ui.end_row(); }
        });
        let nums:Vec<_>=values.iter().map(|v|parse(v)).collect();
        let ok=nums.iter().all(Option::is_some);
        if ui.add_enabled(ok,egui::Button::new("Create 3D dimension")).clicked(){
            action=Some(("dim3d",json!({"a":[nums[0],nums[1],nums[2]],"b":[nums[3],nums[4],nums[5]],"offset":[nums[6],nums[7],nums[8]]})));
        }
        ui.small("Current R18 distance dimension is non-associative: it measures Euclidean 3D distance and stores native entities, not ACIS/B-Rep constraints.");
    });
    for (i,v) in values.into_iter().enumerate(){ctx.data_mut(|d|d.insert_temp(id(&format!("v{i}")),v));}
    action
}

fn leader3d_dialog(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    if app.session.doc().is_err() { no_drawing(ctx, "3D Leader", open); return None; }
    let id=|n:&str| egui::Id::new(("leader3d_dialog",n));
    let mut values:Vec<String>=(0..9).map(|i|ctx.data_mut(|d|d.get_temp::<String>(id(&format!("v{i}")))).unwrap_or_else(||if i==3{"10".into()}else if i==6{"3".into()}else{"0".into()})).collect();
    let mut text=ctx.data_mut(|d|d.get_temp::<String>(id("text"))).unwrap_or_else(||"3D Leader".into());
    let parse=|v:&str|v.trim().parse::<f64>().ok().filter(|v|v.is_finite());
    let nums:Vec<_>=values.iter().map(|v|parse(v)).collect();
    let mut action=None;
    egui::Window::new("3D Leader").open(open).default_width(470.0).resizable(false).show(ctx,|ui|{
        ui.label("Creates a projected WCS 3D leader line and a screen-facing text label.");
        egui::Grid::new("leader3d_grid").num_columns(4).show(ui,|ui|{
            ui.label("");ui.label("X");ui.label("Y");ui.label("Z");ui.end_row();
            for (row,base) in [("Arrow",0usize),("Landing",3usize),("Dogleg",6usize)]{ui.label(row);for i in 0..3{ui.text_edit_singleline(&mut values[base+i]);}ui.end_row();}
            ui.label("Text");ui.text_edit_singleline(&mut text);ui.end_row();
        });
        let ok=nums.iter().all(Option::is_some)&&!text.trim().is_empty();
        if ui.add_enabled(ok,egui::Button::new("Create 3D leader")).clicked(){action=Some(("leader3d",json!({"arrow":[nums[0],nums[1],nums[2]],"landing":[nums[3],nums[4],nums[5]],"dogleg":[nums[6],nums[7],nums[8]],"text":text.trim()})));}
    });
    for (i,v) in values.into_iter().enumerate(){ctx.data_mut(|d|d.insert_temp(id(&format!("v{i}")),v));}
    ctx.data_mut(|d|d.insert_temp(id("text"),text));
    action
}

fn dim3d_angle_dialog(app:&mut CadApp,ctx:&egui::Context,open:&mut bool)->Action{
    if app.session.doc().is_err(){no_drawing(ctx,"3D Angular Dimension",open);return None;}
    let id=|n:&str|egui::Id::new(("dim3d_angle",n)); let defaults=["0","0","0","10","0","0","0","10","0","5"];
    let mut v:Vec<String>=defaults.iter().enumerate().map(|(i,d)|ctx.data_mut(|x|x.get_temp::<String>(id(&format!("v{i}")))).unwrap_or_else(||(*d).into())).collect();
    let nums:Vec<_>=v.iter().map(|x|x.trim().parse::<f64>().ok().filter(|n|n.is_finite())).collect(); let mut action=None;
    egui::Window::new("3D Angular Dimension").open(open).default_width(520.0).show(ctx,|ui|{
        egui::Grid::new("dim3da").num_columns(4).show(ui,|ui|{ui.label("");ui.label("X");ui.label("Y");ui.label("Z");ui.end_row();for (row,base) in [("Vertex",0usize),("Ray A",3),("Ray B",6)]{ui.label(row);for i in 0..3{ui.text_edit_singleline(&mut v[base+i]);}ui.end_row();}ui.label("Offset");ui.text_edit_singleline(&mut v[9]);ui.end_row();});
        if ui.add_enabled(nums.iter().all(Option::is_some),egui::Button::new("Create 3D angular dimension")).clicked(){action=Some(("dim3d.angle",json!({"vertex":[nums[0],nums[1],nums[2]],"a":[nums[3],nums[4],nums[5]],"b":[nums[6],nums[7],nums[8]],"offset":nums[9]})));}
    }); for(i,x)in v.into_iter().enumerate(){ctx.data_mut(|d|d.insert_temp(id(&format!("v{i}")),x));} action
}

fn dim3d_radius_dialog(app:&mut CadApp,ctx:&egui::Context,open:&mut bool)->Action{
    if app.session.doc().is_err(){no_drawing(ctx,"3D Radius Dimension",open);return None;}
    let id=|n:&str|egui::Id::new(("dim3d_radius",n)); let defaults=["0","0","0","5","0","0"];
    let mut v:Vec<String>=defaults.iter().enumerate().map(|(i,d)|ctx.data_mut(|x|x.get_temp::<String>(id(&format!("v{i}")))).unwrap_or_else(||(*d).into())).collect();
    let nums:Vec<_>=v.iter().map(|x|x.trim().parse::<f64>().ok().filter(|n|n.is_finite())).collect(); let mut action=None;
    egui::Window::new("3D Radius Dimension").open(open).default_width(480.0).show(ctx,|ui|{egui::Grid::new("dim3dr").num_columns(4).show(ui,|ui|{ui.label("");ui.label("X");ui.label("Y");ui.label("Z");ui.end_row();for(row,base)in[("Center",0usize),("Point",3)]{ui.label(row);for i in 0..3{ui.text_edit_singleline(&mut v[base+i]);}ui.end_row();}});if ui.add_enabled(nums.iter().all(Option::is_some),egui::Button::new("Create 3D radius dimension")).clicked(){action=Some(("dim3d.radius",json!({"center":[nums[0],nums[1],nums[2]],"point":[nums[3],nums[4],nums[5]]})));}}); for(i,x)in v.into_iter().enumerate(){ctx.data_mut(|d|d.insert_temp(id(&format!("v{i}")),x));} action
}

fn visual_style(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(d) = app.session.doc() else { no_drawing(ctx, "Visual Styles", open); return None; };
    let current = d.header.str("VISUALSTYLE", "2DWireframe");
    let styles = ["2DWireframe", "Wireframe", "Hidden", "Conceptual", "Realistic", "Shaded", "ShadedWithEdges", "Sketchy", "X-ray"];
    let mut action = None;
    egui::Window::new("Visual Styles").open(open).default_width(360.0).resizable(false).show(ctx, |ui| {
        ui.strong(format!("Current: {current}"));
        ui.separator();
        for style in styles {
            if ui.selectable_label(style.eq_ignore_ascii_case(&current), style).clicked() {
                action = Some(("visualstyle", json!({"name":style})));
            }
        }
        ui.separator();
        ui.small("The 3DFACE renderer consumes this style: wireframe styles draw edges, shaded styles fill facets, and ShadedWithEdges keeps both. Realistic currently uses the same faceted light model (no texture maps yet).");
    });
    action
}

fn material_dialog(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(d)=app.session.doc() else { no_drawing(ctx,"Material",open); return None; };
    let id=|n:&str| egui::Id::new(("material_dialog",n));
    let mut name=ctx.data_mut(|x| x.get_temp::<String>(id("name"))).unwrap_or_else(|| d.header.str("MATERIALCURRENT","Default").to_string());
    let mut color=ctx.data_mut(|x| x.get_temp::<String>(id("color"))).unwrap_or_else(|| d.header.str("MATERIALCOLOR","#b8c0cc").to_string());
    let mut rough=ctx.data_mut(|x| x.get_temp::<f64>(id("rough"))).unwrap_or_else(|| d.header.f64("MATERIALROUGHNESS",0.55));
    let mut metal=ctx.data_mut(|x| x.get_temp::<f64>(id("metal"))).unwrap_or_else(|| d.header.f64("MATERIALMETALLIC",0.0));
    let mut action=None;
    egui::Window::new("Material").open(open).default_width(390.0).resizable(false).show(ctx,|ui|{
        egui::Grid::new("material_grid").num_columns(2).show(ui,|ui|{
            ui.label("Name"); ui.text_edit_singleline(&mut name); ui.end_row();
            ui.label("Color (#RRGGBB)"); ui.text_edit_singleline(&mut color); ui.end_row();
            ui.label("Roughness"); ui.add(egui::Slider::new(&mut rough,0.0..=1.0)); ui.end_row();
            ui.label("Metallic"); ui.add(egui::Slider::new(&mut metal,0.0..=1.0)); ui.end_row();
        });
        if ui.button("Apply to 3D viewport").clicked(){ action=Some(("material",json!({"name":name,"color":color,"roughness":rough,"metallic":metal}))); }
        ui.small("Material is currently a document-level faceted viewport material. Per-face UV/PBR maps remain a later renderer task.");
    });
    ctx.data_mut(|d|{d.insert_temp(id("name"),name);d.insert_temp(id("color"),color);d.insert_temp(id("rough"),rough);d.insert_temp(id("metal"),metal);});
    action
}

fn light_dialog(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(d)=app.session.doc() else { no_drawing(ctx,"Directional Light",open); return None; };
    let id=|n:&str| egui::Id::new(("light_dialog",n));
    let mut x=ctx.data_mut(|v|v.get_temp::<f64>(id("x"))).unwrap_or_else(||d.header.f64("LIGHTDIRX",0.4));
    let mut y=ctx.data_mut(|v|v.get_temp::<f64>(id("y"))).unwrap_or_else(||d.header.f64("LIGHTDIRY",-0.6));
    let mut z=ctx.data_mut(|v|v.get_temp::<f64>(id("z"))).unwrap_or_else(||d.header.f64("LIGHTDIRZ",0.7));
    let mut intensity=ctx.data_mut(|v|v.get_temp::<f64>(id("i"))).unwrap_or_else(||d.header.f64("LIGHTINTENSITY",1.0));
    let mut action=None;
    egui::Window::new("Directional Light").open(open).default_width(390.0).resizable(false).show(ctx,|ui|{
        egui::Grid::new("light_grid").num_columns(2).show(ui,|ui|{
            for (label,v) in [("X",&mut x),("Y",&mut y),("Z",&mut z)] { ui.label(label); ui.add(egui::DragValue::new(v).speed(0.05)); ui.end_row(); }
            ui.label("Intensity"); ui.add(egui::Slider::new(&mut intensity,0.0..=3.0)); ui.end_row();
        });
        if ui.button("Apply light").clicked(){ action=Some(("light",json!({"direction":[x,y,z],"intensity":intensity}))); }
        ui.small("A single persistent directional light shades 3DFACE geometry in shaded visual styles.");
    });
    ctx.data_mut(|d|{d.insert_temp(id("x"),x);d.insert_temp(id("y"),y);d.insert_temp(id("z"),z);d.insert_temp(id("i"),intensity);});
    action
}

fn render_dialog(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(d)=app.session.doc() else { no_drawing(ctx,"Render Preview",open); return None; };
    let vs=d.header.str("VISUALSTYLE","2DWireframe").to_string();
    let mat=d.header.str("MATERIALCURRENT","Default").to_string();
    let col=d.header.str("MATERIALCOLOR","#b8c0cc").to_string();
    egui::Window::new("Render Preview").open(open).default_width(420.0).resizable(false).show(ctx,|ui|{
        ui.strong("Viewport Faceted Render");
        ui.label(format!("Visual style: {vs}"));
        ui.label(format!("Material: {mat}  {col}"));
        ui.label(format!("Light: [{:.2}, {:.2}, {:.2}]  intensity {:.2}",d.header.f64("LIGHTDIRX",0.4),d.header.f64("LIGHTDIRY",-0.6),d.header.f64("LIGHTDIRZ",0.7),d.header.f64("LIGHTINTENSITY",1.0)));
        ui.separator();
        ui.small("This is a live viewport renderer for faceted geometry, not an offline ray tracer and not ACIS/B-Rep rendering.");
    });
    None
}

// ---------- UCS Manager ----------

/// Named UCS manager. The drawing table is the single source of truth; every mutation goes
/// through the engine `ucs` command so undo/save/reopen semantics stay centralized.
fn ucs_manager(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "UCS Manager", open);
        return None;
    };
    let current = d.header.str("UCSNAME", "World");
    let mut rows: Vec<_> = d.ucss.iter().map(|u| (u.name.clone(), u.origin, u.x_axis, u.y_axis)).collect();
    if !rows.iter().any(|(name, ..)| name.eq_ignore_ascii_case("World")) {
        rows.insert(0, ("World".into(), cadcraft_engine::geom::Vec3::new(0.0, 0.0, 0.0), cadcraft_engine::geom::Vec3::new(1.0, 0.0, 0.0), cadcraft_engine::geom::Vec3::new(0.0, 1.0, 0.0)));
    }
    let mut action = None;
    egui::Window::new("UCS Manager").open(open).default_width(620.0).resizable(true).show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.strong(format!("Current: {current}"));
            if ui.button("World").clicked() {
                action = Some(("ucs", json!({"mode":"world"})));
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
            egui::Grid::new("ucs_rows").striped(true).num_columns(5).show(ui, |ui| {
                ui.strong("Name"); ui.strong("Origin"); ui.strong("X axis"); ui.strong("Y axis"); ui.strong("Actions"); ui.end_row();
                for (name, origin, x, y) in &rows {
                    ui.label(name);
                    ui.label(format!("{:.3}, {:.3}, {:.3}", origin.x, origin.y, origin.z));
                    ui.label(format!("{:.3}, {:.3}, {:.3}", x.x, x.y, x.z));
                    ui.label(format!("{:.3}, {:.3}, {:.3}", y.x, y.y, y.z));
                    ui.horizontal(|ui| {
                        if !name.eq_ignore_ascii_case(&current) && ui.small_button("Set").clicked() {
                            action = Some(("ucs", json!({"mode":"set", "name":name})));
                        }
                        if !name.eq_ignore_ascii_case("World") && ui.small_button("Delete").clicked() {
                            action = Some(("ucs", json!({"mode":"delete", "name":name})));
                        }
                    });
                    ui.end_row();
                }
            });
        });
        ui.separator();
        ui.strong("Create / replace named UCS");
        let name_id = egui::Id::new("ucs_new_name");
        let origin_id = egui::Id::new("ucs_new_origin");
        let x_id = egui::Id::new("ucs_new_x");
        let y_id = egui::Id::new("ucs_new_y");
        let mut name = ctx.data_mut(|d| d.get_temp::<String>(name_id)).unwrap_or_default();
        let mut origin = ctx.data_mut(|d| d.get_temp::<String>(origin_id)).unwrap_or_else(|| "0,0,0".into());
        let mut x_axis = ctx.data_mut(|d| d.get_temp::<String>(x_id)).unwrap_or_else(|| "1,0,0".into());
        let mut y_axis = ctx.data_mut(|d| d.get_temp::<String>(y_id)).unwrap_or_else(|| "0,1,0".into());
        egui::Grid::new("ucs_create").num_columns(2).show(ui, |ui| {
            ui.label("Name"); ui.text_edit_singleline(&mut name); ui.end_row();
            ui.label("Origin X,Y,Z"); ui.text_edit_singleline(&mut origin); ui.end_row();
            ui.label("X axis X,Y,Z"); ui.text_edit_singleline(&mut x_axis); ui.end_row();
            ui.label("Y axis X,Y,Z"); ui.text_edit_singleline(&mut y_axis); ui.end_row();
        });
        ctx.data_mut(|d| { d.insert_temp(name_id, name.clone()); d.insert_temp(origin_id, origin.clone()); d.insert_temp(x_id, x_axis.clone()); d.insert_temp(y_id, y_axis.clone()); });
        let parse = |s: &str| -> Option<Vec<f64>> {
            let v: Vec<_> = s.split(',').map(|x| x.trim().parse::<f64>()).collect::<Result<_, _>>().ok()?;
            (v.len() == 3 && v.iter().all(|x| x.is_finite())).then_some(v)
        };
        let valid = !name.trim().is_empty() && parse(&origin).is_some() && parse(&x_axis).is_some() && parse(&y_axis).is_some();
        if ui.add_enabled(valid, egui::Button::new("Save and make current")).clicked() {
            if let (Some(o), Some(x), Some(y)) = (parse(&origin), parse(&x_axis), parse(&y_axis)) {
                action = Some(("ucs", json!({"mode":"save", "name":name.trim(), "origin":o, "xAxis":x, "yAxis":y})));
            }
        }
        ui.small("Named UCS definitions persist in the drawing. 2D drawing input is still WCS until the UCS transform pipeline is completed.");
    });
    action
}

// ---------- Linetype Manager ----------

/// Linetype Manager: current linetype and global scale, loaded linetypes, and load/current actions; returns the command to run after drawing.
fn linetype(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let t = Tokens::get();
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "Linetype Manager", open);
        return None;
    };
    let current = d.header.str("CELTYPE", "ByLayer");
    let ltscale = d.header.f64("LTSCALE", 1.0);
    let mut rows: Vec<(String, String, Vec<f64>)> =
        vec![("ByLayer".into(), String::new(), Vec::new()), ("ByBlock".into(), String::new(), Vec::new())];
    rows.extend(
        d.linetypes
            .iter()
            .filter(|l| !l.name.eq_ignore_ascii_case("ByLayer") && !l.name.eq_ignore_ascii_case("ByBlock"))
            .map(|l| (l.name.clone(), l.description.clone(), l.pattern.iter().map(|e| e.length).collect())),
    );
    let library: Vec<String> = cadcraft_doc::library::standard_linetypes().into_iter().map(|l| l.name).filter(|n| d.linetype(n).is_none()).collect();
    let mut sel = temp::<String>(ctx, "ltmgr_sel").filter(|s| rows.iter().any(|r| r.0.eq_ignore_ascii_case(s))).unwrap_or_else(|| current.clone());
    let mut action = None;
    egui::Window::new("Linetype Manager").open(open).default_size(vec2(520.0, 360.0)).resizable(true).show(ctx, |ui| {
        ui.horizontal(|ui| {
            craft_ui::ComboBox::from_id_salt("ltmgr_load").selected_text("Load...").width(110.0).show_ui(ui, |ui| {
                if library.is_empty() {
                    ui.label("Every library linetype is loaded.");
                } else if ui.selectable_label(false, "All").clicked() {
                    action = Some(("linetype", json!({ "load": "*" })));
                }
                for n in &library {
                    if ui.selectable_label(false, n).clicked() {
                        action = Some(("linetype", json!({ "load": n })));
                    }
                }
                // The user's own .lin file (where the host has a file picker).
                if let Some(pick) = app.services.pick_open.as_ref() {
                    ui.separator();
                    if ui.selectable_label(false, "From File...").clicked()
                        && let Some(path) = pick()
                    {
                        action = Some(("linetype", json!({ "load": "*", "file": path })));
                    }
                }
            });
            let is_current = sel.eq_ignore_ascii_case(&current);
            if ui.add_enabled(!is_current, egui::Button::new("Current")).on_hover_text("Draw new objects with the selected linetype").clicked() {
                action = Some(("linetype", json!({ "current": sel })));
            }
        });
        ui.label(RichText::new(format!("Current linetype: {current}")).color(t.text_dim));
        ui.separator();
        egui::ScrollArea::vertical().max_height(240.0).auto_shrink([false, true]).show(ui, |ui| {
            egui::Grid::new("ltmgr_grid").striped(true).num_columns(3).spacing(vec2(16.0, 4.0)).show(ui, |ui| {
                for h in ["Linetype", "Appearance", "Description"] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for (name, desc, pattern) in &rows {
                    let mut label = RichText::new(name);
                    if name.eq_ignore_ascii_case(&current) {
                        label = label.strong();
                    }
                    let r = ui.selectable_label(sel.eq_ignore_ascii_case(name), label);
                    if r.clicked() {
                        sel = name.clone();
                    }
                    if r.double_clicked() {
                        action = Some(("linetype", json!({ "current": name })));
                    }
                    let (rect, _) = ui.allocate_exact_size(vec2(120.0, 14.0), Sense::hover());
                    if !matches!(name.as_str(), "ByLayer" | "ByBlock") {
                        draw_pattern(ui.painter(), rect, pattern, Stroke::new(1.5, t.text));
                    }
                    ui.label(RichText::new(desc).color(t.text_dim));
                    ui.end_row();
                }
            });
        });
        ui.separator();
        ui.horizontal(|ui| {
            let label = ui.label("Global scale factor");
            if let Some(v) = num_field(ui, "ltmgr_scale", ltscale, Some(label.id)).filter(|v| *v > 0.0) {
                action = Some(("ltscale", json!({ "scale": v })));
            }
        });
    });
    set_temp(ctx, "ltmgr_sel", sel);
    action
}

/// A linetype sample: dashes (positive lengths), gaps (negative) and dots (zero), two repeats.
fn draw_pattern(p: &egui::Painter, rect: egui::Rect, pattern: &[f64], stroke: Stroke) {
    let y = rect.center().y;
    let total: f64 = pattern.iter().map(|l| l.abs()).sum();
    if pattern.is_empty() || !total.is_finite() || total <= 0.0 {
        p.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], stroke);
        return;
    }
    let k = f64::from(rect.width()) / (total * 2.0);
    let mut x = f64::from(rect.left());
    for l in pattern.iter().cycle().take(400) {
        if x >= f64::from(rect.right()) {
            break;
        }
        let len = l.abs() * k;
        if *l > 0.0 {
            let end = (x + len).min(f64::from(rect.right()));
            p.line_segment([egui::pos2(x as f32, y), egui::pos2(end as f32, y)], stroke);
        } else if *l == 0.0 {
            p.circle_filled(egui::pos2(x as f32, y), stroke.width, stroke.color);
        }
        x += len.max(1.0);
    }
}

// ---------- Point Style ----------

/// PDMODE values in dialog order: four rows of five.
const PDMODES: [i64; 20] = [0, 1, 2, 3, 4, 32, 33, 34, 35, 36, 64, 65, 66, 67, 68, 96, 97, 98, 99, 100];

/// The point glyph for a PDMODE, as drawn by the point style tiles.
fn draw_point(p: &egui::Painter, c: egui::Pos2, r: f32, mode: i64, stroke: Stroke) {
    let line = |a: egui::Vec2, b: egui::Vec2| p.line_segment([c + a, c + b], stroke);
    match mode & 7 {
        0 => {
            p.circle_filled(c, stroke.width, stroke.color);
        }
        2 => {
            line(vec2(-r, 0.0), vec2(r, 0.0));
            line(vec2(0.0, -r), vec2(0.0, r));
        }
        3 => {
            line(vec2(-r, -r), vec2(r, r));
            line(vec2(-r, r), vec2(r, -r));
        }
        4 => {
            line(vec2(0.0, 0.0), vec2(0.0, -r));
        }
        _ => {}
    }
    if mode & 32 != 0 {
        p.circle_stroke(c, r * 0.7, stroke);
    }
    if mode & 64 != 0 {
        p.rect_stroke(egui::Rect::from_center_size(c, vec2(r * 1.4, r * 1.4)), 0.0, stroke, egui::StrokeKind::Middle);
    }
}

/// Point Style dialog for the PDMODE/PDSIZE presets; returns the `ddptype` action.
fn point_style(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let t = Tokens::get();
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "Point Style", open);
        return None;
    };
    let (mode, pdsize) = (d.header.i64("PDMODE", 0), d.header.f64("PDSIZE", 0.0));
    // PDSIZE 0 is 5% of the screen, negative a percentage of the screen, positive drawing units.
    let rel = pdsize <= 0.0;
    let size = if pdsize == 0.0 { 5.0 } else { pdsize.abs() };
    let set = |m: i64, size: f64, rel: bool| Some(("ddptype", json!({ "pdmode": m, "pdsize": if rel { -size } else { size } })));
    let mut action = None;
    egui::Window::new("Point Style").open(open).collapsible(false).resizable(false).show(ctx, |ui| {
        egui::Grid::new("ptype_grid").spacing(vec2(6.0, 6.0)).show(ui, |ui| {
            for (i, v) in PDMODES.iter().enumerate() {
                let (rect, r) = ui.allocate_exact_size(vec2(44.0, 36.0), Sense::click());
                let fill = if *v == mode {
                    t.accent
                } else if r.hovered() {
                    t.control_hover
                } else {
                    t.control
                };
                ui.painter().rect_filled(rect, 3.0, fill);
                draw_point(ui.painter(), rect.center(), 9.0, *v, Stroke::new(1.5, t.text));
                if r.on_hover_text(format!("PDMODE {v}")).clicked() && *v != mode {
                    action = set(*v, size, rel);
                }
                if i % 5 == 4 {
                    ui.end_row();
                }
            }
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let label = ui.label("Point Size");
            if let Some(v) = num_field(ui, "ptype_size", size, Some(label.id)).filter(|v| *v > 0.0) {
                action = set(mode, v, rel);
            }
            ui.label(if rel { "%" } else { "units" });
        });
        if ui.radio(rel, "Set Size Relative to Screen").clicked() && !rel {
            action = set(mode, size, true);
        }
        if ui.radio(!rel, "Set Size in Absolute Units").clicked() && rel {
            action = set(mode, size, false);
        }
    });
    action
}

// ---------- Dimension Style Manager ----------

/// A practical DIMSTYLE manager over the drawing's real dimension-style table.
///
/// The engine already owns creation/current/delete and field validation.  This dialog only reads
/// a snapshot and routes every edit back through those commands, so changes stay undoable and
/// scriptable instead of creating a second UI-only style model.
fn dimension_style(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "Dimension Style Manager", open);
        return None;
    };
    let current = d.header.str("DIMSTYLE", "Standard");
    let styles = d.dim_styles.clone();
    if styles.is_empty() {
        return None;
    }
    let mut selected = temp::<String>(ctx, "dimstyle_sel")
        .filter(|n| styles.iter().any(|st| st.name.eq_ignore_ascii_case(n)))
        .unwrap_or_else(|| current.clone());
    if !styles.iter().any(|st| st.name.eq_ignore_ascii_case(&selected)) {
        selected = styles[0].name.clone();
    }
    let style = styles
        .iter()
        .find(|st| st.name.eq_ignore_ascii_case(&selected))
        .cloned()
        .unwrap_or_else(|| styles[0].clone());
    let mut action = None;
    egui::Window::new("Dimension Style Manager")
        .open(open)
        .default_size(vec2(610.0, 430.0))
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(180.0);
                    ui.label(RichText::new("Styles").strong());
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        for st in &styles {
                            let label = if st.name.eq_ignore_ascii_case(&current) {
                                format!("{}  (current)", st.name)
                            } else {
                                st.name.clone()
                            };
                            if ui.selectable_label(st.name.eq_ignore_ascii_case(&selected), label).clicked() {
                                selected = st.name.clone();
                            }
                        }
                    });
                    ui.separator();
                    if ui.button("Set Current").clicked() && !selected.eq_ignore_ascii_case(&current) {
                        action = Some(("dimstyle.current", json!({ "name": selected })));
                    }
                    if ui.button("New from Current").clicked() {
                        let mut n = 1usize;
                        let name = loop {
                            let candidate = format!("TBT360-{n}");
                            if !styles.iter().any(|st| st.name.eq_ignore_ascii_case(&candidate)) {
                                break candidate;
                            }
                            n += 1;
                        };
                        action = Some(("dimstyle", json!({ "name": name, "current": false })));
                    }
                    let deletable = !selected.eq_ignore_ascii_case("Standard") && !selected.eq_ignore_ascii_case(&current);
                    if ui.add_enabled(deletable, egui::Button::new("Delete")).clicked() {
                        action = Some(("dimstyle.delete", json!({ "name": selected })));
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.heading(&style.name);
                    ui.label("Core dimension appearance and placement");
                    egui::Grid::new("dimstyle_core").num_columns(2).spacing(vec2(12.0, 7.0)).show(ui, |ui| {
                        let label = ui.label("Overall scale (DIMSCALE)");
                        if let Some(v) = num_field(ui, "dimstyle_scale", style.scale, Some(label.id)).filter(|v| *v >= 0.0) {
                            action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMSCALE": v })));
                        }
                        ui.end_row();
                        let label = ui.label("Arrow size (DIMASZ)");
                        if let Some(v) = num_field(ui, "dimstyle_arrow", style.arrow_size, Some(label.id)).filter(|v| *v >= 0.0) {
                            action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMASZ": v })));
                        }
                        ui.end_row();
                        let label = ui.label("Text height (DIMTXT)");
                        if let Some(v) = num_field(ui, "dimstyle_text", style.text_height, Some(label.id)).filter(|v| *v > 0.0) {
                            action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMTXT": v })));
                        }
                        ui.end_row();
                        let label = ui.label("Extension offset (DIMEXO)");
                        if let Some(v) = num_field(ui, "dimstyle_exo", style.ext_offset, Some(label.id)).filter(|v| *v >= 0.0) {
                            action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMEXO": v })));
                        }
                        ui.end_row();
                        let label = ui.label("Extension beyond (DIMEXE)");
                        if let Some(v) = num_field(ui, "dimstyle_exe", style.ext_extend, Some(label.id)).filter(|v| *v >= 0.0) {
                            action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMEXE": v })));
                        }
                        ui.end_row();
                        let label = ui.label("Baseline spacing (DIMDLI)");
                        if let Some(v) = num_field(ui, "dimstyle_dli", style.baseline_spacing, Some(label.id)).filter(|v| *v >= 0.0) {
                            action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMDLI": v })));
                        }
                        ui.end_row();
                        let mut decimals = i32::from(style.decimals);
                        ui.label("Precision (DIMDEC)");
                        if ui.add(egui::DragValue::new(&mut decimals).range(0..=8)).changed() {
                            action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMDEC": decimals })));
                        }
                        ui.end_row();
                    });
                    ui.separator();
                    let mut annotative = style.annotative;
                    if ui.checkbox(&mut annotative, "Annotative").changed() {
                        action = Some(("dimstyle", json!({ "name": style.name, "current": false, "annotative": annotative })));
                    }
                    let mut tolerance = style.tolerance;
                    if ui.checkbox(&mut tolerance, "Show tolerances (DIMTOL)").changed() {
                        action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMTOL": tolerance })));
                    }
                    let mut alt = style.alt;
                    if ui.checkbox(&mut alt, "Alternate units (DIMALT)").changed() {
                        action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMALT": alt })));
                    }
                    ui.separator();
                    ui.label(RichText::new("Advanced text / tolerance").strong());
                    egui::Grid::new("dimstyle_advanced").num_columns(2).spacing(vec2(12.0, 7.0)).show(ui, |ui| {
                        ui.label("Alt tolerance precision (DIMALTTD)");
                        let mut alt_tol_dec = i32::from(style.alt_tol_decimals);
                        if ui.add(egui::DragValue::new(&mut alt_tol_dec).range(0..=8)).changed() {
                            action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMALTTD": alt_tol_dec })));
                        }
                        ui.end_row();
                        ui.label("Alt tolerance zero suppression (DIMALTTZ)");
                        let mut alt_tol_zero = i32::from(style.alt_tol_zero_suppression);
                        if ui.add(egui::DragValue::new(&mut alt_tol_zero).range(0..=15)).changed() {
                            action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMALTTZ": alt_tol_zero })));
                        }
                        ui.end_row();
                    });
                    let mut user_positioned = style.user_positioned_text;
                    if ui.checkbox(&mut user_positioned, "Keep user-positioned text (DIMUPT)").changed() {
                        action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMUPT": user_positioned })));
                    }
                    let mut rtl = style.text_direction;
                    if ui.checkbox(&mut rtl, "Right-to-left dimension text (DIMTXTDIRECTION)").changed() {
                        action = Some(("dimstyle", json!({ "name": style.name, "current": false, "DIMTXTDIRECTION": rtl })));
                    }
                    ui.add_space(8.0);
                    ui.label("All edits use the engine DIMSTYLE command and its validation; unsupported values are not stored.");
                });
            });
        });
    set_temp(ctx, "dimstyle_sel", selected);
    action
}


// ---------- Geometric Tolerance ----------

#[derive(Clone, Debug)]
struct ToleranceDialogState {
    characteristic: String,
    tol1: String,
    tol2: String,
    datums: [String; 3],
}

impl Default for ToleranceDialogState {
    fn default() -> Self {
        Self {
            characteristic: "Position".into(),
            tol1: "%%c0.1(M)".into(),
            tol2: String::new(),
            datums: ["A".into(), "B".into(), String::new()],
        }
    }
}

/// Practical feature-control-frame dialog. The dialog gathers the same values as the engine's
/// interactive TOLERANCE machine, then leaves that machine at its final point prompt so placement
/// still happens directly on the drawing canvas.
fn tolerance_dialog(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    if app.session.doc().is_err() {
        no_drawing(ctx, "Geometric Tolerance", open);
        return None;
    }
    let mut st = temp::<ToleranceDialogState>(ctx, "tolerance_dialog_state").unwrap_or_default();
    let mut action = None;
    const CHARACTERISTICS: &[&str] = &[
        "Position",
        "Concentricity",
        "Symmetry",
        "Parallelism",
        "Perpendicularity",
        "Angularity",
        "Cylindricity",
        "Flatness",
        "Circularity",
        "Straightness",
        "SurfaceProfile",
        "LineProfile",
        "CircularRunout",
        "TotalRunout",
        "None",
    ];
    egui::Window::new("Geometric Tolerance")
        .open(open)
        .default_size(vec2(470.0, 330.0))
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Build a feature control frame, then place it directly on the drawing.");
            ui.separator();
            egui::Grid::new("tolerance_dialog_grid").num_columns(2).spacing(vec2(12.0, 8.0)).show(ui, |ui| {
                ui.label("Characteristic");
                craft_ui::ComboBox::from_id_salt("tolerance_characteristic")
                    .selected_text(&st.characteristic)
                    .show_ui(ui, |ui| {
                        for c in CHARACTERISTICS {
                            if ui.selectable_label(st.characteristic.eq_ignore_ascii_case(c), *c).clicked() {
                                st.characteristic = (*c).to_string();
                            }
                        }
                    });
                ui.end_row();
                ui.label("Tolerance 1");
                ui.text_edit_singleline(&mut st.tol1).on_hover_text("Examples: 0.1, %%c0.1(M), 0.02(L)");
                ui.end_row();
                ui.label("Tolerance 2");
                ui.text_edit_singleline(&mut st.tol2);
                ui.end_row();
                for i in 0..3 {
                    ui.label(format!("Datum {}", i + 1));
                    ui.text_edit_singleline(&mut st.datums[i]).on_hover_text("Examples: A, B(M), C(L)");
                    ui.end_row();
                }
            });
            ui.separator();
            ui.label(RichText::new("Placement stays interactive: after Place, click the insertion point on the canvas; Esc cancels.").color(Tokens::get().text_dim));
            ui.horizontal(|ui| {
                if ui.button("Place on Drawing").clicked() {
                    action = Some((
                        "ui.tolerance.place",
                        json!({
                            "characteristic": st.characteristic.clone(),
                            "tol1": st.tol1.clone(),
                            "tol2": st.tol2.clone(),
                            "datums": st.datums.clone(),
                        }),
                    ));
                }
                if ui.button("Reset").clicked() {
                    st = ToleranceDialogState::default();
                }
            });
        });
    set_temp(ctx, "tolerance_dialog_state", st);
    action
}

// ---------- Text Style, Table Style and Multileader Style ----------

/// One editable style property.
enum Field {
    Num(&'static str, &'static str, f64),
    Bool(&'static str, &'static str, bool),
    Choice(&'static str, &'static str, String, Vec<String>),
}

struct Style {
    name: String,
    fields: Vec<Field>,
}

fn text_style(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "Text Style", open);
        return None;
    };
    let mut styles: Vec<Style> = d
        .text_styles
        .iter()
        .map(|s| Style {
            name: s.name.clone(),
            fields: vec![
                Field::Num("height", "Height (0 = ask)", s.height),
                Field::Num("widthFactor", "Width factor", s.width_factor),
                Field::Bool("backwards", "Backwards", s.backwards),
                Field::Bool("upsideDown", "Upside down", s.upside_down),
            ],
        })
        .collect();
    if styles.is_empty() {
        let s = cadcraft_doc::TextStyle::default();
        styles.push(Style { name: s.name, fields: Vec::new() });
    }
    let current = d.header.str("TEXTSTYLE", "Standard");
    style_manager(ctx, open, "Text Style", "style", &current, &styles)
}

fn table_style(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "Table Style", open);
        return None;
    };
    let mut styles: Vec<Style> = d
        .table_styles
        .iter()
        .map(|s| Style {
            name: s.name.clone(),
            fields: vec![
                Field::Num("textHeight", "Text height", s.text_height),
                Field::Num("margin", "Cell margin", s.margin),
                Field::Bool("title", "Title row", s.title),
                Field::Bool("header", "Header row", s.header),
            ],
        })
        .collect();
    if styles.is_empty() {
        let s = cadcraft_doc::TableStyle::default();
        styles.push(Style { name: s.name, fields: Vec::new() });
    }
    let current = d.header.str("CTABLESTYLE", "Standard");
    style_manager(ctx, open, "Table Style", "tablestyle", &current, &styles)
}

fn mleader_style(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "Multileader Style", open);
        return None;
    };
    let text_styles: Vec<String> = d.text_styles.iter().map(|s| s.name.clone()).collect();
    let mut styles: Vec<Style> = d
        .mleader_styles
        .iter()
        .map(|s| Style {
            name: s.name.clone(),
            fields: vec![
                Field::Num("arrowSize", "Arrowhead size", s.arrow_size),
                Field::Num("textHeight", "Text height", s.text_height),
                Field::Num("landingGap", "Landing gap", s.landing_gap),
                Field::Num("dogleg", "Landing length", s.dogleg),
                Field::Choice("textStyle", "Text style", s.text_style.clone(), text_styles.clone()),
            ],
        })
        .collect();
    if styles.is_empty() {
        let s = cadcraft_doc::MLeaderStyle::default();
        styles.push(Style { name: s.name, fields: Vec::new() });
    }
    let current = d.header.str("CMLEADERSTYLE", "Standard");
    style_manager(ctx, open, "Multileader Style", "mleaderstyle", &current, &styles)
}

/// A style list (set current, new) beside the selected style's properties. `cmd` takes
/// `{name, <field>: value, current?}`.
fn style_manager(ctx: &egui::Context, open: &mut bool, title: &str, cmd: &'static str, current: &str, styles: &[Style]) -> Action {
    let t = Tokens::get();
    let sel_key = format!("{cmd}_sel");
    let new_key = format!("{cmd}_new");
    let mut sel = temp::<String>(ctx, &sel_key).filter(|s| styles.iter().any(|x| x.name == *s)).unwrap_or_else(|| current.to_string());
    let mut new_name = temp::<String>(ctx, &new_key).unwrap_or_default();
    let mut action = None;
    egui::Window::new(title).open(open).default_size(vec2(520.0, 280.0)).resizable(true).show(ctx, |ui| {
        ui.label(RichText::new(format!("Current style: {current}")).color(t.text_dim));
        ui.separator();
        ui.columns(2, |cols| {
            let [left, right] = cols else { return };
            left.label(RichText::new("Styles").strong());
            egui::ScrollArea::vertical().id_salt((cmd, "list")).max_height(160.0).auto_shrink([false, true]).show(left, |ui| {
                for s in styles {
                    let mut label = RichText::new(&s.name);
                    if s.name.eq_ignore_ascii_case(current) {
                        label = label.strong();
                    }
                    if ui.selectable_label(s.name == sel, label).clicked() {
                        sel = s.name.clone();
                    }
                }
            });
            if left.add_enabled(!sel.eq_ignore_ascii_case(current), egui::Button::new("Set Current")).clicked() {
                action = Some((cmd, json!({ "name": sel })));
            }
            left.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut new_name).hint_text("New style name").desired_width(120.0)).on_hover_text("New style name");
                let name = new_name.trim().to_string();
                let ok = !name.is_empty() && !styles.iter().any(|s| s.name.eq_ignore_ascii_case(&name));
                if ui.add_enabled(ok, egui::Button::new("New")).clicked() {
                    action = Some((cmd, json!({ "name": name, "current": false })));
                    sel = name;
                    new_name.clear();
                }
            });
            right.label(RichText::new(format!("Properties of {sel}")).strong());
            let Some(style) = styles.iter().find(|s| s.name == sel) else { return };
            egui::Grid::new((cmd, "props")).num_columns(2).spacing(vec2(10.0, 6.0)).show(right, |ui| {
                for f in &style.fields {
                    let change = match f {
                        Field::Num(key, label, v) => {
                            let field_label = ui.label(*label);
                            num_field(ui, (cmd, &style.name, *key), *v, Some(field_label.id)).filter(|v| *v >= 0.0).map(|v| (*key, json!(v)))
                        }
                        Field::Bool(key, label, v) => {
                            ui.label("");
                            let mut b = *v;
                            ui.checkbox(&mut b, *label).changed().then(|| (*key, json!(b)))
                        }
                        Field::Choice(key, label, v, choices) => {
                            ui.label(*label);
                            let mut picked = None;
                            craft_ui::ComboBox::from_id_salt((cmd, *key)).selected_text(v.as_str()).show_ui(ui, |ui| {
                                for c in choices {
                                    if ui.selectable_label(c == v, c).clicked() && c != v {
                                        picked = Some(c.clone());
                                    }
                                }
                            });
                            picked.map(|c| (*key, json!(c)))
                        }
                    };
                    if let Some((key, v)) = change {
                        action = Some((cmd, json!({ "name": style.name, key: v, "current": false })));
                    }
                    ui.end_row();
                }
            });
        });
    });
    set_temp(ctx, &sel_key, sel);
    set_temp(ctx, &new_key, new_name);
    action
}

// ---------- Page Setup Manager ----------

/// Page Setup Manager over layouts sorted by tab order; returns the selected setup command.
fn page_setup(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let t = Tokens::get();
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "Page Setup Manager", open);
        return None;
    };
    let mut layouts: Vec<(u32, String, cadcraft_doc::PageSetup)> = d.layouts.iter().map(|l| (l.tab_order, l.name.clone(), l.page.clone())).collect();
    layouts.sort_by_key(|l| l.0);
    let shown = app.session.layout_space().layout().map(str::to_string);
    let mut sel = temp::<String>(ctx, "pagesetup_sel")
        .filter(|s| layouts.iter().any(|l| l.1 == *s))
        .or(shown)
        .or_else(|| layouts.first().map(|l| l.1.clone()))
        .unwrap_or_default();
    let mut action = None;
    let choices = PageChoices::new(ctx, d);
    egui::Window::new("Page Setup Manager").open(open).default_size(vec2(560.0, 300.0)).resizable(true).show(ctx, |ui| {
        if layouts.is_empty() {
            ui.label("This drawing has no layouts.");
            return;
        }
        ui.columns(2, |cols| {
            let [left, right] = cols else { return };
            left.label(RichText::new("Layouts").strong());
            egui::ScrollArea::vertical().id_salt("pagesetup_list").max_height(200.0).auto_shrink([false, true]).show(left, |ui| {
                for (_, name, _) in &layouts {
                    if ui.selectable_label(*name == sel, name).clicked() {
                        sel = name.clone();
                    }
                }
            });
            let Some((_, name, page)) = layouts.iter().find(|l| l.1 == sel) else { return };
            right.label(RichText::new(format!("Page setup of {name}")).strong());
            // Layout plotting always prints the whole sheet, so only that area is offered.
            let rows = PageRows { areas: &["layout"], scale: true, center: true };
            egui::Grid::new("pagesetup_grid").num_columns(2).spacing(vec2(10.0, 6.0)).show(right, |ui| {
                if let Some((k, v)) = page_rows(ui, egui::Id::unique(("pagesetup", name)), page, &rows, &choices, None) {
                    action = Some(("pagesetup", json!({ "layout": name, k: v })));
                }
            });
            let (w, h) = if page.landscape { (page.height_mm, page.width_mm) } else { (page.width_mm, page.height_mm) };
            right.label(RichText::new(format!("{w:.1} × {h:.1} mm")).color(t.text_dim));
        });
    });
    set_temp(ctx, "pagesetup_sel", sel);
    action
}

/// Which page-setup rows a dialog shows.
struct PageRows {
    /// Plot area choices (`plotArea` values).
    areas: &'static [&'static str],
    scale: bool,
    center: bool,
}

/// The devices and plot style tables the page-setup rows offer.
struct PageChoices {
    devices: std::sync::Arc<Vec<cadcraft_doc::PlotterConfig>>,
    styles: Vec<String>,
}

impl PageChoices {
    fn new(ctx: &egui::Context, d: &cadcraft_doc::Drawing) -> Self {
        PageChoices { devices: device_choices(ctx), styles: cadcraft_doc::plot_style_table_names(d) }
    }
}

/// The plotter devices (built-in, then the user's files), read again at most every 2 s: the
/// dialogs ask every frame and user devices are files.
fn device_choices(ctx: &egui::Context) -> std::sync::Arc<Vec<cadcraft_doc::PlotterConfig>> {
    type Cached = (f64, std::sync::Arc<Vec<cadcraft_doc::PlotterConfig>>);
    let now = crate::now_ms();
    if let Some((at, list)) = temp::<Cached>(ctx, "plot_devices")
        && now - at < 2000.0
    {
        return list;
    }
    let list = std::sync::Arc::new(cadcraft_engine::cmd::plotters::devices());
    set_temp::<Cached>(ctx, "plot_devices", (now, list.clone()));
    list
}

/// A page setup's device, or `"None"` (plot to PDF on the page's paper).
fn device_label(page: &cadcraft_doc::PageSetup) -> &str {
    if page.device.trim().is_empty() { "None" } else { page.device.as_str() }
}

/// A page setup's plot style table, or `"None"`.
fn style_label(page: &cadcraft_doc::PageSetup) -> &str {
    if cadcraft_doc::no_plot_style_table(&page.plot_style_table) { "None" } else { page.plot_style_table.as_str() }
}

fn area_label(a: &str) -> &str {
    match a {
        "layout" => "Layout",
        "extents" => "Extents",
        "display" => "Display",
        "limits" => "Limits",
        "window" => "Window",
        other => other,
    }
}

/// A plot scale typed as a number (paper units per drawing unit) or a ratio such as `1:50`.
fn parse_ratio(t: &str) -> Option<f64> {
    let v = match t.trim().split_once([':', '/']) {
        Some((a, b)) => a.trim().parse::<f64>().ok()? / b.trim().parse::<f64>().ok()?,
        None => t.trim().trim_end_matches(['x', 'X']).parse().ok()?,
    };
    (v.is_finite() && v > 0.0).then_some(v)
}

/// The page-setup rows (device, paper, orientation, plot area, fit, scale, center, lineweights,
/// transparency, plot style table) shared by the Page Setup Manager and the Plot dialog, inside a
/// two-column grid. Returns the setting the user changed as a `pagesetup` parameter (`device`,
/// `paper`, `landscape`, `plotArea`, `scaleToFit`, `scale`, `center`, `lineweights`,
/// `plotTransparency` or `plotStyleTable`).
fn page_rows(
    ui: &mut egui::Ui,
    salt: egui::Id,
    page: &cadcraft_doc::PageSetup,
    rows: &PageRows,
    choices: &PageChoices,
    placement: Option<(&mut bool, &mut [f64; 2], bool)>,
) -> Option<(&'static str, Value)> {
    let mut change = None;
    ui.label("Printer/plotter");
    craft_ui::ComboBox::from_id_salt(salt.with("device")).selected_text(device_label(page)).width(190.0).show_ui(ui, |ui| {
        if ui.selectable_label(device_label(page) == "None", "None").on_hover_text("PDF on the paper chosen below").clicked()
            && device_label(page) != "None"
        {
            change = Some(("device", json!("None")));
        }
        for dev in choices.devices.iter() {
            let on = dev.name.eq_ignore_ascii_case(&page.device);
            if ui.selectable_label(on, &dev.name).on_hover_text(format!("{} ({})", dev.description, dev.output.as_str().to_uppercase())).clicked()
                && !on
            {
                change = Some(("device", json!(dev.name)));
            }
        }
    });
    ui.end_row();
    // The device's papers, else the standard sizes.
    let device = choices.devices.iter().find(|dv| dv.name.eq_ignore_ascii_case(&page.device));
    ui.label("Paper size");
    craft_ui::ComboBox::from_id_salt(salt.with("paper")).selected_text(page.paper.as_str()).width(190.0).show_ui(ui, |ui| match device {
        Some(dev) => {
            for p in &dev.papers {
                if ui.selectable_label(p.name == page.paper, &p.name).clicked() && p.name != page.paper {
                    change = Some(("paper", json!(p.name)));
                }
            }
        }
        None => {
            for p in cadcraft_render::PAPER_SIZES {
                if ui.selectable_label(p.name == page.paper, p.name).clicked() && p.name != page.paper {
                    change = Some(("paper", json!(p.name)));
                }
            }
        }
    });
    ui.end_row();
    ui.label("Orientation");
    ui.horizontal(|ui| {
        if ui.radio(!page.landscape, "Portrait").clicked() && page.landscape {
            change = Some(("landscape", json!(false)));
        }
        if ui.radio(page.landscape, "Landscape").clicked() && !page.landscape {
            change = Some(("landscape", json!(true)));
        }
    });
    ui.end_row();
    ui.label("Plot area");
    ui.add_enabled_ui(rows.areas.len() > 1, |ui| {
        craft_ui::ComboBox::from_id_salt(salt.with("area")).selected_text(area_label(&page.plot_area)).show_ui(ui, |ui| {
            for a in rows.areas {
                if ui.selectable_label(page.plot_area == *a, area_label(a)).clicked() && page.plot_area != *a {
                    change = Some(("plotArea", json!(a)));
                }
            }
        });
    });
    ui.end_row();
    ui.label("");
    let mut fit = page.scale_to_fit;
    if ui.checkbox(&mut fit, "Fit to paper").changed() {
        change = Some(("scaleToFit", json!(fit)));
    }
    ui.end_row();
    if let Some((center, offset_mm, supported)) = placement {
        ui.label("Plot offset");
        ui.add_enabled_ui(supported, |ui| ui.checkbox(center, "Center the plot")).response.on_hover_text(if supported {
            "Center the plotted model bounds on the printable area"
        } else {
            "Model-space offset placement is not supported by SVG"
        });
        ui.end_row();
        for (i, label) in [(0, "X offset (mm)"), (1, "Y offset (mm)")] {
            let field_label = ui.label(label);
            ui.add_enabled_ui(supported && !*center, |ui| {
                if let Some(v) = num_field(ui, ("plot_offset", i), offset_mm[i], Some(field_label.id)) {
                    offset_mm[i] = v;
                }
            })
            .response
            .on_hover_text("Distance from the printable area's lower-left corner; values range from −100000 to 100000 mm");
            ui.end_row();
        }
        if supported && !*center && offset_mm.iter().any(|v| !v.is_finite() || v.abs() > cadcraft_render::paper::MAX_PLOT_OFFSET_MM) {
            ui.label("");
            ui.colored_label(Tokens::get().warn, "Offsets must be within ±100000 mm");
            ui.end_row();
        }
    }
    if rows.scale {
        let label = ui.label("Scale");
        ui.add_enabled_ui(!page.scale_to_fit, |ui| {
            if let Some(v) = expr_field(ui, salt.with("scale"), &fmt_num(page.scale), Some(label.id)).and_then(|t| parse_ratio(&t)) {
                change = Some(("scale", json!(v)));
            }
        })
        .response
        .on_hover_text("Paper units per drawing unit, or a ratio such as 1:50");
        ui.end_row();
    }
    if rows.center {
        ui.label("");
        let mut center = page.center;
        if ui.checkbox(&mut center, "Center the plot").changed() {
            change = Some(("center", json!(center)));
        }
        ui.end_row();
    }
    ui.label("");
    let mut lw = page.lineweights;
    if ui.checkbox(&mut lw, "Plot object lineweights").changed() {
        change = Some(("lineweights", json!(lw)));
    }
    ui.end_row();
    ui.label("");
    let mut transparency = page.plot_transparency;
    if ui
        .checkbox(&mut transparency, "Plot transparency")
        .on_hover_text("Plot objects with their transparency (PLOTTRANSPARENCYOVERRIDE can override it)")
        .changed()
    {
        change = Some(("plotTransparency", json!(transparency)));
    }
    ui.end_row();
    ui.label("Plot style table");
    craft_ui::ComboBox::from_id_salt(salt.with("styles")).selected_text(style_label(page)).width(190.0).show_ui(ui, |ui| {
        if ui.selectable_label(style_label(page) == "None", "None").clicked() && style_label(page) != "None" {
            change = Some(("plotStyleTable", json!("None")));
        }
        for name in &choices.styles {
            let on = name.eq_ignore_ascii_case(&page.plot_style_table);
            if ui.selectable_label(on, name).clicked() && !on {
                change = Some(("plotStyleTable", json!(name)));
            }
        }
    });
    ui.end_row();
    change
}

/// Apply a [`page_rows`] change to a page setup held by a dialog.
/// Device and paper changes follow PAGESETUP's rules (`plotters::page_device`): a device brings
/// its papers, orientation and printable areas.
fn apply_page_change(page: &mut cadcraft_doc::PageSetup, key: &str, v: &Value) {
    use cadcraft_engine::cmd::plotters::page_device;
    match key {
        "device" => {
            let _ = page_device("plot", &json!({ "device": v }), page);
        }
        "paper" => {
            // A device's paper, else a standard size (no device).
            if !page_device("plot", &json!({ "paper": v }), page).unwrap_or(false)
                && let Some(ps) = v.as_str().and_then(cadcraft_render::paper_size)
            {
                page.paper = ps.name.into();
                page.width_mm = ps.width_mm;
                page.height_mm = ps.height_mm;
            }
        }
        "plotStyleTable" => page.plot_style_table = v.as_str().unwrap_or_default().chars().take(260).collect(),
        "plotArea" => page.plot_area = v.as_str().unwrap_or("extents").into(),
        "scale" => page.scale = v.as_f64().filter(|s| s.is_finite() && *s > 0.0).unwrap_or(page.scale),
        _ => {
            let b = v.as_bool().unwrap_or_default();
            match key {
                "landscape" => page.landscape = b,
                "scaleToFit" => page.scale_to_fit = b,
                "center" => page.center = b,
                "lineweights" => page.lineweights = b,
                "plotTransparency" => page.plot_transparency = b,
                _ => {}
            }
        }
    }
}

// ---------- Plot ----------

/// Bumped each time PLOT opens its dialog, so it starts again from the page setup.
static PLOT_GEN: AtomicU32 = AtomicU32::new(0);

/// Set while PLOTWINDOW picks the Plot dialog's window ("Window <").
const PICKING_WINDOW: &str = "plot_picking_window";
/// Set by Print... in a preview opened from the Plot dialog: plot as soon as the dialog is back.
pub(crate) const PLOT_NOW: &str = "plot_now";

/// Every frame: once PLOTWINDOW (started by the Plot dialog's "Window <") ends, bring the dialog
/// back with its settings, plotting the picked window (unchanged if the pick was cancelled).
pub fn plot_window_resume(app: &mut CadApp, ctx: &egui::Context) {
    if temp::<bool>(ctx, PICKING_WINDOW) != Some(true) || app.session.running.is_some() {
        return;
    }
    set_temp(ctx, PICKING_WINDOW, false);
    if let (Some(w), Some(mut st)) = (app.session.plot_window.take(), temp::<PlotState>(ctx, "plot_state")) {
        st.window = [w.min.x, w.min.y, w.max.x, w.max.y];
        st.page.plot_area = "window".into();
        set_temp(ctx, "plot_state", st);
    }
    app.ui.dialog = Some("plot".into());
}

/// Set a flag the Plot dialog reads (see [`PLOT_NOW`]).
pub(crate) fn set_flag(ctx: &egui::Context, id: &str, on: bool) {
    set_temp(ctx, id, on);
}

/// Plot areas for model space (a layout always plots its sheet).
const MODEL_AREAS: &[&str] = &["extents", "display", "limits", "window"];

/// The Plot dialog's settings until OK.
#[derive(Clone)]
struct PlotState {
    generation: u32,
    /// "Model" or a layout name.
    target: String,
    page: cadcraft_doc::PageSetup,
    /// Plot area "window" corners in drawing units: [x1, y1, x2, y2].
    window: [f64; 4],
    /// Model-space placement; stays outside PageSetup so layouts keep their existing behavior.
    center: bool,
    offset_mm: [f64; 2],
}

/// The page setup a plot of `target` starts from: the layout's, or model-space defaults (fitted
/// extents on A4 in metric drawings, ANSI A otherwise).
fn plot_page(d: &cadcraft_doc::Drawing, target: &str) -> cadcraft_doc::PageSetup {
    if let Some(l) = d.layouts.iter().find(|l| l.name == target) {
        let mut page = l.page.clone();
        page.plot_area = "layout".into();
        return page;
    }
    let mut page = cadcraft_doc::PageSetup { plot_area: "extents".into(), scale_to_fit: true, ..Default::default() };
    if cadcraft_render::paper::paper_unit_mm(d) == 1.0
        && let Some(a4) = cadcraft_render::paper_size("A4")
    {
        page.paper = a4.name.into();
        page.width_mm = a4.width_mm;
        page.height_mm = a4.height_mm;
    }
    page
}

/// The `plot` parameters for the dialog's settings (without the output path). `preview_dpi`
/// asks for a PNG at that resolution instead of the device's output, on the same sheet.
fn plot_params(st: &PlotState, preview_dpi: Option<f64>) -> Value {
    let page = &st.page;
    let mut p = json!({
        "layout": st.target,
        "landscape": page.landscape,
        "fit": page.scale_to_fit,
        "lineweights": page.lineweights,
        "plotTransparency": page.plot_transparency,
    });
    if let Some(o) = p.as_object_mut() {
        let device = device_label(page) != "None";
        let output = cadcraft_engine::cmd::plotters::find_device(&page.device).map_or("pdf", |d| d.output.as_str());
        match preview_dpi {
            // The page's own sheet, so a device's paper and printable area show as they plot.
            Some(dpi) => {
                o.insert("device".into(), json!("None"));
                o.insert("output".into(), json!("png"));
                o.insert("resolution".into(), json!(dpi));
                o.insert("width".into(), json!(page.width_mm));
                o.insert("height".into(), json!(page.height_mm));
                o.insert("margins".into(), json!(page.margins_mm));
            }
            None if device => {
                o.insert("device".into(), json!(page.device));
                o.insert("paper".into(), json!(page.paper));
            }
            None if cadcraft_render::paper_size(&page.paper).is_some() => {
                o.insert("device".into(), json!("None"));
                o.insert("paper".into(), json!(page.paper));
            }
            None => {
                o.insert("device".into(), json!("None"));
                o.insert("width".into(), json!(page.width_mm));
                o.insert("height".into(), json!(page.height_mm));
            }
        }
        if style_label(page) != "None" {
            o.insert("plotStyleTable".into(), json!(page.plot_style_table));
        }
        if st.target == "Model" {
            o.insert("plotArea".into(), json!(page.plot_area));
            if output != "svg" {
                o.insert("center".into(), json!(st.center));
                if !st.center {
                    o.insert("offset".into(), json!(st.offset_mm));
                }
            }
            if !page.scale_to_fit {
                o.insert("scale".into(), json!(page.scale));
            }
            if page.plot_area == "window" {
                let [x1, y1, x2, y2] = st.window;
                o.insert("window".into(), json!([[x1, y1], [x2, y2]]));
            }
        }
    }
    p
}

/// The Plot dialog: what to plot, on which paper and at what scale; OK asks where to save the PDF
/// (where the host has a file dialog) and runs `plot` with those settings.
fn plot(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let t = Tokens::get();
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "Plot", open);
        return None;
    };
    let mut targets = vec!["Model".to_string()];
    let mut layouts: Vec<(u32, String)> = d.layouts.iter().map(|l| (l.tab_order, l.name.clone())).collect();
    layouts.sort_by_key(|l| l.0);
    targets.extend(layouts.into_iter().map(|l| l.1));
    let generation = PLOT_GEN.load(Ordering::Relaxed);
    let mut st = temp::<PlotState>(ctx, "plot_state").filter(|s| s.generation == generation && targets.contains(&s.target)).unwrap_or_else(|| {
        let target = app.session.layout_space().tab_name().to_string();
        let mut ext = d.extents(&cadcraft_doc::Space::Model);
        if ext.is_empty() {
            ext = cadcraft_geom::Bounds2::new(cadcraft_geom::Vec2::ZERO, cadcraft_geom::Vec2::new(12.0, 9.0));
        }
        PlotState {
            generation,
            page: plot_page(d, &target),
            target,
            window: [ext.min.x, ext.min.y, ext.max.x, ext.max.y],
            center: true,
            offset_mm: [0.0, 0.0],
        }
    });
    let model = st.target == "Model";
    let choices = PageChoices::new(ctx, d);
    let offset_supported =
        choices.devices.iter().find(|dev| dev.name.eq_ignore_ascii_case(&st.page.device)).is_none_or(|dev| dev.output.as_str() != "svg");
    // Print... in a preview opened from here plots with these settings.
    let mut ok = temp::<bool>(ctx, PLOT_NOW) == Some(true);
    set_temp(ctx, PLOT_NOW, false);
    let (mut cancel, mut action, mut pick, mut preview) = (false, None, false, false);
    let center = ctx.content_rect().center();
    egui::Window::new("Plot").open(open).collapsible(false).resizable(false).pivot(egui::Align2::CENTER_CENTER).default_pos(center).show(ctx, |ui| {
        egui::Grid::new("plot_target").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
            ui.label("Layout");
            craft_ui::ComboBox::from_id_salt("plot_layout").selected_text(st.target.as_str()).width(190.0).show_ui(ui, |ui| {
                for n in &targets {
                    if ui.selectable_label(*n == st.target, n).clicked() && *n != st.target {
                        st.page = plot_page(d, n);
                        st.target = n.clone();
                    }
                }
            });
            ui.end_row();
        });
        ui.separator();
        ui.horizontal_top(|ui| {
            egui::Grid::new("plot_grid").num_columns(2).spacing(vec2(10.0, 6.0)).show(ui, |ui| {
                let rows = if model {
                    PageRows { areas: MODEL_AREAS, scale: true, center: false }
                } else {
                    PageRows { areas: &["layout"], scale: false, center: false }
                };
                let placement = model.then_some((&mut st.center, &mut st.offset_mm, offset_supported));
                if let Some((k, v)) = page_rows(ui, egui::Id::unique(("plot", st.target.as_str())), &st.page, &rows, &choices, placement) {
                    apply_page_change(&mut st.page, k, &v);
                }
                if model && st.page.plot_area == "window" {
                    for (i, label) in [(0, "First corner"), (2, "Opposite corner")] {
                        ui.label(label);
                        ui.horizontal(|ui| {
                            for (j, axis) in ["X", "Y"].iter().enumerate() {
                                let axis_label = ui.label(*axis);
                                let k = i + j;
                                if let Some(slot) = st.window.get_mut(k)
                                    && let Some(v) = num_field(ui, ("plot_window", k), *slot, Some(axis_label.id))
                                {
                                    *slot = v;
                                }
                            }
                        });
                        ui.end_row();
                    }
                    ui.label("");
                    pick = ui.button("Window <").on_hover_text("Pick the window's corners in the drawing").clicked();
                    ui.end_row();
                }
            });
            sheet_sketch(ui, &st.page, &t);
        });
        ui.separator();
        ui.horizontal(|ui| {
            if !model && ui.button("Apply to Layout").on_hover_text("Save these settings in the layout's page setup").clicked() {
                let page = &st.page;
                action = Some((
                    "pagesetup",
                    json!({
                        "layout": st.target,
                        "paper": page.paper,
                        "landscape": page.landscape,
                        "scaleToFit": page.scale_to_fit,
                        "lineweights": page.lineweights,
                        "plotTransparency": page.plot_transparency,
                    }),
                ));
            }
            preview = ui.button("Preview").on_hover_text("The sheet as these settings plot it").clicked();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                cancel = ui.button("Cancel").clicked();
                ok |= ui.button("OK").clicked();
            });
        });
    });
    // The output file type follows the device.
    let ext = choices.devices.iter().find(|dv| dv.name.eq_ignore_ascii_case(&st.page.device)).map_or("pdf", |dv| dv.output.as_str());
    set_temp(ctx, "plot_state", st.clone());
    if cancel {
        *open = false;
    }
    if pick {
        // The dialog comes back with these settings and the picked window (`plot_window_resume`).
        set_temp(ctx, PICKING_WINDOW, true);
        *open = false;
        app.start("plotwindow");
        return action;
    }
    if preview {
        // A sheet about 1600 pixels on its long side.
        let long_in = st.page.width_mm.max(st.page.height_mm).max(1.0) / 25.4;
        let mut info = format!("{} · {}", st.target, st.page.paper);
        if st.page.landscape {
            info.push_str(" · landscape");
        }
        crate::exchange::preview_plot(app, ctx, plot_params(&st, Some((1600.0 / long_in).clamp(50.0, 300.0))), info);
        return action;
    }
    if ok {
        let mut params = plot_params(&st, None);
        if let Some(pick) = app.services.pick_save.as_ref() {
            let title = app.session.state().map(|s| s.title.clone()).unwrap_or_default();
            let stem = std::path::Path::new(&title).file_stem().map(|s| s.to_string_lossy().to_string()).filter(|s| !s.is_empty());
            let suggested = format!("{}-{}.{ext}", stem.unwrap_or_else(|| "Drawing".into()), st.target);
            // Cancelling the file dialog keeps the Plot dialog open with its settings.
            let Some(path) = pick(&suggested) else { return action };
            if let Some(o) = params.as_object_mut() {
                o.insert("path".into(), json!(path));
            }
        }
        *open = false;
        match app.run("plot", params) {
            Ok(r) => {
                let msg = match r.get("path").and_then(Value::as_str) {
                    Some(path) => format!("Plotted {} to {path}", st.target),
                    None => format!("Plotted {} ({} bytes of PDF; there is no file dialog here)", st.target, r["bytes"]),
                };
                app.session.echo(msg);
            }
            // `run` already echoed the error; keep the dialog open to fix the settings.
            Err(_) => *open = true,
        }
    }
    action
}

/// The sheet in the chosen orientation with its printable area, and its size.
fn sheet_sketch(ui: &mut egui::Ui, page: &cadcraft_doc::PageSetup, t: &Tokens) {
    let (w, h) = if page.landscape { (page.height_mm, page.width_mm) } else { (page.width_mm, page.height_mm) };
    ui.vertical(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(120.0, 120.0), Sense::hover());
        let k = (110.0 / w.max(h).max(1.0)) as f32;
        let sheet = egui::Rect::from_center_size(rect.center(), vec2(w as f32 * k, h as f32 * k));
        ui.painter().rect_filled(sheet, 0.0, t.field);
        ui.painter().rect_stroke(sheet, 0.0, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
        let [l, b, r, top] = page.margins_mm.map(|m| m as f32 * k);
        let printable = egui::Rect::from_min_max(sheet.min + vec2(l, top), sheet.max - vec2(r, b));
        if printable.is_positive() {
            ui.painter().rect_stroke(printable, 0.0, Stroke::new(1.0, t.accent), egui::StrokeKind::Inside);
        }
        ui.label(RichText::new(format!("{w:.1} × {h:.1} mm")).color(t.text_dim));
    });
}

// ---------- Constraint Settings ----------

/// Constraint Settings dialog editing the drawing's parametric settings; returns the `constraintsettings` action.
fn constraint_settings(app: &mut CadApp, ctx: &egui::Context, open: &mut bool) -> Action {
    let t = Tokens::get();
    let Ok(d) = app.session.doc() else {
        no_drawing(ctx, "Constraint Settings", open);
        return None;
    };
    let s = d.parametric.settings.clone();
    let types = cadcraft_doc::ParametricSettings::default().auto_types;
    let mut action = None;
    let set = |k: &str, v: Value| Some(("constraintsettings", json!({ k: v })));
    egui::Window::new("Constraint Settings").open(open).collapsible(false).resizable(false).show(ctx, |ui| {
        ui.heading("Geometric");
        let mut infer = s.infer;
        if ui.checkbox(&mut infer, "Infer geometric constraints").changed() {
            action = set("infer", json!(infer));
        }
        let mut bars = s.bars_visible;
        if ui.checkbox(&mut bars, "Show constraint bars").changed() {
            action = set("barsVisible", json!(bars));
        }
        ui.horizontal(|ui| {
            let label = ui.label("Constraint bar transparency (%)");
            if let Some(v) = num_field(ui, "csettings_transp", f64::from(s.bar_transparency), Some(label.id)) {
                action = set("barTransparency", json!(v));
            }
        });
        ui.separator();
        ui.heading("Dimensional");
        let mut dims = s.dims_visible;
        if ui.checkbox(&mut dims, "Show dynamic constraints").changed() {
            action = set("dimsVisible", json!(dims));
        }
        ui.separator();
        ui.heading("AutoConstrain");
        ui.label(RichText::new("Constraint types AUTOCONSTRAIN applies").color(t.text_dim));
        egui::Grid::new("csettings_types").num_columns(3).show(ui, |ui| {
            for (i, ty) in types.iter().enumerate() {
                let mut on = s.auto_types.iter().any(|a| a.eq_ignore_ascii_case(ty));
                if ui.checkbox(&mut on, ty.as_str()).changed() {
                    let list: Vec<&String> =
                        types.iter().filter(|x| if *x == ty { on } else { s.auto_types.iter().any(|a| a.eq_ignore_ascii_case(x)) }).collect();
                    action = set("autoTypes", json!(list));
                }
                if i % 3 == 2 {
                    ui.end_row();
                }
            }
        });
        egui::Grid::new("csettings_tol").num_columns(2).show(ui, |ui| {
            let label = ui.label("Distance tolerance");
            if let Some(v) = num_field(ui, "csettings_dist", s.distance_tolerance, Some(label.id)) {
                action = set("distanceTolerance", json!(v));
            }
            ui.end_row();
            let label = ui.label("Angle tolerance (degrees)");
            if let Some(v) = num_field(ui, "csettings_angle", s.angle_tolerance, Some(label.id)) {
                action = set("angleTolerance", json!(v));
            }
            ui.end_row();
        });
    });
    action
}

#[cfg(test)]
mod plot_dialog_tests {
    use super::*;
    use cadcraft_engine::Session;

    fn state(target: &str) -> PlotState {
        let d = cadcraft_doc::Drawing::new_metric();
        PlotState {
            generation: 0,
            page: plot_page(&d, target),
            target: target.into(),
            window: [0.0, 0.0, 10.0, 10.0],
            center: true,
            offset_mm: [0.0, 0.0],
        }
    }

    #[test]
    fn tolerance_dialog_place_leaves_the_engine_waiting_for_a_canvas_point() {
        let mut app = CadApp::new(Session::new(), crate::Services::default());
        let result = route(
            &mut app,
            "ui.tolerance.place",
            &json!({
                "characteristic": "Position",
                "tol1": "%%c0.1(M)",
                "tol2": "",
                "datums": ["A", "B(M)", ""]
            }),
        );
        assert!(matches!(result, Some(Ok(_))));
        assert!(app.session.prompting(), "placement should still be interactive on the canvas");
        assert!(app.session.prompt_text().to_ascii_lowercase().contains("location"));
        app.session.cmdline("5,6").unwrap();
        assert!(!app.session.prompting());
        assert!(app.session.doc().unwrap().model.iter().any(|e| matches!(e.kind, cadcraft_doc::EntityKind::Tolerance(_))));
    }

    #[test]
    fn a_device_brings_its_papers_and_the_plot_names_them() {
        let mut st = state("Model");
        apply_page_change(&mut st.page, "device", &json!("CADCraft PNG"));
        assert_eq!(st.page.device, "CADCraft PNG");
        let dev = cadcraft_engine::cmd::plotters::find_device("CADCraft PNG").unwrap();
        assert!(dev.papers.iter().any(|p| p.name == st.page.paper), "a paper of the device: {}", st.page.paper);
        assert!(st.page.margins_mm.iter().any(|m| *m > 0.0), "the paper's printable area");
        apply_page_change(&mut st.page, "plotStyleTable", &json!("monochrome.ctb"));
        let p = plot_params(&st, None);
        assert_eq!((p["device"].as_str(), p["paper"].as_str()), (Some("CADCraft PNG"), Some(st.page.paper.as_str())));
        assert_eq!(p["plotStyleTable"], json!("monochrome.ctb"));
        // The preview plots the same sheet to PNG, whatever the device.
        let q = plot_params(&st, Some(150.0));
        assert_eq!((q["output"].as_str(), q["device"].as_str()), (Some("png"), Some("None")));
        assert_eq!(q["width"].as_f64(), Some(st.page.width_mm));
        assert_eq!(q["margins"], json!(st.page.margins_mm));
        // Back to no device: a standard paper, no device sent.
        apply_page_change(&mut st.page, "device", &json!("None"));
        apply_page_change(&mut st.page, "paper", &json!("A3"));
        let p = plot_params(&st, None);
        let a3 = cadcraft_render::paper_size("A3").unwrap().name;
        assert_eq!((p["device"].as_str(), p["paper"].as_str()), (Some("None"), Some(a3)));
        apply_page_change(&mut st.page, "plotStyleTable", &json!("None"));
        assert!(plot_params(&st, None).get("plotStyleTable").is_none());
    }

    #[test]
    fn plot_transparency_checkbox_reaches_the_plot() {
        let mut st = state("Model");
        assert!(!st.page.plot_transparency, "off by default");
        assert_eq!(plot_params(&st, None)["plotTransparency"], json!(false));
        apply_page_change(&mut st.page, "plotTransparency", &json!(true));
        assert!(st.page.plot_transparency);
        assert_eq!(plot_params(&st, None)["plotTransparency"], json!(true));
        // The preview plots it the same way.
        assert_eq!(plot_params(&st, Some(150.0))["plotTransparency"], json!(true));
        // Into a layout's page setup and back out to its Plot dialog.
        let mut s = Session::new();
        s.execute("pagesetup", &json!({ "layout": "Layout1", "plotTransparency": true })).unwrap();
        let page = plot_page(s.doc().unwrap(), "Layout1");
        assert!(page.plot_transparency);
        apply_page_change(&mut st.page, "plotTransparency", &json!(false));
        assert_eq!(plot_params(&st, None)["plotTransparency"], json!(false));
    }

    #[test]
    fn plot_center_and_offsets_are_model_only_and_omitted_for_svg() {
        let mut st = state("Model");
        assert_eq!(plot_params(&st, None)["center"], json!(true));
        st.center = false;
        st.offset_mm = [12.5, -7.0];
        let params = plot_params(&st, None);
        assert_eq!(params["center"], json!(false));
        assert_eq!(params["offset"], json!([12.5, -7.0]));
        let preview = plot_params(&st, Some(150.0));
        assert_eq!(preview["output"], json!("png"));
        assert_eq!(preview["offset"], json!([12.5, -7.0]));

        let svg = cadcraft_engine::cmd::plotters::devices().into_iter().find(|dev| dev.output.as_str() == "svg").expect("built-in SVG device");
        apply_page_change(&mut st.page, "device", &json!(svg.name));
        let params = plot_params(&st, None);
        assert!(params.get("center").is_none() && params.get("offset").is_none(), "SVG does not receive unsupported placement options: {params}");
        let preview = plot_params(&st, Some(150.0));
        assert!(
            preview.get("center").is_none() && preview.get("offset").is_none(),
            "disabled SVG placement controls cannot affect preview output either"
        );

        st.target = "Layout1".into();
        st.page = plot_page(&cadcraft_doc::Drawing::new_metric(), "Layout1");
        let params = plot_params(&st, None);
        assert!(params.get("center").is_none() && params.get("offset").is_none(), "layout plots retain their page setup mapping");
    }

    #[test]
    fn window_pick_brings_the_dialog_back_with_the_picked_window() {
        let ctx = egui::Context::default();
        let mut app = CadApp::new(Session::new(), crate::Services::default());
        set_temp(&ctx, "plot_state", state("Model"));
        set_temp(&ctx, PICKING_WINDOW, true);
        app.start("plotwindow");
        plot_window_resume(&mut app, &ctx);
        assert_eq!(app.ui.dialog, None, "still picking");
        app.session.cmdline("2,3").unwrap();
        app.session.cmdline("12,9").unwrap();
        plot_window_resume(&mut app, &ctx);
        assert_eq!(app.ui.dialog.as_deref(), Some("plot"));
        let st = temp::<PlotState>(&ctx, "plot_state").unwrap();
        assert_eq!((st.page.plot_area.as_str(), st.window), ("window", [2.0, 3.0, 12.0, 9.0]));
        // A cancelled pick brings the dialog back unchanged.
        app.ui.dialog = None;
        set_temp(&ctx, PICKING_WINDOW, true);
        app.start("plotwindow");
        app.session.input(cadcraft_engine::Input::Cancel).ok();
        plot_window_resume(&mut app, &ctx);
        assert_eq!(app.ui.dialog.as_deref(), Some("plot"));
        assert_eq!(temp::<PlotState>(&ctx, "plot_state").unwrap().window, [2.0, 3.0, 12.0, 9.0]);
    }
}
