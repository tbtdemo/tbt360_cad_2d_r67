//! Drafting settings and system variables: ORTHO, OSNAP, GRID, SNAP, POLAR, SETVAR/GETVAR.

use serde_json::{Value, json};

use super::{CommandSpec, always, bad, point_param, str_param};

use crate::{Accept, Input, Interactive, Prompt, Result, Session, Step};

pub(super) mod iso;

fn toggle(s: &mut Session, p: &Value, get: fn(&mut crate::Settings) -> &mut bool, name: &str) -> Result<Value> {
    let v = get(&mut s.settings);
    *v = p.get("on").and_then(Value::as_bool).unwrap_or(!*v);
    let on = *v;
    s.touch();
    Ok(json!({ "on": on, "message": format!("<{name} {}>", if on { "on" } else { "off" }) }))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("ortho", "Ortho Mode", |s, p| toggle(s, p, |st| &mut st.orthomode, "Ortho"))
            .key("F8")
            .params("{on?: bool}")
            .enabled(always)
            .noundo()
            .transparent(),
        CommandSpec::new("grid", "Grid Display", |s, p| toggle(s, p, |st| &mut st.gridmode, "Grid"))
            .key("F7")
            .params("{on?: bool}")
            .enabled(always)
            .noundo()
            .transparent(),
        CommandSpec::new("snap", "Snap Mode", |s, p| toggle(s, p, |st| &mut st.snapmode, "Snap"))
            .key("F9")
            .params("{on?: bool}")
            .enabled(always)
            .noundo()
            .transparent(),
        CommandSpec::new("polar", "Polar Tracking", |s, p| toggle(s, p, |st| &mut st.polarmode, "Polar"))
            .key("F10")
            .params("{on?: bool}")
            .enabled(always)
            .noundo()
            .transparent(),
        CommandSpec::new("otrack", "Object Snap Tracking", |s, p| toggle(s, p, |st| &mut st.otrack, "Object Snap Tracking"))
            .key("F11")
            .params("{on?: bool}")
            .enabled(always)
            .noundo()
            .transparent(),
        CommandSpec::new("dynmode", "Dynamic Input", |s, p| toggle(s, p, |st| &mut st.dynmode, "Dynamic Input"))
            .key("F12")
            .params("{on?: bool}")
            .enabled(always)
            .noundo()
            .transparent(),
        CommandSpec::new("lwdisplay", "Show/Hide Lineweight", |s, p| toggle(s, p, |st| &mut st.lwdisplay, "Lineweight"))
            .params("{on?: bool}")
            .enabled(always)
            .noundo()
            .transparent(),
        CommandSpec::new("isodraft", "Isometric Drafting", iso::run_isodraft)
            .params("{on?: bool, plane?: \"left\"|\"top\"|\"right\"|\"orthographic\"} (neither: toggle)")
            .enabled(always)
            .noundo()
            .transparent()
            .interactive(|_| Ok(Box::new(iso::IsodraftM))),
        CommandSpec::new("isoplane", "Isometric Plane", iso::run_isoplane)
            .key("F5")
            .params("{plane?: \"left\"|\"top\"|\"right\"} (none: the next plane)")
            .enabled(always)
            .noundo()
            .transparent()
            .interactive(|_| Ok(Box::new(iso::IsoplaneM))),
        CommandSpec::new("transparencydisplay", "Show/Hide Transparency", |s, p| toggle(s, p, |st| &mut st.transparency_display, "Transparency"))
            .params("{on?: bool}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("qpmode", "Quick Properties", |s, p| toggle(s, p, |st| &mut st.qpmode, "Quick Properties"))
            .params("{on?: bool}")
            .enabled(always)
            .noundo()
            .transparent(),
        CommandSpec::new("selectioncycling", "Selection Cycling", |s, p| toggle(s, p, |st| &mut st.selection_cycling, "Selection Cycling"))
            .params("{on?: bool}")
            .enabled(always)
            .noundo(),
        CommandSpec::new("visualstyle", "Visual Style", run_visualstyle)
            .alias(&["vscurrent", "vstyle", "visualstyles"])
            .params("{name?: 2dwireframe|wireframe|hidden|conceptual|realistic|shaded|shadedwithedges|sketchy|xray} (none: query)")
            .enabled(always)
            .noundo(),
        CommandSpec::new("material", "Material", run_material)
            .alias(&["materials"])
            .params("{name?: string, color?: #RRGGBB, roughness?: 0..1, metallic?: 0..1} (none: query current material)")
            .enabled(always),
        CommandSpec::new("light", "Directional Light", run_light)
            .alias(&["lights"])
            .params("{direction?: [x,y,z], intensity?: 0..10} (none: query)")
            .enabled(always),
        CommandSpec::new("render", "Render Preview", run_render)
            .params("{} -> current visual/material/light render state")
            .enabled(always)
            .noundo(),
        CommandSpec::new("backfacecull", "Back-face Culling", run_backfacecull)
            .alias(&["bfcull"])
            .params("{on?: bool} (none: toggle) -> persisted faceted 3D viewport culling")
            .enabled(always),
        CommandSpec::new("osnap", "Object Snap", run_osnap)
            .key("F3")
            .alias(&["os", "ddosnap"])
            .params("{on?: bool, modes?: [\"end\",\"mid\",...] | osmode?: n}")
            .enabled(always)
            .noundo()
            .transparent(),
        CommandSpec::new("dsettings", "Drafting Settings...", run_dsettings)
            .menu(&["Tools", "Drafting Settings..."])
            .alias(&["ds", "se"])
            .params("{snapunit?: [x,y], gridunit?: [x,y], polarang? (degrees), gridmajor?}")
            .enabled(always),
        CommandSpec::new("setvar", "Set Variable", run_setvar)
            .menu(&["Tools", "Inquiry", "Set Variable"])
            .alias(&["set"])
            .params("{name, value}")
            .transparent()
            .interactive(|_| Ok(Box::new(SetvarM::default()))),
        CommandSpec::new("getvar", "Get Variable", run_getvar).params("{name}").enabled(always).noundo(),
        CommandSpec::new("sysvars", "List System Variables", |s, _| Ok(crate::sysvars::list(s))).enabled(always).noundo(),
    ]
}

/// The document-level visual style selector. The 3D facet renderer consumes this setting
/// for wireframe/shaded display while keeping all 2D entities unchanged.
fn run_visualstyle(s: &mut Session, p: &Value) -> Result<Value> {
    let requested = p.get("name").or_else(|| p.get("style")).and_then(Value::as_str);
    let current = s.doc()?.header.str("VISUALSTYLE", "2DWireframe").to_string();
    let style = match requested {
        None => current,
        Some(name) => {
            let canonical = normalize_visualstyle(name).ok_or_else(|| {
                bad("visualstyle", "unsupported style; use 2DWireframe, Wireframe, Hidden, Conceptual, Realistic, Shaded, ShadedWithEdges, Sketchy, or X-ray")
            })?;
            s.doc_mut()?.header.set_str("VISUALSTYLE", canonical);
            s.touch();
            canonical.to_string()
        }
    };
    Ok(json!({ "visualStyle": style, "persisted": true, "rendered": true, "representation": "faceted-3d" }))
}

fn run_backfacecull(s: &mut Session, p: &Value) -> Result<Value> {
    let current = s.doc()?.header.i64("BACKFACECULL", 0) != 0;
    let on = p.get("on").and_then(Value::as_bool).unwrap_or(!current);
    s.doc_mut()?.header.set_i64("BACKFACECULL", if on { 1 } else { 0 });
    s.touch();
    Ok(json!({"on": on, "persisted": true, "representation": "faceted-3d"}))
}

fn run_material(s: &mut Session, p: &Value) -> Result<Value> {
    if let Some(name) = p.get("name").and_then(Value::as_str) {
        let name = name.trim();
        if name.is_empty() || name.len() > 128 { return Err(bad("material", "name must contain 1..128 characters")); }
        s.doc_mut()?.header.set_str("MATERIALCURRENT", name);
    }
    if let Some(color) = p.get("color").and_then(Value::as_str) {
        if cadcraft_color::Rgb::parse_hex(color).is_none() { return Err(bad("material", "color must be #RRGGBB")); }
        s.doc_mut()?.header.set_str("MATERIALCOLOR", color);
    }
    for (key, header) in [("roughness", "MATERIALROUGHNESS"), ("metallic", "MATERIALMETALLIC")] {
        if let Some(v) = p.get(key).and_then(Value::as_f64) {
            if !v.is_finite() || !(0.0..=1.0).contains(&v) { return Err(bad("material", &format!("{key} must be between 0 and 1"))); }
            s.doc_mut()?.header.set_f64(header, v);
        }
    }
    if !p.is_null() { s.touch(); }
    let d=s.doc()?;
    Ok(json!({
        "name": d.header.str("MATERIALCURRENT", "Default"),
        "color": d.header.str("MATERIALCOLOR", "#b8c0cc"),
        "roughness": d.header.f64("MATERIALROUGHNESS", 0.55),
        "metallic": d.header.f64("MATERIALMETALLIC", 0.0),
        "representation": "faceted-preview"
    }))
}

fn run_light(s: &mut Session, p: &Value) -> Result<Value> {
    if let Some(a)=p.get("direction").and_then(Value::as_array) {
        if a.len()!=3 { return Err(bad("light", "direction must be [x,y,z]")); }
        let vals: Vec<f64>=a.iter().map(|v| v.as_f64().unwrap_or(f64::NAN)).collect();
        if vals.iter().any(|v| !v.is_finite()) || vals.iter().all(|v| v.abs()<1e-12) { return Err(bad("light", "direction must be a non-zero finite vector")); }
        let len=(vals[0]*vals[0]+vals[1]*vals[1]+vals[2]*vals[2]).sqrt();
        s.doc_mut()?.header.set_f64("LIGHTDIRX", vals[0]/len);
        s.doc_mut()?.header.set_f64("LIGHTDIRY", vals[1]/len);
        s.doc_mut()?.header.set_f64("LIGHTDIRZ", vals[2]/len);
    }
    if let Some(v)=p.get("intensity").and_then(Value::as_f64) {
        if !v.is_finite() || !(0.0..=10.0).contains(&v) { return Err(bad("light", "intensity must be between 0 and 10")); }
        s.doc_mut()?.header.set_f64("LIGHTINTENSITY", v);
    }
    if !p.is_null() { s.touch(); }
    let d=s.doc()?;
    Ok(json!({"direction":[d.header.f64("LIGHTDIRX",0.4),d.header.f64("LIGHTDIRY",-0.6),d.header.f64("LIGHTDIRZ",0.7)],"intensity":d.header.f64("LIGHTINTENSITY",1.0),"type":"directional"}))
}

fn run_render(s: &mut Session, _p: &Value) -> Result<Value> {
    let d=s.doc()?;
    Ok(json!({
        "visualStyle": d.header.str("VISUALSTYLE","2DWireframe"),
        "material": d.header.str("MATERIALCURRENT","Default"),
        "materialColor": d.header.str("MATERIALCOLOR","#b8c0cc"),
        "lightDirection": [d.header.f64("LIGHTDIRX",0.4),d.header.f64("LIGHTDIRY",-0.6),d.header.f64("LIGHTDIRZ",0.7)],
        "lightIntensity": d.header.f64("LIGHTINTENSITY",1.0),
        "mode": "viewport-faceted-preview",
        "acis": false
    }))
}

fn normalize_visualstyle(value: &str) -> Option<&'static str> {
    let key = value.trim().to_ascii_lowercase().replace([' ', '_', '-'], "");
    match key.as_str() {
        "2dwireframe" | "2d" => Some("2DWireframe"),
        "wireframe" => Some("Wireframe"),
        "hidden" | "hiddenline" => Some("Hidden"),
        "conceptual" => Some("Conceptual"),
        "realistic" => Some("Realistic"),
        "shaded" => Some("Shaded"),
        "shadedwithedges" | "shadededges" => Some("ShadedWithEdges"),
        "sketchy" => Some("Sketchy"),
        "xray" => Some("X-ray"),
        _ => None,
    }
}

fn run_osnap(s: &mut Session, p: &Value) -> Result<Value> {
    use crate::snap::mode;
    if let Some(n) = p.get("osmode").and_then(Value::as_u64) {
        s.settings.osmode = (n as u32) & 0x7fff;
    }
    if let Some(ms) = p.get("modes").and_then(Value::as_array) {
        let mut bits = 0u32;
        for m in ms.iter().filter_map(Value::as_str) {
            let b = match m.to_ascii_lowercase().get(..3).unwrap_or("") {
                "end" => mode::END,
                "mid" => mode::MID,
                "cen" => mode::CEN,
                "nod" => mode::NOD,
                "qua" => mode::QUA,
                "int" => mode::INT,
                "ins" => mode::INS,
                "per" => mode::PER,
                "tan" => mode::TAN,
                "nea" => mode::NEA,
                "gce" => mode::GCEN,
                "app" => mode::APP,
                "ext" => mode::EXT,
                "par" => mode::PAR,
                _ => 0,
            };
            bits |= b;
        }
        s.settings.osmode = bits;
    }
    if p.get("osmode").is_none() && p.get("modes").is_none() {
        let on = p.get("on").and_then(Value::as_bool).unwrap_or(s.settings.osmode & mode::OFF != 0);
        if on {
            s.settings.osmode &= !mode::OFF;
        } else {
            s.settings.osmode |= mode::OFF;
        }
    }
    s.touch();
    let on = s.settings.osmode & mode::OFF == 0;
    let active: Vec<&str> = mode::ALL.iter().filter(|(b, _)| s.settings.osmode & b != 0).map(|(_, n)| *n).collect();
    Ok(json!({ "on": on, "osmode": s.settings.osmode, "modes": active, "message": format!("<Osnap {}>", if on { "on" } else { "off" }) }))
}

fn run_dsettings(s: &mut Session, p: &Value) -> Result<Value> {
    if let Some(v) = point_param(p, "snapunit").filter(|v| v.x > 0.0 && v.y > 0.0) {
        crate::sysvars::set(s, "SNAPUNIT", &json!([v.x, v.y]))?;
    }
    if let Some(v) = point_param(p, "gridunit").filter(|v| v.x >= 0.0 && v.y >= 0.0) {
        crate::sysvars::set(s, "GRIDUNIT", &json!([v.x, v.y]))?;
    }
    if let Some(a) = p.get("polarang").and_then(Value::as_f64).filter(|a| *a > 0.0) {
        s.settings.polarang = a.to_radians();
    }
    if let Some(m) = p.get("gridmajor").and_then(Value::as_u64) {
        s.settings.gridmajor = (m as u32).clamp(1, 100);
    }
    s.touch();
    Ok(serde_json::to_value(s.drafting_settings()).unwrap_or(Value::Null))
}

fn run_setvar(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_param(p, "name").ok_or_else(|| bad("setvar", "`name` is required"))?;
    let v = p.get("value").ok_or_else(|| bad("setvar", "`value` is required"))?;
    crate::sysvars::set(s, name, v)?;
    s.touch();
    Ok(json!({ "name": name.to_ascii_uppercase(), "value": crate::sysvars::get(s, name) }))
}

/// A system variable value as the command line shows it.
fn show_value(v: &Value) -> String {
    match v {
        Value::String(t) => t.clone(),
        Value::Array(a) => a.iter().map(show_value).collect::<Vec<_>>().join(","),
        other => other.to_string(),
    }
}

/// SETVAR at the command line: "Enter variable name or [?]", then "Enter new value for NAME <current>".
#[derive(Default)]
struct SetvarM {
    name: Option<String>,
}

impl Interactive for SetvarM {
    fn name(&self) -> &'static str {
        "SETVAR"
    }
    fn prompt(&self, s: &Session) -> Prompt {
        // Not a free-text prompt: Space ends the input, as it does for names and numbers.
        let accept = Accept { number: true, ..Accept::TEXT };
        match &self.name {
            None => Prompt::new("Enter variable name", accept).kw(&["?"]),
            Some(n) => {
                let p = Prompt::new(format!("Enter new value for {n}"), accept);
                match crate::sysvars::get(s, n) {
                    Some(v) => p.default(show_value(&v)),
                    None => p,
                }
            }
        }
    }
    fn input(&mut self, s: &mut Session, i: Input) -> Result<Step> {
        match (self.name.clone(), i) {
            (_, Input::Enter | Input::Cancel) => Ok(Step::Done),
            (None, Input::Keyword(_)) => {
                if let Value::Object(m) = crate::sysvars::list(s) {
                    for (k, v) in m {
                        s.echo(format!("{k:<12} {}", show_value(&v)));
                    }
                }
                Ok(Step::Done)
            }
            (None, Input::Text(t)) => {
                let n = t.trim().to_ascii_uppercase();
                match crate::sysvars::get(s, &n) {
                    None => {
                        s.echo(format!("Unknown variable {n}. Type SETVAR ? to list variables."));
                        Ok(Step::Done)
                    }
                    Some(v) if crate::sysvars::is_read_only(&n) => {
                        s.echo(format!("{n} = {} (read only)", show_value(&v)));
                        Ok(Step::Done)
                    }
                    Some(_) => {
                        self.name = Some(n);
                        Ok(Step::Continue)
                    }
                }
            }
            (Some(n), Input::Text(t)) => match crate::sysvars::set(s, &n, &Value::String(t)) {
                Ok(()) => {
                    s.touch();
                    Ok(Step::Done)
                }
                Err(e) => {
                    s.echo(e.to_string());
                    Ok(Step::Continue)
                }
            },
            _ => Ok(Step::Continue),
        }
    }
}

fn run_getvar(s: &mut Session, p: &Value) -> Result<Value> {
    let name = str_param(p, "name").ok_or_else(|| bad("getvar", "`name` is required"))?;
    let v = crate::sysvars::get(s, name).ok_or_else(|| bad("getvar", format!("unknown variable `{name}`")))?;
    Ok(json!({ "name": name.to_ascii_uppercase(), "value": v }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_style_round_trips_and_rejects_unknown_styles() {
        let mut s = Session::new();
        let changed = s.execute("visualstyle", &json!({"name": "shaded with edges"})).unwrap();
        assert_eq!(changed["visualStyle"], "ShadedWithEdges");
        assert_eq!(s.doc().unwrap().header.str("VISUALSTYLE", ""), "ShadedWithEdges");
        let queried = s.execute("vscurrent", &json!({})).unwrap();
        assert_eq!(queried["visualStyle"], "ShadedWithEdges");
        assert!(s.execute("visualstyle", &json!({"name": "toon"})).is_err());
    }

    #[test]
    fn backface_culling_persists_and_toggles() {
        let mut s = Session::new();
        let on = s.execute("backfacecull", &json!({"on": true})).unwrap();
        assert_eq!(on["on"], true);
        assert_eq!(s.doc().unwrap().header.i64("BACKFACECULL", 0), 1);
        let off = s.execute("bfcull", &json!({})).unwrap();
        assert_eq!(off["on"], false);
        assert_eq!(s.doc().unwrap().header.i64("BACKFACECULL", 1), 0);
    }

    #[test]
    fn material_and_light_persist_and_validate() {
        let mut s = Session::new();
        let mat=s.execute("material", &json!({"name":"Steel","color":"#8090a0","roughness":0.3,"metallic":0.8})).unwrap();
        assert_eq!(mat["name"], "Steel");
        assert_eq!(s.doc().unwrap().header.str("MATERIALCOLOR", ""), "#8090a0");
        let light=s.execute("light", &json!({"direction":[1.0,-2.0,3.0],"intensity":1.25})).unwrap();
        assert_eq!(light["type"], "directional");
        assert!((s.doc().unwrap().header.f64("LIGHTINTENSITY",0.0)-1.25).abs()<1e-12);
        assert!(s.execute("material", &json!({"color":"not-a-color"})).is_err());
        assert!(s.execute("light", &json!({"direction":[0.0,0.0,0.0]})).is_err());
    }
}
