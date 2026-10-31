//! Menus (in-window menu bar; the native macOS menu is built from the same tree), UI-only
//! commands and keyboard shortcuts.

use serde_json::{Value, json};

use crate::CadApp;

/// The top-level menus, in order.
pub const MENUS: &[&str] = &["File", "Edit", "View", "Insert", "Format", "Tools", "Draw", "Dimension", "Modify", "3D", "Showcase", "Window", "Help"];

/// File ▸ Open Recent: one `ui.recent.<n>` item per recent file, then Clear Recent Files.
pub const OPEN_RECENT: &str = "Open Recent";
const RECENT_ITEM: &str = "ui.recent.";
const RECENT_CLEAR: &str = "ui.recent.clear";

/// UI-only commands: (id, label, menu path, shortcut).
pub const UI_COMMANDS: &[(&str, &str, &[&str], Option<&str>)] = &[
    ("ui.spacemouse", "3D Mouse...", &["Tools", "3D Mouse..."], None),
    ("ui.mode.3d", "Open 3D Workspace", &["3D", "Open 3D Workspace"], None),
    ("ui.center.3d", "3D Control Center", &["3D", "3D Control Center"], None),
    ("ui.demo.3d", "Create 3D Demo Scene", &["3D", "Create 3D Demo Scene"], None),
    ("ui.demo.3d.arch", "Create Architectural 3D Demo", &["3D", "Create Architectural 3D Demo"], None),
    ("ui.demo.3d.qa", "Create 3D QA Showcase", &["3D", "Create 3D QA Showcase"], None),
    ("ui.demo.3d.house", "Create 3D House Demo", &["3D", "Create 3D House Demo"], None),
    ("ui.demo.3d.structuremep", "Create 3D Structure + MEP Demo", &["3D", "Create 3D Structure + MEP Demo"], None),
    ("ui.demo.3d.site", "Create 3D Site + Landscape Demo", &["3D", "Create 3D Site + Landscape Demo"], None),
    ("ui.demo.3d.interior", "Create 3D Interior + Lighting Demo", &["3D", "Create 3D Interior + Lighting Demo"], None),
    ("ui.demo.3d.facade", "Create 3D Facade + Roof Detail Demo", &["3D", "Create 3D Facade + Roof Detail Demo"], None),
    ("ui.demo.2d", "Create 2D QA Demo", &["Draw", "Create 2D QA Demo"], None),
    ("ui.demo.2d.annotated", "Create Annotated 2D Demo", &["Draw", "Create Annotated 2D Demo"], None),
    ("ui.demo.2d.floorplan", "Create 2D Floor Plan Demo", &["Draw", "Create 2D Floor Plan Demo"], None),
    ("ui.demo.2d.section", "Create 2D Section Demo", &["Draw", "Create 2D Section Demo"], None),
    ("ui.demo.2d.elevation", "Create 2D Elevation Demo", &["Draw", "Create 2D Elevation Demo"], None),
    ("ui.demo.2d.mep", "Create 2D MEP Plan Demo", &["Draw", "Create 2D MEP Plan Demo"], None),
    ("ui.demo.2d.structural", "Create 2D Structural Plan Demo", &["Draw", "Create 2D Structural Plan Demo"], None),
    ("ui.demo.2d.rcp", "Create 2D Ceiling + Lighting Plan Demo", &["Draw", "Create 2D Ceiling + Lighting Plan Demo"], None),
    ("ui.demo.2d.plumbingfire", "Create 2D Plumbing + Fire Plan Demo", &["Draw", "Create 2D Plumbing + Fire Plan Demo"], None),
    ("ui.demo.2d.schedule", "Create Door + Window Schedule Demo", &["Draw", "Create Door + Window Schedule Demo"], None),
    ("ui.showcase.2d", "2D QA Demo", &["Showcase", "2D QA Demo"], None),
    ("ui.showcase.2d.annotated", "Annotated 2D Demo", &["Showcase", "Annotated 2D Demo"], None),
    ("ui.showcase.3d", "3D Primitive Demo", &["Showcase", "3D Primitive Demo"], None),
    ("ui.showcase.3d.arch", "Architectural 3D Demo", &["Showcase", "Architectural 3D Demo"], None),
    ("ui.showcase.3d.qa", "3D QA Showcase", &["Showcase", "3D QA Showcase"], None),
    ("ui.showcase.2d.floorplan", "2D Floor Plan Demo", &["Showcase", "2D Floor Plan Demo"], None),
    ("ui.showcase.3d.house", "3D House Demo", &["Showcase", "3D House Demo"], None),
    ("ui.showcase.3d.structuremep", "3D Structure + MEP Demo", &["Showcase", "3D Structure + MEP Demo"], None),
    ("ui.showcase.2d.section", "2D Section Demo", &["Showcase", "2D Section Demo"], None),
    ("ui.showcase.2d.elevation", "2D Elevation Demo", &["Showcase", "2D Elevation Demo"], None),
    ("ui.showcase.2d.mep", "2D MEP Plan Demo", &["Showcase", "2D MEP Plan Demo"], None),
    ("ui.showcase.3d.site", "3D Site + Landscape Demo", &["Showcase", "3D Site + Landscape Demo"], None),
    ("ui.showcase.2d.structural", "2D Structural Plan Demo", &["Showcase", "2D Structural Plan Demo"], None),
    ("ui.showcase.2d.rcp", "2D Ceiling + Lighting Plan Demo", &["Showcase", "2D Ceiling + Lighting Plan Demo"], None),
    ("ui.showcase.3d.interior", "3D Interior + Lighting Demo", &["Showcase", "3D Interior + Lighting Demo"], None),
    ("ui.showcase.2d.plumbingfire", "2D Plumbing + Fire Plan Demo", &["Showcase", "2D Plumbing + Fire Plan Demo"], None),
    ("ui.showcase.2d.schedule", "Door + Window Schedule Demo", &["Showcase", "Door + Window Schedule Demo"], None),
    ("ui.showcase.3d.facade", "3D Facade + Roof Detail Demo", &["Showcase", "3D Facade + Roof Detail Demo"], None),
    ("ui.spacemouse.calibrate", "Calibrate 3D Mouse", &[], None),
    ("ui.spacemouse.navigate", "3D Mouse Navigation", &[], None),
    ("ui.dock", "Arrange Panel", &[], None),
    ("ui.dock.reset", "Reset Panel Layout", &["Window", "Reset Panel Layout"], None),
    ("ui.open", "Open...", &[], Some("Cmd+O")),
    ("ui.newtemplate", "New from Template...", &["File", "New from Template..."], None),
    ("ui.saveas", "Save As...", &[], None),
    ("ui.sample", "Open Sample Drawing", &["Help", "Open Sample Drawing"], None),
    ("ui.toggle.toolsets", "Tool Sets", &["Window", "Tool Sets"], Some("Cmd+3")),
    ("ui.panel.properties", "Properties Inspector", &["Window", "Properties Inspector"], Some("Cmd+1")),
    ("ui.panel.layers", "Layers Palette", &["Window", "Layers Palette"], None),
    ("ui.toggle.quickcalc", "QuickCalc", &["Window", "QuickCalc"], Some("Cmd+8")),
    ("ui.panel.quickcalc", "QuickCalc", &[], None),
    ("ui.toggle.systemcursor", "Show System Cursor", &["View", "Accessibility", "Show System Cursor"], None),
    ("ui.animations.system", "Use System Setting", &["View", "Accessibility", "Animation Effects", "Use System Setting"], None),
    ("ui.animations.on", "On", &["View", "Accessibility", "Animation Effects", "On"], None),
    ("ui.animations.off", "Off", &["View", "Accessibility", "Animation Effects", "Off"], None),
    ("ui.animations", "Animation Effects", &[], None),
    ("ui.toggle.toolbar", "Tool Bar", &["Window", "Tool Bar"], None),
    ("ui.toggle.filetabs", "File Tab", &["Window", "File Tab"], None),
    ("ui.toggle.statusbar", "Status Bar", &["Window", "Status Bar"], None),
    ("ui.toggle.cmdline", "Command Line", &["Window", "Command Line"], Some("Cmd+9")),
    ("ui.toggle.viewcube", "ViewCube", &["View", "ViewCube", "On"], None),
    ("ui.toggle.ucsicon", "UCS Icon", &["View", "UCS Icon", "On"], None),
    ("ui.theme.system", "Use System Setting", &["View", "Interface Theme", "Use System Setting"], None),
    ("ui.theme.light", "Light", &["View", "Interface Theme", "Light"], None),
    ("ui.theme.dark", "Dark", &["View", "Interface Theme", "Dark"], None),
    ("ui.theme", "Interface Theme", &[], None),
    ("ui.saveformat.dxf", "DXF", &["File", "Default Save Format", "DXF"], None),
    ("ui.saveformat.dwg", "DWG", &["File", "Default Save Format", "DWG"], None),
    ("ui.saveformat", "Default Save Format", &[], None),
    ("ui.saveversion.2018", "Version 2018", &["File", "Default Save Format", "Version 2018"], None),
    ("ui.saveversion.2013", "Version 2013", &["File", "Default Save Format", "Version 2013"], None),
    ("ui.saveversion.2010", "Version 2010", &["File", "Default Save Format", "Version 2010"], None),
    ("ui.saveversion.2007", "Version 2007", &["File", "Default Save Format", "Version 2007"], None),
    ("ui.saveversion.2004", "Version 2004", &["File", "Default Save Format", "Version 2004"], None),
    ("ui.saveversion.2000", "Version 2000", &["File", "Default Save Format", "Version 2000"], None),
    ("ui.saveversion", "Default Save Version", &[], None),
    ("ui.toggle.menubar", "In-window Menu Bar", &["Window", "In-window Menu Bar"], None),
    ("ui.hidepalettes", "Hide Palettes", &["Window", "Hide Palettes"], None),
    ("ui.resetpalettes", "Reset Palettes", &["Window", "Reset Palettes"], None),
    ("ui.start", "Start", &["Window", "Start"], None),
    ("ui.dialog.layers", "Layer Properties Manager", &["Window", "Layers"], None),
    ("ui.dialog.blocks", "Blocks", &["Window", "Blocks"], None),
    ("ui.dialog.qselect", "Quick Select...", &[], None),
    ("ui.dialog.parameters", "Parameters Manager", &["Window", "Parameters Manager"], None),
    ("ui.dialog.dsettings", "Drafting Settings...", &[], None),
    ("ui.dialog.about", "About CAD 2D - TBT 360", &["Help", "About CAD 2D - TBT 360"], None),
    ("ui.dialog.commands", "Command Reference", &["Help", "CAD 2D - TBT 360 Help"], Some("F1")),
    ("ui.toggle.history", "Command History", &["Window", "Command History"], Some("F2")),
    ("ui.cmdline.lines", "Command Line History Lines", &[], None),
    ("ui.cmdline.clear", "Clear Command History", &[], None),
    ("ui.dialog.language", "Interface Language", &["Edit", "Interface Language"], None),
    ("ui.language", "Language", &[], None),
    ("ui.noop", "", &[], None),
    ("ui.quit", "Quit CAD 2D - TBT 360", &[], Some("Cmd+Q")),
];

pub fn is_ui_command(id: &str) -> bool {
    UI_COMMANDS.iter().any(|c| c.0 == id)
}

/// Run a UI-only command. `None` if `id` isn't one.
pub fn run_ui_command(app: &mut CadApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    crate::i18n::with_language(app.language(), || run_ui_command_inner(app, id, params))
}

fn run_ui_command_inner(app: &mut CadApp, id: &str, params: &Value) -> Option<Result<Value, String>> {
    let id = match id {
        "ui.showcase.2d" => "ui.demo.2d",
        "ui.showcase.2d.annotated" => "ui.demo.2d.annotated",
        "ui.showcase.3d" => "ui.demo.3d",
        "ui.showcase.3d.arch" => "ui.demo.3d.arch",
        "ui.showcase.3d.qa" => "ui.demo.3d.qa",
        "ui.showcase.2d.floorplan" => "ui.demo.2d.floorplan",
        "ui.showcase.3d.house" => "ui.demo.3d.house",
        "ui.showcase.3d.structuremep" => "ui.demo.3d.structuremep",
        "ui.showcase.2d.section" => "ui.demo.2d.section",
        "ui.showcase.2d.elevation" => "ui.demo.2d.elevation",
        "ui.showcase.2d.mep" => "ui.demo.2d.mep",
        "ui.showcase.3d.site" => "ui.demo.3d.site",
        "ui.showcase.2d.structural" => "ui.demo.2d.structural",
        "ui.showcase.2d.rcp" => "ui.demo.2d.rcp",
        "ui.showcase.3d.interior" => "ui.demo.3d.interior",
        "ui.showcase.2d.plumbingfire" => "ui.demo.2d.plumbingfire",
        "ui.showcase.2d.schedule" => "ui.demo.2d.schedule",
        "ui.showcase.3d.facade" => "ui.demo.3d.facade",
        other => other,
    };
    if id == "ui.mode.3d" {
        app.ui.toolset_tab = "Modeling".into();
        app.ui.show_toolsets = true;
        app.ui.show_viewcube = true;
        app.ui.show_ucs_icon = true;
        app.set_status("3D workspace: Modeling tools, ViewCube and UCS are visible");
        return Some(Ok(json!({"workspace":"3D","toolset":"Modeling"})));
    }
    if id == "ui.center.3d" {
        app.ui.toolset_tab = "Modeling".into();
        app.ui.show_toolsets = true;
        app.ui.show_viewcube = true;
        app.ui.show_ucs_icon = true;
        app.ui.dialog = Some("3dcenter".into());
        app.set_status("3D Control Center opened");
        return Some(Ok(json!({"dialog":"3dcenter","workspace":"3D"})));
    }
    if id == "ui.demo.2d" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("line", json!({"points":[[-30.0,-18.0],[30.0,-18.0],[30.0,18.0],[-30.0,18.0],[-30.0,-18.0]]})),
            ("circle", json!({"center":[-14.0,0.0],"radius":7.0})),
            ("circle", json!({"center":[14.0,0.0],"radius":7.0})),
            ("line", json!({"points":[[-30.0,0.0],[30.0,0.0]]})),
            ("line", json!({"points":[[0.0,-18.0],[0.0,18.0]]})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Drafting".into();
        app.ui.show_toolsets = true;
        app.set_status(&format!("2D QA demo created: {created}/5 command groups"));
        return Some(Ok(json!({"created":created,"workspace":"2D"})));
    }

    if id == "ui.demo.2d.annotated" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("rectang", json!({"p1":[-40.0,-25.0],"p2":[40.0,25.0]})),
            ("rectang", json!({"p1":[-30.0,-15.0],"p2":[30.0,15.0]})),
            ("circle", json!({"center":[0.0,0.0],"radius":8.0})),
            ("dimlinear", json!({"p1":[-40.0,-25.0],"p2":[40.0,-25.0],"at":[0.0,-32.0]})),
            ("dimlinear", json!({"p1":[40.0,-25.0],"p2":[40.0,25.0],"at":[48.0,0.0]})),
            ("mtext", json!({"at":[-38.0,29.0],"text":"TBT360 CAD 2027 - 2D QA / Annotation Demo","height":2.5,"width":76.0})),
            ("mtext", json!({"at":[-28.0,11.0],"text":"Native geometry + dimensions + text","height":2.0,"width":56.0})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Drafting".into();
        app.ui.show_toolsets = true;
        app.set_status(&format!("Annotated 2D demo created: {created}/7 command groups"));
        return Some(Ok(json!({"created":created,"workspace":"2D","evidence":"annotation"})));
    }

    if id == "ui.demo.2d.floorplan" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("rectang", json!({"p1":[-30.0,-20.0],"p2":[30.0,20.0]})),
            ("rectang", json!({"p1":[-29.4,-19.4],"p2":[29.4,19.4]})),
            ("line", json!({"points":[[0.0,-19.4],[0.0,7.0]]})),
            ("line", json!({"points":[[0.0,11.0],[0.0,19.4]]})),
            ("line", json!({"points":[[-29.4,2.0],[-4.0,2.0]]})),
            ("line", json!({"points":[[4.0,2.0],[29.4,2.0]]})),
            ("line", json!({"points":[[-12.0,2.0],[-12.0,19.4]]})),
            ("line", json!({"points":[[14.0,2.0],[14.0,19.4]]})),
            ("rectang", json!({"p1":[-5.0,-20.3],"p2":[5.0,-19.1]})),
            ("rectang", json!({"p1":[-25.0,19.1],"p2":[-15.0,20.3]})),
            ("rectang", json!({"p1":[15.0,19.1],"p2":[25.0,20.3]})),
            ("dimlinear", json!({"p1":[-30.0,-20.0],"p2":[30.0,-20.0],"at":[0.0,-25.0]})),
            ("dimlinear", json!({"p1":[30.0,-20.0],"p2":[30.0,20.0],"at":[35.0,0.0]})),
            ("mtext", json!({"at":[-24.0,-8.0],"text":"LIVING / PHONG KHACH","height":2.0,"width":20.0})),
            ("mtext", json!({"at":[5.0,-8.0],"text":"DINING + KITCHEN / BEP","height":2.0,"width":20.0})),
            ("mtext", json!({"at":[-27.0,12.0],"text":"BED 01","height":1.8,"width":12.0})),
            ("mtext", json!({"at":[2.0,12.0],"text":"HALL","height":1.8,"width":10.0})),
            ("mtext", json!({"at":[17.0,12.0],"text":"BED 02","height":1.8,"width":10.0})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Drafting".into();
        app.ui.show_toolsets = true;
        app.set_status(&format!("2D floor-plan demo created: {created}/18 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"2D","evidence":"floor-plan"})));
    }


    if id == "ui.demo.2d.section" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            // footing / wall / slab section, all native editable 2D entities
            ("rectang", json!({"p1":[-18.0,-10.0],"p2":[18.0,-6.0]})),
            ("rectang", json!({"p1":[-4.0,-6.0],"p2":[4.0,18.0]})),
            ("rectang", json!({"p1":[-22.0,8.0],"p2":[22.0,10.0]})),
            ("rectang", json!({"p1":[-22.0,18.0],"p2":[22.0,19.2]})),
            ("line", json!({"points":[[-22.0,0.0],[22.0,0.0]]})),
            ("hatch", json!({"points":[[0.0,-8.0]],"pattern":"ANSI31","scale":1.2,"angle":45.0})),
            ("hatch", json!({"points":[[0.0,9.0]],"pattern":"ANSI31","scale":1.0,"angle":45.0})),
            ("dimlinear", json!({"p1":[-18.0,-10.0],"p2":[18.0,-10.0],"at":[0.0,-14.0]})),
            ("dimlinear", json!({"p1":[4.0,-6.0],"p2":[4.0,18.0],"at":[9.0,6.0]})),
            ("mtext", json!({"at":[-20.0,22.0],"text":"TBT360 2D SECTION / MAT CAT","height":2.0,"width":40.0})),
            ("mtext", json!({"at":[6.0,13.0],"text":"RC WALL / TUONG BTCT","height":1.5,"width":18.0})),
            ("mtext", json!({"at":[-16.0,-4.0],"text":"GROUND / DAT","height":1.4,"width":14.0})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Drafting".into();
        app.ui.show_toolsets = true;
        app.set_status(&format!("2D section demo created: {created}/12 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"2D","evidence":"section","hatch":true,"dimensions":true})));
    }


    if id == "ui.demo.2d.elevation" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("rectang", json!({"p1":[-30.0,0.0],"p2":[30.0,20.0]})),
            ("line", json!({"points":[[-32.0,0.0],[32.0,0.0]]})),
            ("line", json!({"points":[[-30.0,20.0],[0.0,30.0],[30.0,20.0]]})),
            ("rectang", json!({"p1":[-4.0,0.0],"p2":[4.0,10.0]})),
            ("rectang", json!({"p1":[-24.0,6.0],"p2":[-14.0,14.0]})),
            ("rectang", json!({"p1":[14.0,6.0],"p2":[24.0,14.0]})),
            ("rectang", json!({"p1":[-10.0,6.0],"p2":[-5.0,14.0]})),
            ("rectang", json!({"p1":[5.0,6.0],"p2":[10.0,14.0]})),
            ("line", json!({"points":[[-4.0,5.0],[4.0,5.0]]})),
            ("line", json!({"points":[[-24.0,10.0],[-14.0,10.0]]})),
            ("line", json!({"points":[[14.0,10.0],[24.0,10.0]]})),
            ("dimlinear", json!({"p1":[-30.0,0.0],"p2":[30.0,0.0],"at":[0.0,-5.0]})),
            ("dimlinear", json!({"p1":[30.0,0.0],"p2":[30.0,20.0],"at":[35.0,10.0]})),
            ("mtext", json!({"at":[-15.0,33.0],"text":"FRONT ELEVATION / MAT DUNG","height":2.0,"width":30.0})),
            ("mtext", json!({"at":[-3.0,3.0],"text":"ENTRY","height":1.2,"width":8.0})),
            ("mtext", json!({"at":[-29.0,-3.0],"text":"TBT360 2D ARCHITECTURAL EVIDENCE","height":1.3,"width":40.0})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Drafting".into();
        app.ui.show_toolsets = true;
        app.set_status(&format!("2D elevation demo created: {created}/16 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"2D","evidence":"elevation","dimensions":true,"annotation":true})));
    }

    if id == "ui.demo.2d.mep" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("rectang", json!({"p1":[-30.0,-20.0],"p2":[30.0,20.0]})),
            ("line", json!({"points":[[-24.0,12.0],[-8.0,12.0],[-8.0,2.0],[8.0,2.0],[8.0,-10.0],[24.0,-10.0]]})),
            ("line", json!({"points":[[-22.0,-12.0],[-12.0,-12.0],[-12.0,-2.0],[12.0,-2.0],[12.0,10.0],[22.0,10.0]]})),
            ("circle", json!({"center":[-24.0,12.0],"radius":1.5})),
            ("circle", json!({"center":[24.0,-10.0],"radius":1.5})),
            ("circle", json!({"center":[-22.0,-12.0],"radius":1.2})),
            ("circle", json!({"center":[22.0,10.0],"radius":1.2})),
            ("rectang", json!({"p1":[-5.0,7.0],"p2":[5.0,13.0]})),
            ("rectang", json!({"p1":[-5.0,-13.0],"p2":[5.0,-7.0]})),
            ("mtext", json!({"at":[-28.0,24.0],"text":"MEP PLAN / MAT BANG HE THONG","height":2.0,"width":45.0})),
            ("mtext", json!({"at":[-3.5,9.0],"text":"AHU","height":1.5,"width":8.0})),
            ("mtext", json!({"at":[-3.5,-11.0],"text":"PUMP","height":1.5,"width":8.0})),
            ("dimlinear", json!({"p1":[-30.0,-20.0],"p2":[30.0,-20.0],"at":[0.0,-25.0]})),
            ("mtext", json!({"at":[13.0,14.0],"text":"SUPPLY AIR","height":1.2,"width":12.0})),
            ("mtext", json!({"at":[13.0,-16.0],"text":"WATER / FIRE","height":1.2,"width":12.0})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Drafting".into();
        app.ui.show_toolsets = true;
        app.set_status(&format!("2D MEP plan demo created: {created}/15 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"2D","evidence":"mep-plan","systems":["hvac","plumbing-fire"]})));
    }


    if id == "ui.demo.2d.plumbingfire" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("rectang", json!({"p1":[-32.0,-20.0],"p2":[32.0,20.0]})),
            ("rectang", json!({"p1":[-28.0,-16.0],"p2":[28.0,16.0]})),
            ("line", json!({"points":[[-24.0,12.0],[-10.0,12.0],[-10.0,3.0],[4.0,3.0],[4.0,-10.0],[24.0,-10.0]]})),
            ("line", json!({"points":[[-24.0,8.0],[-15.0,8.0],[-15.0,-4.0],[12.0,-4.0],[12.0,10.0],[24.0,10.0]]})),
            ("line", json!({"points":[[-22.0,-12.0],[-6.0,-12.0],[-6.0,0.0],[18.0,0.0]]})),
            ("circle", json!({"center":[-24.0,12.0],"radius":1.4})),
            ("circle", json!({"center":[24.0,-10.0],"radius":1.4})),
            ("circle", json!({"center":[-24.0,8.0],"radius":1.2})),
            ("circle", json!({"center":[24.0,10.0],"radius":1.2})),
            ("circle", json!({"center":[-22.0,-12.0],"radius":1.2})),
            ("rectang", json!({"p1":[-3.0,-3.0],"p2":[3.0,3.0]})),
            ("rectang", json!({"p1":[15.0,-3.0],"p2":[21.0,3.0]})),
            ("mtext", json!({"at":[-30.0,24.0],"text":"PLUMBING + FIRE PLAN / MAT BANG CAP THOAT NUOC + PCCC","height":1.7,"width":60.0})),
            ("mtext", json!({"at":[-28.0,18.0],"text":"CW / WATER","height":1.1,"width":12.0})),
            ("mtext", json!({"at":[8.0,18.0],"text":"FIRE / PCCC","height":1.1,"width":12.0})),
            ("mtext", json!({"at":[-1.8,-1.0],"text":"PUMP","height":1.0,"width":6.0})),
            ("mtext", json!({"at":[16.0,-1.0],"text":"RISER","height":1.0,"width":6.0})),
            ("dimlinear", json!({"p1":[-32.0,-20.0],"p2":[32.0,-20.0],"at":[0.0,-25.0]})),
            ("dimlinear", json!({"p1":[32.0,-20.0],"p2":[32.0,20.0],"at":[38.0,0.0]})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Drafting".into();
        app.ui.show_toolsets = true;
        app.set_status(&format!("2D plumbing + fire plan demo created: {created}/19 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"2D","evidence":"plumbing-fire-plan","features":["water","fire","pump","riser","dimensions"]})));
    }

    if id == "ui.demo.2d.schedule" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("rectang", json!({"p1":[-34.0,-18.0],"p2":[34.0,18.0]})),
            ("line", json!({"points":[[-34.0,12.0],[34.0,12.0]]})),
            ("line", json!({"points":[[-34.0,6.0],[34.0,6.0]]})),
            ("line", json!({"points":[[-34.0,0.0],[34.0,0.0]]})),
            ("line", json!({"points":[[-34.0,-6.0],[34.0,-6.0]]})),
            ("line", json!({"points":[[-34.0,-12.0],[34.0,-12.0]]})),
            ("line", json!({"points":[[-22.0,-18.0],[-22.0,18.0]]})),
            ("line", json!({"points":[[-8.0,-18.0],[-8.0,18.0]]})),
            ("line", json!({"points":[[8.0,-18.0],[8.0,18.0]]})),
            ("line", json!({"points":[[22.0,-18.0],[22.0,18.0]]})),
            ("mtext", json!({"at":[-32.0,22.0],"text":"DOOR + WINDOW SCHEDULE / BANG THONG KE CUA","height":1.8,"width":58.0})),
            ("mtext", json!({"at":[-31.0,14.0],"text":"MARK","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[-20.0,14.0],"text":"TYPE","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[-6.0,14.0],"text":"W x H","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[10.0,14.0],"text":"QTY","height":1.1,"width":8.0})),
            ("mtext", json!({"at":[23.0,14.0],"text":"NOTE","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[-31.0,8.0],"text":"D01","height":1.1,"width":8.0})),
            ("mtext", json!({"at":[-20.0,8.0],"text":"DOOR","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[-6.0,8.0],"text":"900 x 2100","height":1.1,"width":12.0})),
            ("mtext", json!({"at":[11.0,8.0],"text":"4","height":1.1,"width":4.0})),
            ("mtext", json!({"at":[23.0,8.0],"text":"TIMBER","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[-31.0,2.0],"text":"D02","height":1.1,"width":8.0})),
            ("mtext", json!({"at":[-20.0,2.0],"text":"DOOR","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[-6.0,2.0],"text":"1200 x 2400","height":1.1,"width":12.0})),
            ("mtext", json!({"at":[11.0,2.0],"text":"1","height":1.1,"width":4.0})),
            ("mtext", json!({"at":[23.0,2.0],"text":"GLASS","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[-31.0,-4.0],"text":"W01","height":1.1,"width":8.0})),
            ("mtext", json!({"at":[-20.0,-4.0],"text":"WINDOW","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[-6.0,-4.0],"text":"1500 x 1500","height":1.1,"width":12.0})),
            ("mtext", json!({"at":[11.0,-4.0],"text":"8","height":1.1,"width":4.0})),
            ("mtext", json!({"at":[23.0,-4.0],"text":"ALUMINUM","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[-31.0,-10.0],"text":"W02","height":1.1,"width":8.0})),
            ("mtext", json!({"at":[-20.0,-10.0],"text":"WINDOW","height":1.1,"width":10.0})),
            ("mtext", json!({"at":[-6.0,-10.0],"text":"900 x 1200","height":1.1,"width":12.0})),
            ("mtext", json!({"at":[11.0,-10.0],"text":"6","height":1.1,"width":4.0})),
            ("mtext", json!({"at":[23.0,-10.0],"text":"ALUMINUM","height":1.1,"width":10.0})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Drafting".into();
        app.ui.show_toolsets = true;
        app.set_status(&format!("Door + window schedule demo created: {created}/36 evidence items"));
        return Some(Ok(json!({"created":created,"workspace":"2D","evidence":"door-window-schedule","features":["doors","windows","sizes","quantities","notes"]})));
    }

    if id == "ui.demo.2d.structural" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            // grid axes
            ("line", json!({"points":[[-30.0,-18.0],[-30.0,18.0]]})),
            ("line", json!({"points":[[-10.0,-18.0],[-10.0,18.0]]})),
            ("line", json!({"points":[[10.0,-18.0],[10.0,18.0]]})),
            ("line", json!({"points":[[30.0,-18.0],[30.0,18.0]]})),
            ("line", json!({"points":[[-34.0,-14.0],[34.0,-14.0]]})),
            ("line", json!({"points":[[-34.0,0.0],[34.0,0.0]]})),
            ("line", json!({"points":[[-34.0,14.0],[34.0,14.0]]})),
            // columns
            ("rectang", json!({"p1":[-31.0,-15.0],"p2":[-29.0,-13.0]})),
            ("rectang", json!({"p1":[-11.0,-15.0],"p2":[-9.0,-13.0]})),
            ("rectang", json!({"p1":[9.0,-15.0],"p2":[11.0,-13.0]})),
            ("rectang", json!({"p1":[29.0,-15.0],"p2":[31.0,-13.0]})),
            ("rectang", json!({"p1":[-31.0,-1.0],"p2":[-29.0,1.0]})),
            ("rectang", json!({"p1":[-11.0,-1.0],"p2":[-9.0,1.0]})),
            ("rectang", json!({"p1":[9.0,-1.0],"p2":[11.0,1.0]})),
            ("rectang", json!({"p1":[29.0,-1.0],"p2":[31.0,1.0]})),
            ("rectang", json!({"p1":[-31.0,13.0],"p2":[-29.0,15.0]})),
            ("rectang", json!({"p1":[-11.0,13.0],"p2":[-9.0,15.0]})),
            ("rectang", json!({"p1":[9.0,13.0],"p2":[11.0,15.0]})),
            ("rectang", json!({"p1":[29.0,13.0],"p2":[31.0,15.0]})),
            // primary beam centre-lines and slab boundary
            ("rectang", json!({"p1":[-31.0,-15.0],"p2":[31.0,15.0]})),
            ("line", json!({"points":[[-30.0,-14.0],[30.0,-14.0]]})),
            ("line", json!({"points":[[-30.0,0.0],[30.0,0.0]]})),
            ("line", json!({"points":[[-30.0,14.0],[30.0,14.0]]})),
            ("dimlinear", json!({"p1":[-30.0,-14.0],"p2":[30.0,-14.0],"at":[0.0,-21.0]})),
            ("dimlinear", json!({"p1":[30.0,-14.0],"p2":[30.0,14.0],"at":[38.0,0.0]})),
            ("mtext", json!({"at":[-28.0,22.0],"text":"STRUCTURAL PLAN / MAT BANG KET CAU","height":2.0,"width":50.0})),
            ("mtext", json!({"at":[-28.0,18.5],"text":"GRID + COLUMN + BEAM + SLAB EVIDENCE","height":1.2,"width":48.0})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Drafting".into();
        app.ui.show_toolsets = true;
        app.set_status(&format!("2D structural plan demo created: {created}/27 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"2D","evidence":"structural-plan","features":["grid","columns","beams","slab","dimensions"]})));
    }

    if id == "ui.demo.2d.rcp" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("rectang", json!({"p1":[-30.0,-18.0],"p2":[30.0,18.0]})),
            ("rectang", json!({"p1":[-24.0,-12.0],"p2":[24.0,12.0]})),
            // ceiling grid
            ("line", json!({"points":[[-24.0,-6.0],[24.0,-6.0]]})),
            ("line", json!({"points":[[-24.0,0.0],[24.0,0.0]]})),
            ("line", json!({"points":[[-24.0,6.0],[24.0,6.0]]})),
            ("line", json!({"points":[[-12.0,-12.0],[-12.0,12.0]]})),
            ("line", json!({"points":[[0.0,-12.0],[0.0,12.0]]})),
            ("line", json!({"points":[[12.0,-12.0],[12.0,12.0]]})),
            // downlights
            ("circle", json!({"center":[-18.0,-9.0],"radius":1.0})),
            ("circle", json!({"center":[-6.0,-9.0],"radius":1.0})),
            ("circle", json!({"center":[6.0,-9.0],"radius":1.0})),
            ("circle", json!({"center":[18.0,-9.0],"radius":1.0})),
            ("circle", json!({"center":[-18.0,9.0],"radius":1.0})),
            ("circle", json!({"center":[-6.0,9.0],"radius":1.0})),
            ("circle", json!({"center":[6.0,9.0],"radius":1.0})),
            ("circle", json!({"center":[18.0,9.0],"radius":1.0})),
            // linear lights / return air grilles
            ("rectang", json!({"p1":[-15.0,-1.0],"p2":[-3.0,1.0]})),
            ("rectang", json!({"p1":[3.0,-1.0],"p2":[15.0,1.0]})),
            ("rectang", json!({"p1":[-27.0,-3.0],"p2":[-23.0,3.0]})),
            ("rectang", json!({"p1":[23.0,-3.0],"p2":[27.0,3.0]})),
            ("dimlinear", json!({"p1":[-30.0,-18.0],"p2":[30.0,-18.0],"at":[0.0,-23.0]})),
            ("mtext", json!({"at":[-28.0,22.0],"text":"REFLECTED CEILING + LIGHTING PLAN / MAT BANG TRAN DEN","height":1.8,"width":56.0})),
            ("mtext", json!({"at":[-28.0,19.0],"text":"DOWNLIGHT + LINEAR LIGHT + AIR GRILLE","height":1.2,"width":42.0})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Drafting".into();
        app.ui.show_toolsets = true;
        app.set_status(&format!("2D ceiling + lighting demo created: {created}/23 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"2D","evidence":"ceiling-lighting-plan","features":["ceiling-grid","downlights","linear-lights","air-grilles","dimensions"]})));
    }

    if id == "ui.demo.3d.interior" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            // room shell with front open for inspection
            ("box", json!({"base":[-24.0,-18.0,0.0],"length":48.0,"width":36.0,"height":0.5})),
            ("box", json!({"base":[-24.0,17.5,0.5],"length":48.0,"width":0.5,"height":14.0})),
            ("box", json!({"base":[-24.0,-18.0,0.5],"length":0.5,"width":36.0,"height":14.0})),
            ("box", json!({"base":[23.5,-18.0,0.5],"length":0.5,"width":36.0,"height":14.0})),
            // sofa assembly
            ("box", json!({"base":[-17.0,7.0,0.5],"length":14.0,"width":5.5,"height":2.2})),
            ("box", json!({"base":[-17.0,11.5,2.7],"length":14.0,"width":1.0,"height":3.5})),
            ("box", json!({"base":[-18.0,7.0,0.5],"length":1.0,"width":5.5,"height":4.0})),
            ("box", json!({"base":[-3.0,7.0,0.5],"length":1.0,"width":5.5,"height":4.0})),
            // round coffee table + stools
            ("cylinder", json!({"center":[2.0,8.0,0.5],"radius":3.0,"height":1.6,"segments":24})),
            ("cylinder", json!({"center":[7.0,8.0,0.5],"radius":1.6,"height":1.5,"segments":18})),
            ("cylinder", json!({"center":[2.0,3.5,0.5],"radius":1.6,"height":1.5,"segments":18})),
            // TV console + screen
            ("box", json!({"base":[12.0,13.5,0.5],"length":9.0,"width":2.0,"height":2.2})),
            ("box", json!({"base":[13.0,16.0,4.0],"length":7.0,"width":0.5,"height":5.0})),
            // dining table + four chairs
            ("box", json!({"base":[4.0,-11.0,3.0],"length":12.0,"width":6.0,"height":0.6})),
            ("box", json!({"base":[5.0,-10.0,0.5],"length":1.0,"width":1.0,"height":2.5})),
            ("box", json!({"base":[14.0,-10.0,0.5],"length":1.0,"width":1.0,"height":2.5})),
            ("box", json!({"base":[5.0,-6.0,0.5],"length":1.0,"width":1.0,"height":2.5})),
            ("box", json!({"base":[14.0,-6.0,0.5],"length":1.0,"width":1.0,"height":2.5})),
            // ceiling/downlight geometry and pendant lights
            ("cylinder", json!({"center":[-12.0,0.0,12.8],"radius":0.8,"height":0.25,"segments":18})),
            ("cylinder", json!({"center":[0.0,0.0,12.8],"radius":0.8,"height":0.25,"segments":18})),
            ("cylinder", json!({"center":[12.0,0.0,12.8],"radius":0.8,"height":0.25,"segments":18})),
            ("cylinder", json!({"center":[10.0,-8.0,9.0],"radius":0.15,"height":3.8,"segments":12})),
            ("sphere", json!({"center":[10.0,-8.0,8.5],"radius":1.0,"segments":16,"rings":8})),
            ("text3d", json!({"at":[-22.0,-16.0,15.0],"text":"INTERIOR + LIGHTING / NOI THAT + CHIEU SANG","height":1.2})),
            ("dim3d", json!({"a":[-24.0,-18.0,0.5],"b":[24.0,-18.0,0.5],"offset":[0.0,0.0,2.0],"textHeight":0.9})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("visualstyle", json!({"name":"ShadedWithEdges"}));
        let _ = app.run("3dorbit", json!({"preset":"seiso","projection":"perspective","distance":100.0}));
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Modeling".into();
        app.ui.show_toolsets = true;
        app.ui.show_viewcube = true;
        app.ui.show_ucs_icon = true;
        app.set_status(&format!("3D interior + lighting demo created: {created}/25 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"3D","evidence":"interior-lighting","features":["room-shell","sofa","tables","tv","chairs","downlights","pendant"]})));
    }

    if id == "ui.demo.3d.facade" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("box", json!({"base":[-28.0,-12.0,0.0],"length":56.0,"width":24.0,"height":0.6})),
            // front facade split around door + windows
            ("box", json!({"base":[-28.0,-12.0,0.6],"length":9.0,"width":0.6,"height":11.0})),
            ("box", json!({"base":[-13.0,-12.0,0.6],"length":10.0,"width":0.6,"height":4.2})),
            ("box", json!({"base":[-13.0,-12.0,8.2],"length":10.0,"width":0.6,"height":3.4})),
            ("box", json!({"base":[3.0,-12.0,0.6],"length":10.0,"width":0.6,"height":4.2})),
            ("box", json!({"base":[3.0,-12.0,8.2],"length":10.0,"width":0.6,"height":3.4})),
            ("box", json!({"base":[19.0,-12.0,0.6],"length":9.0,"width":0.6,"height":11.0})),
            ("box", json!({"base":[-3.0,-12.0,4.6],"length":6.0,"width":0.6,"height":7.0})),
            // side/back walls
            ("box", json!({"base":[-28.0,11.4,0.6],"length":56.0,"width":0.6,"height":11.0})),
            ("box", json!({"base":[-28.0,-12.0,0.6],"length":0.6,"width":24.0,"height":11.0})),
            ("box", json!({"base":[27.4,-12.0,0.6],"length":0.6,"width":24.0,"height":11.0})),
            // balcony slab + railing posts
            ("box", json!({"base":[-12.0,-15.0,6.0],"length":24.0,"width":3.0,"height":0.35})),
            ("cylinder", json!({"center":[-10.0,-14.5,6.35],"radius":0.12,"height":2.0,"segments":10})),
            ("cylinder", json!({"center":[-5.0,-14.5,6.35],"radius":0.12,"height":2.0,"segments":10})),
            ("cylinder", json!({"center":[0.0,-14.5,6.35],"radius":0.12,"height":2.0,"segments":10})),
            ("cylinder", json!({"center":[5.0,-14.5,6.35],"radius":0.12,"height":2.0,"segments":10})),
            ("cylinder", json!({"center":[10.0,-14.5,6.35],"radius":0.12,"height":2.0,"segments":10})),
            ("box", json!({"base":[-10.0,-14.6,8.0],"length":20.0,"width":0.2,"height":0.2})),
            // sloped roof faces with overhang
            ("3dface", json!({"points":[[-30.0,-14.0,11.6],[30.0,-14.0,11.6],[30.0,0.0,16.5],[-30.0,0.0,16.5]]})),
            ("3dface", json!({"points":[[-30.0,0.0,16.5],[30.0,0.0,16.5],[30.0,14.0,11.6],[-30.0,14.0,11.6]]})),
            ("3dface", json!({"points":[[-30.0,-14.0,11.6],[-30.0,14.0,11.6],[-30.0,0.0,16.5]]})),
            ("3dface", json!({"points":[[30.0,-14.0,11.6],[30.0,0.0,16.5],[30.0,14.0,11.6]]})),
            // canopy and facade columns
            ("box", json!({"base":[-6.0,-16.0,4.2],"length":12.0,"width":4.0,"height":0.35})),
            ("cylinder", json!({"center":[-5.0,-15.0,0.6],"radius":0.35,"height":3.6,"segments":18})),
            ("cylinder", json!({"center":[5.0,-15.0,0.6],"radius":0.35,"height":3.6,"segments":18})),
            ("text3d", json!({"at":[-26.0,-18.0,18.0],"text":"FACADE + ROOF DETAIL / MAT DUNG + MAI","height":1.3})),
            ("dim3d", json!({"a":[-28.0,-12.0,0.6],"b":[28.0,-12.0,0.6],"offset":[0.0,0.0,2.5],"textHeight":0.9})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("visualstyle", json!({"name":"ShadedWithEdges"}));
        let _ = app.run("3dorbit", json!({"preset":"seiso","projection":"perspective","distance":115.0}));
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Modeling".into();
        app.ui.show_toolsets = true;
        app.ui.show_viewcube = true;
        app.ui.show_ucs_icon = true;
        app.set_status(&format!("3D facade + roof detail demo created: {created}/27 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"3D","evidence":"facade-roof-detail","features":["facade-openings","balcony","railing","sloped-roof","canopy","columns"]})));
    }

    if id == "ui.demo.3d.site" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("box", json!({"base":[-45.0,-32.0,-0.8],"length":90.0,"width":64.0,"height":0.8})),
            ("box", json!({"base":[-45.0,-6.0,0.0],"length":90.0,"width":12.0,"height":0.25})),
            ("box", json!({"base":[-45.0,-10.0,0.0],"length":90.0,"width":3.0,"height":0.18})),
            ("box", json!({"base":[-45.0,7.0,0.0],"length":90.0,"width":3.0,"height":0.18})),
            ("box", json!({"base":[-16.0,12.0,0.0],"length":32.0,"width":16.0,"height":0.5})),
            // fence posts / gate
            ("cylinder", json!({"center":[-40.0,27.0,0.0],"radius":0.3,"height":2.5,"segments":12})),
            ("cylinder", json!({"center":[-30.0,27.0,0.0],"radius":0.3,"height":2.5,"segments":12})),
            ("cylinder", json!({"center":[30.0,27.0,0.0],"radius":0.3,"height":2.5,"segments":12})),
            ("cylinder", json!({"center":[40.0,27.0,0.0],"radius":0.3,"height":2.5,"segments":12})),
            ("box", json!({"base":[-40.0,26.8,1.0],"length":10.0,"width":0.3,"height":0.25})),
            ("box", json!({"base":[30.0,26.8,1.0],"length":10.0,"width":0.3,"height":0.25})),
            // trees: trunk + canopy
            ("cylinder", json!({"center":[-32.0,16.0,0.0],"radius":0.6,"height":4.0,"segments":14})),
            ("sphere", json!({"center":[-32.0,16.0,6.0],"radius":3.2,"segments":18,"rings":9})),
            ("cylinder", json!({"center":[-24.0,18.0,0.0],"radius":0.55,"height":3.8,"segments":14})),
            ("sphere", json!({"center":[-24.0,18.0,5.8],"radius":3.0,"segments":18,"rings":9})),
            ("cylinder", json!({"center":[26.0,18.0,0.0],"radius":0.55,"height":3.8,"segments":14})),
            ("sphere", json!({"center":[26.0,18.0,5.8],"radius":3.0,"segments":18,"rings":9})),
            ("cylinder", json!({"center":[34.0,16.0,0.0],"radius":0.6,"height":4.0,"segments":14})),
            ("sphere", json!({"center":[34.0,16.0,6.0],"radius":3.2,"segments":18,"rings":9})),
            // garden lamps
            ("cylinder", json!({"center":[-12.0,8.0,0.0],"radius":0.18,"height":3.2,"segments":12})),
            ("sphere", json!({"center":[-12.0,8.0,3.5],"radius":0.45,"segments":12,"rings":6})),
            ("cylinder", json!({"center":[12.0,8.0,0.0],"radius":0.18,"height":3.2,"segments":12})),
            ("sphere", json!({"center":[12.0,8.0,3.5],"radius":0.45,"segments":12,"rings":6})),
            ("text3d", json!({"at":[-42.0,-28.0,5.0],"text":"SITE + LANDSCAPE / CANH QUAN","height":1.6})),
            ("dim3d", json!({"a":[-45.0,-32.0,0.0],"b":[45.0,-32.0,0.0],"offset":[0.0,0.0,4.0],"textHeight":1.0})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("visualstyle", json!({"name":"ShadedWithEdges"}));
        let _ = app.run("3dorbit", json!({"preset":"seiso","projection":"perspective","distance":150.0}));
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Modeling".into();
        app.ui.show_toolsets = true;
        app.ui.show_viewcube = true;
        app.ui.show_ucs_icon = true;
        app.set_status(&format!("3D site + landscape demo created: {created}/25 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"3D","evidence":"site-landscape","features":["site","road","walkway","fence","trees","lights"]})));
    }

    if id == "ui.demo.3d.structuremep" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            // structural slab + four columns
            ("box", json!({"base":[-24.0,-18.0,0.0],"length":48.0,"width":36.0,"height":0.8})),
            ("box", json!({"base":[-18.0,-12.0,0.8],"length":1.2,"width":1.2,"height":10.0})),
            ("box", json!({"base":[16.8,-12.0,0.8],"length":1.2,"width":1.2,"height":10.0})),
            ("box", json!({"base":[-18.0,10.8,0.8],"length":1.2,"width":1.2,"height":10.0})),
            ("box", json!({"base":[16.8,10.8,0.8],"length":1.2,"width":1.2,"height":10.0})),
            // primary beams
            ("box", json!({"base":[-18.0,-12.0,9.6],"length":36.0,"width":1.2,"height":1.2})),
            ("box", json!({"base":[-18.0,10.8,9.6],"length":36.0,"width":1.2,"height":1.2})),
            ("box", json!({"base":[-18.0,-10.8,9.6],"length":1.2,"width":21.6,"height":1.2})),
            ("box", json!({"base":[16.8,-10.8,9.6],"length":1.2,"width":21.6,"height":1.2})),
            // visible HVAC ducts and AHU-like equipment
            ("box", json!({"base":[-14.0,-2.0,7.2],"length":28.0,"width":2.2,"height":1.6})),
            ("box", json!({"base":[10.0,-5.0,6.2],"length":7.0,"width":6.0,"height":3.6})),
            ("box", json!({"base":[-17.0,5.0,6.8],"length":10.0,"width":2.0,"height":1.4})),
            // plumbing / fire risers as native faceted cylinders
            ("cylinder", json!({"center":[-12.0,7.0,0.8],"radius":0.55,"height":8.0,"segments":20})),
            ("cylinder", json!({"center":[-9.5,7.0,0.8],"radius":0.35,"height":8.0,"segments":18})),
            ("cylinder", json!({"center":[13.0,6.0,0.8],"radius":0.45,"height":8.5,"segments":20})),
            ("text3d", json!({"at":[-20.0,-18.0,13.0],"text":"STRUCTURE + MEP / KET CAU + HE THONG","height":1.4})),
            ("leader3d", json!({"arrow":[13.0,-2.0,8.0],"landing":[20.0,-9.0,13.0],"dogleg":[4.0,0.0,0.0],"text":"AHU / HVAC EQUIPMENT","textHeight":1.0})),
            ("dim3d", json!({"a":[-18.0,-12.0,0.8],"b":[18.0,-12.0,0.8],"offset":[0.0,0.0,12.0],"textHeight":1.0})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("visualstyle", json!({"name":"ShadedWithEdges"}));
        let _ = app.run("3dorbit", json!({"preset":"seiso","projection":"perspective","distance":120.0}));
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Modeling".into();
        app.ui.show_toolsets = true;
        app.ui.show_viewcube = true;
        app.ui.show_ucs_icon = true;
        app.set_status(&format!("3D structure + MEP demo created: {created}/18 feature groups"));
        return Some(Ok(json!({"created":created,"workspace":"3D","evidence":"structure-mep","systems":["structure","hvac","plumbing"]})));
    }

    if id == "ui.demo.3d" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("box", json!({"base":[-23.0,-5.0,0.0],"length":10.0,"width":10.0,"height":10.0})),
            ("cylinder", json!({"center":[0.0,0.0,0.0],"radius":5.0,"height":12.0,"segments":24})),
            ("sphere", json!({"center":[18.0,0.0,6.0],"radius":6.0,"segments":24,"rings":12})),
            ("torus", json!({"center":[0.0,18.0,5.0],"majorRadius":7.0,"minorRadius":2.0,"segments":24,"tubeSegments":12})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("visualstyle", json!({"name":"ShadedWithEdges"}));
        let _ = app.run("3dorbit", json!({"preset":"seiso","projection":"perspective","distance":80.0}));
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Modeling".into();
        app.ui.show_toolsets = true;
        app.ui.show_viewcube = true;
        app.ui.show_ucs_icon = true;
        app.set_status(&format!("3D demo scene created: {created}/4 primitives"));
        return Some(Ok(json!({"created":created,"view":"seiso","projection":"perspective"})));
    }


    if id == "ui.demo.3d.house" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            ("box", json!({"base":[-24.0,-16.0,0.0],"length":48.0,"width":32.0,"height":0.6})),
            // front wall split around a real 4 m door opening
            ("box", json!({"base":[-24.0,-16.0,0.6],"length":22.0,"width":0.6,"height":9.0})),
            ("box", json!({"base":[2.0,-16.0,0.6],"length":22.0,"width":0.6,"height":9.0})),
            ("box", json!({"base":[-2.0,-16.0,7.2],"length":4.0,"width":0.6,"height":2.4})),
            // back wall and side walls
            ("box", json!({"base":[-24.0,15.4,0.6],"length":48.0,"width":0.6,"height":9.0})),
            ("box", json!({"base":[-24.0,-15.4,0.6],"length":0.6,"width":30.8,"height":9.0})),
            ("box", json!({"base":[23.4,-15.4,0.6],"length":0.6,"width":30.8,"height":9.0})),
            // interior spine and columns
            ("box", json!({"base":[-0.3,-10.0,0.6],"length":0.6,"width":20.0,"height":9.0})),
            ("box", json!({"base":[-18.0,-10.0,0.6],"length":1.2,"width":1.2,"height":9.0})),
            ("box", json!({"base":[16.8,-10.0,0.6],"length":1.2,"width":1.2,"height":9.0})),
            // gable roof as true sloped 3DFACE geometry
            ("3dface", json!({"points":[[-25.0,-17.0,10.0],[25.0,-17.0,10.0],[25.0,0.0,15.0],[-25.0,0.0,15.0]]})),
            ("3dface", json!({"points":[[-25.0,0.0,15.0],[25.0,0.0,15.0],[25.0,17.0,10.0],[-25.0,17.0,10.0]]})),
            ("3dface", json!({"points":[[-25.0,-17.0,10.0],[-25.0,17.0,10.0],[-25.0,0.0,15.0]]})),
            ("3dface", json!({"points":[[25.0,-17.0,10.0],[25.0,0.0,15.0],[25.0,17.0,10.0]]})),
            ("text3d", json!({"at":[-10.0,-18.0,12.0],"text":"TBT360 HOUSE DEMO","height":1.5})),
            ("dim3d", json!({"a":[-24.0,-16.0,0.0],"b":[24.0,-16.0,0.0],"offset":[0.0,0.0,12.0],"textHeight":1.2})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("visualstyle", json!({"name":"ShadedWithEdges"}));
        let _ = app.run("3dorbit", json!({"preset":"seiso","projection":"perspective","distance":115.0}));
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Modeling".into();
        app.ui.show_toolsets = true;
        app.ui.show_viewcube = true;
        app.ui.show_ucs_icon = true;
        app.set_status(&format!("3D house demo created: {created}/16 feature groups, door opening + gable roof"));
        return Some(Ok(json!({"created":created,"workspace":"3D","evidence":"house","roof":"gable-3dface","opening":"front-door"})));
    }

    if id == "ui.demo.3d.arch" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let jobs = [
            // ground/floor slab
            ("box", json!({"base":[-25.0,-18.0,0.0],"length":50.0,"width":36.0,"height":1.0})),
            // perimeter walls, kept as separate editable faceted assemblies
            ("box", json!({"base":[-25.0,-18.0,1.0],"length":50.0,"width":1.0,"height":10.0})),
            ("box", json!({"base":[-25.0,17.0,1.0],"length":50.0,"width":1.0,"height":10.0})),
            ("box", json!({"base":[-25.0,-17.0,1.0],"length":1.0,"width":34.0,"height":10.0})),
            ("box", json!({"base":[24.0,-17.0,1.0],"length":1.0,"width":34.0,"height":10.0})),
            // four columns
            ("box", json!({"base":[-18.0,-11.0,1.0],"length":1.2,"width":1.2,"height":10.0})),
            ("box", json!({"base":[16.8,-11.0,1.0],"length":1.2,"width":1.2,"height":10.0})),
            ("box", json!({"base":[-18.0,9.8,1.0],"length":1.2,"width":1.2,"height":10.0})),
            ("box", json!({"base":[16.8,9.8,1.0],"length":1.2,"width":1.2,"height":10.0})),
            // upper slab / flat roof datum
            ("box", json!({"base":[-25.0,-18.0,11.0],"length":50.0,"width":36.0,"height":0.8})),
            // entrance canopy and two round facade columns
            ("box", json!({"base":[-8.0,-23.0,4.5],"length":16.0,"width":5.0,"height":0.6})),
            ("cylinder", json!({"center":[-6.0,-21.0,1.0],"radius":0.8,"height":7.0,"segments":20})),
            ("cylinder", json!({"center":[6.0,-21.0,1.0],"radius":0.8,"height":7.0,"segments":20})),
        ];
        let mut created = 0usize;
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("visualstyle", json!({"name":"ShadedWithEdges"}));
        let _ = app.run("3dorbit", json!({"preset":"seiso","projection":"perspective","distance":110.0}));
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Modeling".into();
        app.ui.show_toolsets = true;
        app.ui.show_viewcube = true;
        app.ui.show_ucs_icon = true;
        app.set_status(&format!("Architectural 3D demo created: {created}/13 assemblies"));
        return Some(Ok(json!({"created":created,"workspace":"3D","evidence":"architectural","view":"seiso"})));
    }
    if id == "ui.demo.3d.qa" {
        if app.session.doc().is_err() {
            let _ = app.run("new", json!({"metric": true}));
        }
        let _ = run_ui_command_inner(app, "ui.demo.3d.arch", &Value::Null);
        let mut created = 0usize;
        let jobs = [
            ("sectionplane", json!({"origin":[0.0,0.0,6.0],"normal":[1.0,0.0,0.0],"width":42.0,"height":28.0,"name":"QA Section A"})),
            ("dim3d", json!({"a":[-25.0,-18.0,0.0],"b":[25.0,-18.0,0.0],"offset":[0.0,0.0,14.0],"textHeight":1.4})),
            ("leader3d", json!({"arrow":[0.0,-23.0,5.0],"landing":[12.0,-28.0,14.0],"dogleg":[7.0,0.0,0.0],"text":"Entrance canopy / 3D leader","textHeight":1.2})),
            ("text3d", json!({"at":[-18.0,18.0,16.0],"text":"TBT360 3D QA SHOWCASE","height":1.8})),
            ("pointcloud", json!({"name":"SiteScanDemo","points":[[-12.0,-27.0,0.0],[-8.0,-27.5,0.4],[-4.0,-27.0,0.2],[0.0,-27.5,0.6],[4.0,-27.0,0.2],[8.0,-27.5,0.4],[12.0,-27.0,0.0],[-10.0,-25.0,1.0],[-5.0,-25.5,1.5],[0.0,-25.0,1.2],[5.0,-25.5,1.5],[10.0,-25.0,1.0]]})),
            ("pointcloud.box3d", json!({"name":"SiteScanDemo"})),
        ];
        for (cmd, p) in jobs {
            if app.run(cmd, p).is_ok() { created += 1; }
        }
        let _ = app.run("visualstyle", json!({"name":"ShadedWithEdges"}));
        let _ = app.run("3dorbit", json!({"preset":"seiso","projection":"perspective","distance":125.0}));
        let _ = app.run("zoom.extents", json!({}));
        app.ui.toolset_tab = "Modeling".into();
        app.ui.show_toolsets = true;
        app.ui.show_viewcube = true;
        app.ui.show_ucs_icon = true;
        app.set_status(&format!("3D QA showcase created: architecture + {created}/6 QA feature groups"));
        return Some(Ok(json!({"createdFeatureGroups":created,"workspace":"3D","evidence":"qa-showcase","includes":["architecture","section-plane","3d-dimension","3d-leader","3d-text","point-cloud-bounds"]})));
    }
    if let Some(result) = crate::spacemouse::route(app, id, params) {
        return Some(result);
    }
    if let Some(r) = crate::xrefs::route(app, id, params) {
        return Some(r);
    }
    if let Some(r) = crate::bedit::route(app, id, params) {
        return Some(r);
    }
    if let Some(r) = crate::managers::route(app, id, params) {
        return Some(r);
    }
    if matches!(id, "ui.dock" | "ui.dock.reset") {
        return Some(crate::docking::command(app, id == "ui.dock.reset", params));
    }
    if matches!(id, "ui.panel.properties" | "ui.panel.layers") {
        let panel = if id == "ui.panel.properties" { "properties" } else { "layers" };
        return Some(crate::docking::command(app, false, &json!({"operation":"open", "panel":panel})));
    }
    if id == "ui.toggle.quickcalc" {
        return Some(crate::palettes::quickcalc::toggle(app));
    }
    // Explicit opens remain idempotent; typed QUICKCALC with JSON parameters uses the engine's CAL alias.
    if id == "ui.panel.quickcalc" || (matches!(id, "quickcalc" | "qc") && params.is_null()) {
        return Some(crate::palettes::quickcalc::open(app));
    }
    if let Some(r) = crate::plotstyles::route(app, id, params) {
        return Some(r);
    }
    if let Some(r) = crate::options::route(app, id, params) {
        return Some(r);
    }
    let toggle = |b: &mut bool, p: &Value| {
        *b = p.get("on").and_then(Value::as_bool).unwrap_or(!*b);
    };
    // When a file command should ask for a file. The `ui.*` commands are the pickers; the engine's
    // OPEN/SAVEAS/QSAVE ask only when typed or chosen from a menu (no parameters), so JSON calls
    // never open a picker.
    let no_path =
        if id.starts_with("ui.") { params.is_null() || (params.get("path").is_none() && params.get("data").is_none()) } else { params.is_null() };
    if id == RECENT_CLEAR {
        app.ui.recent_files.clear();
        return Some(Ok(Value::Null));
    }
    if let Some(i) = id.strip_prefix(RECENT_ITEM).and_then(|n| n.parse::<usize>().ok()) {
        let Some(path) = app.ui.recent_files.get(i).cloned() else { return Some(Err(format!("no recent file {i}"))) };
        app.open_path(&path);
        return Some(Ok(Value::Null));
    }
    let r = match id {
        "ui.language" => {
            let Some(preference) = params.get("lang").and_then(Value::as_str).and_then(crate::i18n::Preference::parse) else {
                return Some(Err(format!("language must be {}", crate::i18n::Preference::codes())));
            };
            app.ui.interface_language = preference;
            let shown = app.language();
            if shown != preference.resolve(&app.system_languages) {
                app.set_status(if preference.resolve(&app.system_languages) == "ko" {
                    "No font covers the Korean interface, so it stays in English. Install a Hangul font (Malgun Gothic or Nanum Gothic), or build with a Hang/Kore craft-fonts face."
                } else {
                    "No Chinese font is installed, so the interface stays in English. Install one (Noto Sans CJK, for example) to use Chinese."
                });
            }
            Ok(json!({"interfaceLanguage": preference.code(), "shown": shown}))
        }
        "ui.open" | "open" if no_path => {
            let picked = app.services.pick_open.as_ref().and_then(|f| f());
            if let Some(p) = picked {
                app.open_path(&p);
            }
            Ok(Value::Null)
        }
        // APPLOAD typed or chosen from a menu: the `.lsp` file picker, starting where the last
        // one came from. JSON calls carry `path` and never open one. A picked file is loaded as a
        // typed name is, through the interactive command, so a routine it starts that asks for a
        // point waits for it; on the web (no file system) it says to drop the file instead.
        "appload" | "ap" if params.is_null() => {
            if app.services.no_file_system {
                let hint = "APPLOAD: there is no file system here; drop a .lsp file on the window to load it";
                app.session.echo(hint.to_string());
                app.set_status(hint);
                return Some(Ok(Value::Null));
            }
            let pick = app.services.pick_lisp.as_ref()?;
            let dir = app.ui.last_lisp_dir.clone().unwrap_or_default();
            let Some(path) = pick(&dir) else { return Some(Ok(Value::Null)) };
            if let Some(dir) = std::path::Path::new(&path).parent().and_then(std::path::Path::to_str).filter(|d| !d.is_empty()) {
                app.ui.last_lisp_dir = Some(dir.to_string());
            }
            if let Err(e) = app.session.start("appload") {
                return Some(Err(e.to_string()));
            }
            app.cmdline(&path);
            Ok(Value::Null)
        }
        // Pick a template (.dwt or .dxf) and start a new drawing from it.
        "ui.newtemplate" => {
            if let Some(p) = app.services.pick_open.as_ref().and_then(|f| f()) {
                return Some(app.session.execute("new", &json!({ "template": p })).map_err(|e| e.to_string()));
            }
            Ok(Value::Null)
        }
        // RECOVER from the menu or typed: pick the file. Without a picker the engine asks for the
        // file name on the command line.
        "recover" if no_path && app.services.pick_open.is_some() => {
            if let Some(p) = app.services.pick_open.as_ref().and_then(|f| f()) {
                let r = app.run("recover", json!({ "path": p }));
                app.opened(r);
            }
            Ok(Value::Null)
        }
        "ui.saveas" | "saveas" if no_path => {
            let title = app.session.state().map(|s| s.title.clone()).ok();
            let name = app.ui.save_format.suggested_name(title.as_deref());
            if let Some(p) = app.services.pick_save.as_ref().and_then(|f| f(&name)) {
                return Some(app.session.execute("saveas", &saveas_params(&app.ui, &app.session, &p)).map_err(|e| e.to_string()));
            }
            Ok(Value::Null)
        }
        "qsave" if no_path && app.session.state().is_ok_and(|s| s.path.is_none()) => return run_ui_command(app, "ui.saveas", &Value::Null),
        // EXPORTPDF from the menu or typed: ask where to save the PDF (PLOT and PRINT open the Plot
        // dialog, `managers::route`). JSON calls (scripts, control channel, MCP) always carry
        // params, never get here and never open a dialog.
        "exportpdf" if params.is_null() && app.session.state().is_ok() => {
            let pick = app.services.pick_save.as_ref()?;
            let title = app.session.state().map(|s| s.title.clone()).unwrap_or_default();
            let stem = std::path::Path::new(&title).file_stem().map(|s| s.to_string_lossy().to_string()).filter(|s| !s.is_empty());
            let Some(path) = pick(&format!("{}.pdf", stem.unwrap_or_else(|| "Drawing".into()))) else { return Some(Ok(Value::Null)) };
            let r = app.session.execute("exportpdf", &json!({ "path": path })).map_err(|e| e.to_string());
            app.session.echo(match &r {
                Ok(_) => format!("Plotted to {path}"),
                Err(e) => e.clone(),
            });
            r
        }
        "ui.sample" => {
            let d = cadcraft_engine::sample::default_sample();
            app.session.open_drawing(d, "Bracket", None);
            app.canvas.zoom_pending = true;
            app.ui.start_tab = false;
            Ok(Value::Null)
        }
        "ui.toggle.toolsets" => {
            if params.get("on").and_then(Value::as_bool) == Some(true) {
                return Some(crate::docking::command(app, false, &json!({"operation":"open", "panel":"toolSets"})));
            }
            toggle(&mut app.ui.show_toolsets, params);
            Ok(Value::Null)
        }
        "ui.toggle.palettes" => {
            toggle(&mut app.ui.show_palettes, params);
            Ok(Value::Null)
        }
        "ui.toggle.toolbar" => {
            toggle(&mut app.ui.show_toolbar, params);
            Ok(Value::Null)
        }
        "ui.toggle.filetabs" => {
            toggle(&mut app.ui.show_file_tabs, params);
            Ok(Value::Null)
        }
        // Typed or chosen from a menu or the tool bar: ask for the file (or show the preview).
        "export" | "exp" | "import" | "imp" | "publish" | "preview" | "pre" if params.is_null() && app.session.state().is_ok() => {
            return Some(crate::exchange::route(app, id));
        }
        "ui.toggle.statusbar" => {
            toggle(&mut app.ui.show_status_bar, params);
            Ok(Value::Null)
        }
        "ui.toggle.cmdline" => {
            toggle(&mut app.ui.show_command_line, params);
            Ok(Value::Null)
        }
        "ui.toggle.systemcursor" => {
            toggle(&mut app.ui.system_cursor, params);
            Ok(Value::Null)
        }
        "ui.toggle.viewcube" => {
            toggle(&mut app.ui.show_viewcube, params);
            Ok(Value::Null)
        }
        "ui.toggle.ucsicon" => {
            toggle(&mut app.ui.show_ucs_icon, params);
            Ok(Value::Null)
        }
        "ui.toggle.menubar" => {
            toggle(&mut app.ui.in_window_menu, params);
            Ok(Value::Null)
        }
        "ui.theme.system" | "ui.theme.light" | "ui.theme.dark" => {
            let pref = crate::theme::ThemePref::parse(id.trim_start_matches("ui.theme.")).unwrap_or_default();
            set_theme(app, pref)
        }
        // `{"theme": "system"|"light"|"dark"}` sets the choice; without it, reports it.
        "ui.theme" => match params.get("theme").and_then(Value::as_str) {
            Some(name) => match crate::theme::ThemePref::parse(name) {
                Some(pref) => set_theme(app, pref),
                None => Err(format!("unknown theme \"{name}\" (system, light or dark)")),
            },
            None => Ok(theme_json(app)),
        },
        "ui.animations.system" | "ui.animations.on" | "ui.animations.off" => {
            let pref = crate::animation::AnimationPref::parse(id.trim_start_matches("ui.animations.")).unwrap_or_default();
            Ok(set_animations(app, pref))
        }
        // `{"animations": "system"|"on"|"off"}` sets the choice; without it, reports it.
        "ui.animations" => match params.get("animations").and_then(Value::as_str) {
            Some(name) => match crate::animation::AnimationPref::parse(name) {
                Some(pref) => Ok(set_animations(app, pref)),
                None => Err(format!("unknown animations choice \"{}\" (system, on or off)", name.chars().take(40).collect::<String>())),
            },
            None => Ok(animations_json(app)),
        },
        "ui.saveformat.dxf" | "ui.saveformat.dwg" => {
            let format = crate::SaveFormat::parse(id.trim_start_matches("ui.saveformat.")).unwrap_or_default();
            Ok(set_save_format(app, format))
        }
        // `{"format": "dxf"|"dwg"}` sets the default save format for new drawings; without it,
        // reports it.
        "ui.saveformat" => match params.get("format").and_then(Value::as_str) {
            Some(name) => match crate::SaveFormat::parse(name) {
                Some(format) => Ok(set_save_format(app, format)),
                None => Err(format!("unknown save format \"{name}\" (dxf or dwg)")),
            },
            None => Ok(json!({ "saveFormat": app.ui.save_format.as_str() })),
        },
        // `{"version": "2000"…"2018"}` sets the version new drawings are saved in; without it,
        // reports it.
        id if id.starts_with("ui.saveversion") => {
            let name = id
                .strip_prefix("ui.saveversion.")
                .map(str::to_string)
                .or_else(|| params.get("version").map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_string)));
            match name {
                Some(name) => match crate::SaveVersion::parse(&name) {
                    Some(v) => {
                        app.ui.save_version = v.year().to_string();
                        app.session.echo(format!("New drawings will be saved as version {v}"));
                        Ok(json!({ "saveVersion": app.ui.save_version }))
                    }
                    None => Err(format!("unknown version \"{}\" (2000, 2004, 2007, 2010, 2013 or 2018)", name.chars().take(40).collect::<String>())),
                },
                None => Ok(json!({ "saveVersion": app.ui.save_version })),
            }
        }
        "ui.hidepalettes" => {
            app.ui.show_palettes = false;
            app.ui.show_toolsets = false;
            Ok(Value::Null)
        }
        "ui.resetpalettes" => {
            let (menu, theme, interface_language, save_format) = (app.ui.in_window_menu, app.ui.theme, app.ui.interface_language, app.ui.save_format);
            let (animations, save_version) = (app.ui.animations, std::mem::take(&mut app.ui.save_version));
            app.ui = crate::UiState { in_window_menu: menu, theme, animations, interface_language, save_format, save_version, ..Default::default() };
            Ok(Value::Null)
        }
        "ui.start" => {
            app.ui.start_tab = true;
            Ok(Value::Null)
        }
        // The expanded command history (AutoCAD's text window; TEXTSCR opens it).
        "ui.toggle.history" | "textscr" => {
            let open = json!({"on": true});
            toggle(&mut app.cmd.expanded, if id == "textscr" { &open } else { params });
            Ok(json!({ "on": app.cmd.expanded }))
        }
        // `{"lines": n}`: how many history lines show above the command line (0–12); without it, reports it.
        "ui.cmdline.lines" => {
            if let Some(n) = params.get("lines").and_then(Value::as_u64) {
                app.ui.history_lines = n.min(crate::cmdline::MAX_LINES as u64) as usize;
            }
            Ok(json!({ "lines": app.ui.history_lines }))
        }
        "ui.cmdline.clear" => {
            app.session.log.clear();
            Ok(Value::Null)
        }
        "ui.dialog.language"
        | "ui.dialog.layers"
        | "ui.dialog.blocks"
        | "ui.dialog.dsettings"
        | "ui.dialog.about"
        | "ui.dialog.commands"
        | "ui.dialog.qselect"
        | "ui.dialog.parameters" => {
            app.ui.dialog = Some(id.trim_start_matches("ui.dialog.").to_string());
            Ok(Value::Null)
        }
        "ui.dialog.close" => {
            app.ui.dialog = None;
            Ok(Value::Null)
        }
        "ui.quit" => {
            app.quit_requested = true;
            Ok(Value::Null)
        }
        "ui.noop" => Ok(Value::Null),
        // From a menu or typed, closing asks about unsaved changes; JSON calls close without asking.
        "close" if params.is_null() => {
            app.request_close(None);
            Ok(Value::Null)
        }
        "closeall" if params.is_null() => {
            app.request_close_all();
            Ok(Value::Null)
        }
        "layer" | "la" | "layers" if params.is_null() => {
            app.ui.dialog = Some("layers".into());
            Ok(Value::Null)
        }
        // Typed or menu-invoked (no parameters) these open their dialogs; JSON calls run the command.
        "qselect" | "qs" if params.is_null() => {
            app.ui.dialog = Some("qselect".into());
            Ok(Value::Null)
        }
        "units" | "un" if params.is_null() => {
            app.ui.dialog = Some("units".into());
            Ok(Value::Null)
        }
        "parameters" | "par" if params.is_null() => {
            app.ui.dialog = Some("parameters".into());
            Ok(Value::Null)
        }
        "parametersclose" if params.is_null() => {
            if app.ui.dialog.as_deref() == Some("parameters") {
                app.ui.dialog = None;
            }
            Ok(Value::Null)
        }
        "dsettings" | "ds" | "se" if params.is_null() => {
            app.ui.dialog = Some("dsettings".into());
            Ok(Value::Null)
        }
        // Typed or from a menu (no parameters): the Hatch and Gradient dialog (HPDLGMODE 1) and the
        // Hatch Edit dialog. -HATCH, -HATCHEDIT and JSON calls keep the command line and the JSON form.
        "hatch" | "h" | "bh" | "gradient" | "gd" if params.is_null() && crate::hatchdlg::dialog_on(app) => {
            crate::hatchdlg::open_create(app, matches!(id, "gradient" | "gd"));
            Ok(Value::Null)
        }
        // Typed or from the Tools menu: the Command Aliases dialog. JSON calls keep the JSON form.
        "aliasedit" if params.is_null() => {
            crate::aliasdlg::open(app);
            Ok(Value::Null)
        }
        "hatchedit" | "he" if params.is_null() && app.session.state().is_ok() => {
            crate::hatchdlg::open_edit(app);
            Ok(Value::Null)
        }
        "properties" | "pr" | "props" | "ch" if params.is_null() => {
            crate::docking::command(app, false, &json!({"operation":"open", "panel":"properties"}))
        }
        _ => return None,
    };
    Some(r)
}

/// SAVEAS parameters for a path picked in the Save dialog: a drawing that has no file yet is saved
/// in the default save version; one opened from a file keeps that file's version.
fn saveas_params(ui: &crate::UiState, session: &cadcraft_engine::Session, path: &str) -> Value {
    let untitled = session.state().is_ok_and(|s| s.path.is_none());
    match crate::SaveVersion::parse(&ui.save_version) {
        Some(v) if untitled => json!({ "path": path, "version": v.year() }),
        _ => json!({ "path": path }),
    }
}

fn set_save_format(app: &mut CadApp, format: crate::SaveFormat) -> Value {
    app.ui.save_format = format;
    app.session.echo(format!("New drawings will be saved as {}", format.as_str().to_ascii_uppercase()));
    json!({ "saveFormat": format.as_str() })
}

fn set_theme(app: &mut CadApp, pref: crate::theme::ThemePref) -> Result<Value, String> {
    app.ui.theme = pref;
    Ok(theme_json(app))
}

/// The theme choice and, for an explicit choice, the theme it shows (System resolves on the next
/// frame against the OS appearance; see `ui.inspect` → `theme`).
fn theme_json(app: &CadApp) -> Value {
    let shown = match app.ui.theme {
        crate::theme::ThemePref::System => app.shown_theme(),
        p => p.resolve(None),
    };
    json!({ "theme": app.ui.theme.as_str(), "shown": if shown == egui::Theme::Light { "light" } else { "dark" } })
}

fn set_animations(app: &mut CadApp, pref: crate::animation::AnimationPref) -> Value {
    app.ui.animations = pref;
    let shown = if app.shows_animations() { "on" } else { "off" };
    app.session.echo(match pref {
        crate::animation::AnimationPref::System => format!("Animation effects follow the system setting ({shown})"),
        _ => format!("Animation effects are {shown}"),
    });
    animations_json(app)
}

/// The animation effects choice and whether the interface animates with it (System resolves
/// against the OS setting).
fn animations_json(app: &CadApp) -> Value {
    json!({ "animations": app.ui.animations.as_str(), "shown": app.shows_animations() })
}

/// A menu entry.
#[derive(Clone, Debug)]
pub enum Entry {
    Item { label: String, id: String, shortcut: Option<String>, enabled: bool },
    Sub { label: String, children: Vec<Entry> },
}

/// The full menu tree from the command registry plus UI commands, in registration order.
pub fn tree(app: &CadApp) -> Vec<(String, Vec<Entry>)> {
    let mut out: Vec<(String, Vec<Entry>)> = MENUS.iter().map(|m| (m.to_string(), Vec::new())).collect();
    let mut add = |path: &[&str], label: &str, id: &str, shortcut: Option<&str>, enabled: bool| {
        let Some((top, rest)) = path.split_first() else { return };
        let Some((_, entries)) = out.iter_mut().find(|(m, _)| m == top) else { return };
        let mut cur = entries;
        let n = rest.len();
        for (i, seg) in rest.iter().enumerate() {
            if i + 1 == n {
                cur.push(Entry::Item {
                    label: if seg.is_empty() { label.to_string() } else { seg.to_string() },
                    id: id.to_string(),
                    shortcut: shortcut.map(str::to_string),
                    enabled,
                });
            } else {
                let pos = cur.iter().position(|e| matches!(e, Entry::Sub { label, .. } if label == seg));
                let idx = match pos {
                    Some(p) => p,
                    None => {
                        cur.push(Entry::Sub { label: seg.to_string(), children: Vec::new() });
                        cur.len() - 1
                    }
                };
                let Some(Entry::Sub { children, .. }) = cur.get_mut(idx) else { return };
                cur = children;
            }
        }
    };
    for c in cadcraft_engine::command_specs() {
        if !c.menu.is_empty() {
            let enabled = (c.enabled)(&app.session).is_ok();
            add(c.menu, c.label, c.id, c.shortcut, enabled);
            if c.menu.len() >= 2 && c.menu[0] == "Draw" && c.menu[1] == "3D Modeling" {
                let mut path = Vec::with_capacity(c.menu.len() - 1);
                path.push("3D");
                path.extend_from_slice(&c.menu[2..]);
                add(&path, c.label, c.id, c.shortcut, enabled);
            }
        }
    }
    for (id, label, menu, sc) in UI_COMMANDS {
        if !menu.is_empty() {
            add(menu, label, id, *sc, true);
        }
    }
    if let Some((_, file)) = out.iter_mut().find(|(m, _)| m == "File") {
        let at = file.iter().position(|e| matches!(e, Entry::Item { id, .. } if id == "open")).map_or(0, |i| i + 1);
        file.insert(at, Entry::Sub { label: OPEN_RECENT.into(), children: recent_entries(app) });
    }
    out
}

/// The items of File ▸ Open Recent: the recent files by name, newest first, then Clear Recent Files.
pub fn recent_entries(app: &CadApp) -> Vec<Entry> {
    let mut entries: Vec<Entry> = app
        .ui
        .recent_files
        .iter()
        .enumerate()
        .map(|(i, path)| Entry::Item { label: file_name(path), id: format!("{RECENT_ITEM}{i}"), shortcut: None, enabled: true })
        .collect();
    let enabled = !app.ui.recent_files.is_empty();
    entries.push(Entry::Item { label: "Clear Recent Files".into(), id: RECENT_CLEAR.into(), shortcut: None, enabled });
    entries
}

/// The file name of `path` (the whole path when it has none).
pub(crate) fn file_name(path: &str) -> String {
    std::path::Path::new(path).file_name().map_or_else(|| path.to_string(), |n| n.to_string_lossy().to_string())
}

/// Turn a registry shortcut ("Cmd+Shift+Z") into the form shown on this platform: macOS symbols ("⇧⌘Z") or
/// Windows/Linux text ("Ctrl+Shift+Z"). `mac` comes from the runtime OS (`ctx.os()`), so the web build is right too.
pub(crate) fn shortcut_label(s: &str, mac: bool) -> String {
    if !mac {
        return s.replace("Cmd+", "Ctrl+");
    }
    let shift = if s.contains("Shift+") { "⇧" } else { "" };
    let cmd = if s.contains("Cmd+") { "⌘" } else { "" };
    format!("{shift}{cmd}{}", s.replace("Cmd+", "").replace("Shift+", ""))
}

pub(crate) fn entry_ui(ui: &mut egui::Ui, e: &Entry, clicked: &mut Option<String>) {
    match e {
        Entry::Item { label, id, shortcut, enabled } => {
            let mut b = egui::Button::new(crate::i18n::t(label));
            if let Some(s) = shortcut {
                b = b.shortcut_text(shortcut_label(s, ui.ctx().os().is_mac()));
            }
            if ui.add_enabled(*enabled, b).clicked() {
                *clicked = Some(id.clone());
                ui.close();
            }
        }
        Entry::Sub { label, children } => {
            ui.menu_button(crate::i18n::menu(label), |ui| {
                for c in children {
                    entry_ui(ui, c, clicked);
                }
            });
        }
    }
}

pub fn menu_bar(app: &mut CadApp, ui: &mut egui::Ui) {
    let t = crate::theme::Tokens::get();
    let tree = tree(app);
    let mut clicked = None;
    egui::Panel::top("cc_menubar").exact_size(22.0).frame(egui::Frame::NONE.fill(t.chrome_dark).inner_margin(egui::Margin::symmetric(6, 0))).show(
        ui,
        |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                // Titles first, so a hovered title can take over before any menu is drawn.
                let titles: Vec<egui::Response> = tree.iter().map(|(name, _)| ui.add(egui::Button::new(crate::i18n::menu(name)))).collect();
                switch_on_hover(ui.ctx(), &titles);
                for ((_, entries), title) in tree.iter().zip(&titles) {
                    egui::Popup::menu(title).show(|ui| {
                        for e in entries {
                            entry_ui(ui, e, &mut clicked);
                        }
                    });
                }
            });
        },
    );
    if let Some(id) = clicked {
        activate(app, &id);
    }
}

/// While one menu of the bar is open, pointing at another title opens that menu instead.
fn switch_on_hover(ctx: &egui::Context, titles: &[egui::Response]) {
    let is_open = |title: &egui::Response| egui::Popup::is_id_open(ctx, egui::Popup::default_response_id(title));
    if !titles.iter().any(is_open) {
        return;
    }
    // A click on a title toggles its menu by itself.
    if let Some(title) = titles.iter().find(|t| t.hovered() && !t.clicked() && !is_open(t)) {
        egui::Popup::open_id(ctx, egui::Popup::default_response_id(title));
    }
}

/// Activate a menu command. QuickCalc toggles transparently; other items clear pending
/// command-line text and start their command interactively.
pub fn activate(app: &mut CadApp, id: &str) {
    if id == "ui.toggle.quickcalc" {
        run_transparent(app, id);
        return;
    }
    app.cmd.buffer.clear();
    app.start(id);
}

/// Activate the menu command `id` from a native menu bar, which takes the Edit menu's key
/// equivalents (⌘C, ⌘X, ⌘V, ⌘A, ⌘Z, ⇧⌘Z) before any window sees them: while a text field has
/// the keyboard, in the main window or a floating panel's own window, the field gets the key
/// and the drawing command does not run.
pub fn activate_native(app: &mut CadApp, ctx: &egui::Context, id: &str) {
    let (Some(window), Some(text)) = (text_field_window(ctx), text_command(id, ctx.os().is_mac())) else {
        activate(app, id);
        return;
    };
    match text {
        TextCommand::Paste => ctx.send_viewport_cmd_to(window, egui::ViewportCommand::RequestPaste),
        TextCommand::Events(events) if window == egui::ViewportId::ROOT => {
            app.synthetic.extend(events);
            ctx.request_repaint();
        }
        TextCommand::Events(events) => craft_ui::docking::send_keys(ctx, window, events),
    }
}

/// The window whose text field has the keyboard: a floating panel's own OS window, or the main
/// window.
pub fn text_field_window(ctx: &egui::Context) -> Option<egui::ViewportId> {
    craft_ui::docking::typing_window(ctx).or_else(|| ctx.text_edit_focused().then_some(egui::ViewportId::ROOT))
}

/// Whether a focused text field takes the menu command `id` ([`activate_native`]): a native
/// menu keeps such items enabled while one has the keyboard, whatever the drawing allows.
pub fn edits_text(id: &str) -> bool {
    text_command(id, false).is_some()
}

/// What a focused text field does for an Edit menu command.
#[derive(Debug, PartialEq)]
enum TextCommand {
    /// The clipboard text, which the window asks for and receives as `Paste`.
    Paste,
    Events(Vec<egui::Event>),
}

fn text_command(id: &str, mac: bool) -> Option<TextCommand> {
    let key = |key, shift| {
        let modifiers = egui::Modifiers { mac_cmd: mac, ctrl: !mac, command: true, shift, alt: false };
        TextCommand::Events([true, false].map(|pressed| egui::Event::Key { key, physical_key: None, pressed, repeat: false, modifiers }).into())
    };
    Some(match id {
        "copyclip" => TextCommand::Events(vec![egui::Event::Copy]),
        "cutclip" => TextCommand::Events(vec![egui::Event::Cut]),
        "pasteclip" => TextCommand::Paste,
        "u" => key(egui::Key::Z, false),
        "redo" => key(egui::Key::Z, true),
        "selectall" => key(egui::Key::A, false),
        _ => return None,
    })
}

/// Run `id` with default parameters without cancelling the active command, as the drafting
/// toggles (F3, F7-F12, the status bar buttons) are in AutoCAD.
pub fn run_transparent(app: &mut CadApp, id: &str) {
    match app.session.execute(id, &json!({})) {
        Ok(v) => {
            if let Some(m) = v.get("message").and_then(Value::as_str) {
                app.session.echo(m.to_string());
            }
        }
        Err(_) => {
            let _ = app.run(id, json!({}));
        }
    }
}

/// Keyboard shortcuts and function keys.
pub fn shortcuts(app: &mut CadApp, ctx: &egui::Context) {
    use egui::{Key, KeyboardShortcut, Modifiers};
    let sc = |m: Modifiers, k: Key| KeyboardShortcut::new(m, k);
    let cmd = Modifiers::COMMAND;
    let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
    let pairs: &[(KeyboardShortcut, &str)] = &[
        (sc(cmd_shift, Key::A), "pickstyle"),
        (sc(cmd_shift, Key::Z), "redo"),
        (sc(cmd, Key::Z), "u"),
        (sc(cmd, Key::Y), "redo"),
        (sc(cmd, Key::N), "new"),
        (sc(cmd, Key::O), "ui.open"),
        (sc(cmd_shift, Key::S), "ui.saveas"),
        (sc(cmd, Key::S), "qsave"),
        (sc(cmd, Key::P), "plot"),
        (sc(cmd, Key::A), "selectall"),
        (sc(cmd_shift, Key::C), "copybase"),
        (sc(cmd, Key::C), "copyclip"),
        (sc(cmd, Key::X), "cutclip"),
        (sc(cmd_shift, Key::V), "pasteblock"),
        (sc(cmd, Key::V), "pasteclip"),
        (sc(cmd, Key::Num1), "ui.panel.properties"),
        (sc(cmd, Key::Num3), "ui.toggle.toolsets"),
        (sc(cmd, Key::Num9), "ui.toggle.cmdline"),
        (sc(Modifiers::NONE, Key::F1), "ui.dialog.commands"),
        (sc(Modifiers::NONE, Key::F2), "ui.toggle.history"),
        (sc(Modifiers::NONE, Key::F3), "osnap"),
        (sc(Modifiers::NONE, Key::F5), "isoplane"),
        (sc(cmd, Key::E), "isoplane"),
        (sc(Modifiers::NONE, Key::F7), "grid"),
        (sc(Modifiers::NONE, Key::F8), "ortho"),
        (sc(Modifiers::NONE, Key::F9), "snap"),
        (sc(Modifiers::NONE, Key::F10), "polar"),
        (sc(Modifiers::NONE, Key::F11), "otrack"),
        (sc(Modifiers::NONE, Key::F12), "dynmode"),
    ];
    crate::clipboard::mirror(app, ctx);
    if app.closing.is_some() {
        return;
    }
    // A palette toggle is global even while its expression field owns keyboard focus.
    if ctx.input_mut(|i| i.consume_shortcut(&sc(cmd, Key::Num8))) {
        run_transparent(app, "ui.toggle.quickcalc");
    }
    if ctx.egui_wants_keyboard_input() {
        return;
    }
    // Cmd/Ctrl+C, X and V arrive as clipboard events, not key presses.
    let mut fire = crate::clipboard::shortcut_commands(app, ctx);
    ctx.input_mut(|i| {
        for (s, id) in pairs {
            if i.consume_shortcut(s) && !fire.contains(id) {
                fire.push(*id);
            }
        }
    });
    for id in fire {
        // ISOPLANE (F5, Ctrl+E) switches the plane without cancelling the active command.
        if id == "isoplane" {
            if let Some(m) = app.session.execute(id, &json!({})).ok().and_then(|v| v.get("message").and_then(Value::as_str).map(str::to_string)) {
                app.session.echo(m);
            }
            continue;
        }
        // Toggles run transparently (they don't cancel the active command).
        let transparent = ["osnap", "grid", "ortho", "snap", "polar", "otrack", "dynmode", "pickstyle"].contains(&id);
        if transparent || id.starts_with("ui.") {
            run_transparent(app, id);
        } else {
            activate(app, id);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use cadcraft_engine::Session;
    use cadcraft_engine::cmd::file::{IoHooks, io, set_io};
    use serde_json::{Value, json};

    use crate::{CadApp, Services};

    fn fake_plot(_: &cadcraft_doc::Drawing, _: &cadcraft_doc::Space, _: &Value) -> Result<Vec<u8>, String> {
        Ok(b"%PDF-1.4 test".to_vec())
    }

    /// An app whose save dialog records the suggested name and answers `answer`.
    fn app_with_picker(answer: Option<String>) -> (CadApp, Rc<RefCell<Vec<String>>>) {
        let asked = Rc::new(RefCell::new(Vec::new()));
        let log = Rc::clone(&asked);
        let services = Services {
            pick_open: None,
            pick_save: Some(Box::new(move |name: &str| {
                log.borrow_mut().push(name.to_string());
                answer.clone()
            })),
            ..Services::default()
        };
        (CadApp::new(Session::new(), services), asked)
    }

    #[test]
    fn plot_and_print_open_the_plot_dialog_and_exportpdf_asks_for_a_file() {
        set_io(IoHooks { read: |_, _| Err("no reader in tests".into()), write: |_, _| Err("no writer in tests".into()), plot: Some(fake_plot) });
        if io().and_then(|h| h.plot).is_none() {
            return; // another test installed hooks without a plotter first
        }
        let path = std::env::temp_dir().join(format!("cadcraft-print-test-{}.pdf", std::process::id()));
        let ps = path.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&path);
        let (mut app, asked) = app_with_picker(Some(ps.clone()));

        // Toolbar, menu, Cmd+P and typed PRINT open the Plot dialog; its OK asks for the file.
        app.start("plot");
        assert_eq!(app.ui.dialog.as_deref(), Some("plot"));
        app.ui.dialog = None;
        app.cmdline("print");
        assert_eq!(app.ui.dialog.as_deref(), Some("plot"));
        assert!(asked.borrow().is_empty());

        // EXPORTPDF asks where to save straight away.
        app.start("exportpdf");
        assert_eq!(*asked.borrow(), vec!["Drawing1.pdf".to_string()]);
        assert!(std::fs::read(&path).unwrap().starts_with(b"%PDF"));
        assert!(app.session.log.iter().any(|l| l.contains(&ps)));
        let _ = std::fs::remove_file(&path);

        // Programmatic calls never open a dialog.
        app.ui.dialog = None;
        let r = app.run("plot", json!({})).unwrap();
        assert!(r["data"].is_string());
        assert_eq!(asked.borrow().len(), 1);
        assert!(app.ui.dialog.is_none());
    }

    #[test]
    fn cancelled_export_writes_nothing() {
        let (mut app, asked) = app_with_picker(None);
        app.start("exportpdf");
        assert_eq!(asked.borrow().len(), 1);
        assert!(app.session.running.is_none());
    }

    #[test]
    fn floorplan_demo_creates_visible_2d_evidence() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.2d.floorplan", &Value::Null)
            .expect("ui command")
            .expect("floorplan");
        assert_eq!(r["evidence"], "floor-plan");
        assert!(app.session.doc().unwrap().model.len() >= 18);
    }

    #[test]
    fn house_demo_creates_sloped_roof_and_3d_annotation() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.3d.house", &Value::Null)
            .expect("ui command")
            .expect("house");
        assert_eq!(r["evidence"], "house");
        let d = app.session.doc().unwrap();
        assert!(d.model.len() > 25);
        assert!(d.model.iter().any(|e| matches!(e.kind, cadcraft_doc::EntityKind::Face3d(_))));
        assert!(d.model.iter().any(|e| e.common.xdata.iter().any(|t| t.value == "TBT360_3DANNOTATION")));
    }

    #[test]
    fn section_demo_creates_hatch_dimension_and_annotation() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.2d.section", &Value::Null)
            .expect("ui command")
            .expect("section demo");
        assert_eq!(r["evidence"], "section");
        assert!(r["hatch"].as_bool().unwrap_or(false));
        assert!(app.session.doc().unwrap().model.len() >= 10);
    }

    #[test]
    fn structure_mep_demo_creates_visible_3d_systems() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.3d.structuremep", &Value::Null)
            .expect("ui command")
            .expect("structure mep demo");
        assert_eq!(r["evidence"], "structure-mep");
        let d = app.session.doc().unwrap();
        assert!(d.model.len() > 40);
        assert!(d.model.iter().any(|e| matches!(e.kind, cadcraft_doc::EntityKind::Face3d(_))));
        assert!(d.model.iter().any(|e| e.common.xdata.iter().any(|t| t.value == "TBT360_3DANNOTATION")));
    }


    #[test]
    fn elevation_demo_creates_visible_2d_facade_evidence() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.2d.elevation", &Value::Null)
            .expect("ui command")
            .expect("elevation demo");
        assert_eq!(r["evidence"], "elevation");
        assert!(app.session.doc().unwrap().model.len() >= 14);
    }

    #[test]
    fn mep_plan_demo_creates_visible_2d_systems() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.2d.mep", &Value::Null)
            .expect("ui command")
            .expect("mep demo");
        assert_eq!(r["evidence"], "mep-plan");
        assert!(app.session.doc().unwrap().model.len() >= 13);
    }

    #[test]
    fn site_landscape_demo_creates_visible_3d_assets() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.3d.site", &Value::Null)
            .expect("ui command")
            .expect("site demo");
        assert_eq!(r["evidence"], "site-landscape");
        let d = app.session.doc().unwrap();
        assert!(d.model.len() > 80);
        assert!(d.model.iter().any(|e| matches!(e.kind, cadcraft_doc::EntityKind::Face3d(_))));
        assert!(d.model.iter().any(|e| e.common.xdata.iter().any(|t| t.value == "TBT360_3DANNOTATION")));
    }

    #[test]
    fn qa_showcase_creates_visible_3d_evidence() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.3d.qa", &Value::Null)
            .expect("ui command")
            .expect("showcase");
        assert_eq!(r["workspace"], "3D");
        assert_eq!(r["evidence"], "qa-showcase");
        let d = app.session.doc().unwrap();
        assert!(d.model.len() > 40, "showcase should create substantial visible geometry");
        assert!(d.model.iter().any(|e| e.common.xdata.iter().any(|t| t.value == "TBT360_SECTION")));
        assert!(d.model.iter().any(|e| e.common.xdata.iter().any(|t| t.value == "TBT360_POINTCLOUD")));
        assert!(d.model.iter().any(|e| e.common.xdata.iter().any(|t| t.value == "TBT360_3DANNOTATION")));
    }

    #[test]
    fn structural_plan_demo_creates_grid_columns_beams_and_dimensions() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.2d.structural", &Value::Null)
            .expect("ui command")
            .expect("structural plan");
        assert_eq!(r["evidence"], "structural-plan");
        assert!(app.session.doc().unwrap().model.len() >= 24);
    }

    #[test]
    fn ceiling_lighting_plan_demo_creates_visible_lighting_layout() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.2d.rcp", &Value::Null)
            .expect("ui command")
            .expect("ceiling lighting plan");
        assert_eq!(r["evidence"], "ceiling-lighting-plan");
        assert!(app.session.doc().unwrap().model.len() >= 20);
    }

    #[test]
    fn interior_lighting_demo_creates_visible_3d_assets() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.3d.interior", &Value::Null)
            .expect("ui command")
            .expect("interior lighting demo");
        assert_eq!(r["evidence"], "interior-lighting");
        let d = app.session.doc().unwrap();
        assert!(d.model.len() > 60);
        assert!(d.model.iter().any(|e| matches!(e.kind, cadcraft_doc::EntityKind::Face3d(_))));
        assert!(d.model.iter().any(|e| e.common.xdata.iter().any(|t| t.value == "TBT360_3DANNOTATION")));
    }
    #[test]
    fn plumbing_fire_plan_demo_creates_visible_systems() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.2d.plumbingfire", &Value::Null)
            .expect("ui command")
            .expect("plumbing fire plan");
        assert_eq!(r["evidence"], "plumbing-fire-plan");
        assert!(app.session.doc().unwrap().model.len() >= 18);
    }

    #[test]
    fn door_window_schedule_demo_creates_visible_schedule() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.2d.schedule", &Value::Null)
            .expect("ui command")
            .expect("door window schedule");
        assert_eq!(r["evidence"], "door-window-schedule");
        assert!(app.session.doc().unwrap().model.len() >= 30);
    }

    #[test]
    fn facade_roof_detail_demo_creates_visible_3d_facade() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let r = super::run_ui_command(&mut app, "ui.demo.3d.facade", &Value::Null)
            .expect("ui command")
            .expect("facade roof detail");
        assert_eq!(r["evidence"], "facade-roof-detail");
        let d = app.session.doc().unwrap();
        assert!(d.model.len() > 50);
        assert!(d.model.iter().any(|e| matches!(e.kind, cadcraft_doc::EntityKind::Face3d(_))));
        assert!(d.model.iter().any(|e| e.common.xdata.iter().any(|t| t.value == "TBT360_3DANNOTATION")));
    }

}

#[cfg(test)]
mod shortcut_label_tests {
    use super::shortcut_label;

    #[test]
    fn shortcut_label_per_platform() {
        assert_eq!(shortcut_label("Cmd+Shift+Z", true), "⇧⌘Z");
        assert_eq!(shortcut_label("Cmd+Z", true), "⌘Z");
        assert_eq!(shortcut_label("Cmd+Shift+Z", false), "Ctrl+Shift+Z");
        assert_eq!(shortcut_label("Cmd+Z", false), "Ctrl+Z");
        assert_eq!(shortcut_label("Shift+F3", false), "Shift+F3");
        assert_eq!(shortcut_label("Shift+F3", true), "⇧F3");
        assert_eq!(shortcut_label("F1", false), "F1");
        assert_eq!(shortcut_label("F1", true), "F1");
    }
}

#[cfg(test)]
mod units_dialog_tests {
    use cadcraft_engine::Session;
    use serde_json::json;

    use crate::{CadApp, Services};

    #[test]
    fn units_typed_opens_dialog_json_runs_command() {
        let mut app = CadApp::new(Session::new(), Services::default());
        app.cmdline("UNITS");
        assert_eq!(app.ui.dialog.as_deref(), Some("units"));
        app.ui.dialog = None;
        app.start("un");
        assert_eq!(app.ui.dialog.as_deref(), Some("units"));
        app.ui.dialog = None;
        let r = app.run("units", json!({ "lunits": 4, "luprec": 3, "insunits": 4 })).unwrap();
        assert_eq!(r["lunits"], 4);
        assert!(app.ui.dialog.is_none(), "JSON calls never open dialogs");
        let h = &app.session.doc().unwrap().header;
        assert_eq!((h.i64("LUNITS", 0), h.i64("LUPREC", 0), h.i64("INSUNITS", 0)), (4, 3, 4));
    }
}
