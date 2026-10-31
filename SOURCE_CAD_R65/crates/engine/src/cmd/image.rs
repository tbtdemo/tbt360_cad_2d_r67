//! IMAGEATTACH / IMAGECLIP: metadata-first raster image attachment.
//!
//! The drawing model already preserves IMAGE/IMAGEDEF data through DXF. These commands create
//! and edit that metadata without pretending to decode or render raster pixels in the engine.
//! The renderer continues to show the image frame until a host-side bitmap renderer is wired in.

use cadcraft_doc::{EntityKind, Handle, Image};
use cadcraft_geom::{Vec2, Vec3};
use serde_json::{Value, json};

use super::{CommandSpec, bad, bool_or, f64_or, point_param, str_param, targets};
use crate::{Result, Session};

pub(crate) fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("imageattach", "Attach Raster Image", run_imageattach)
            .menu(&["Insert", "Attach Image..."])
            .alias(&["ima", "image"])
            .params("{path, at?: [x,y], width?: number, height?: number, pixels?: [w,h], rotation?: degrees, brightness?: 0..100, contrast?: 0..100, fade?: 0..100} → {handle, path, bounds}"),
        CommandSpec::new("imageclip", "Clip Image", run_imageclip)
            .menu(&["Modify", "Clip", "Image"])
            .alias(&["iclip"])
            .params("{handle?, points?: [[x,y],...] (image pixel coordinates), delete?: bool} → {handle, clipping, points}"),
        CommandSpec::new("imageadjust", "Adjust Raster Image", run_imageadjust)
            .menu(&["Modify", "Image", "Adjust"])
            .alias(&["iadjust"])
            .params("{handle?, brightness?:0..100, contrast?:0..100, fade?:0..100, transparency?:bool, visible?:bool}"),
        CommandSpec::new("imagetransform", "Transform Raster Image", run_imagetransform)
            .menu(&["Modify", "Image", "Transform"])
            .alias(&["itransform"])
            .params("{handle?, at?:[x,y], width?:positive, height?:positive, rotation?:degrees}"),
    ]
}

fn finite_positive(n: f64, key: &str) -> Result<f64> {
    if n.is_finite() && n > 0.0 { Ok(n) } else { Err(bad("imageattach", format!("`{key}` must be a positive number"))) }
}

fn pair(p: &Value, key: &str, default: Vec2) -> Result<Vec2> {
    let Some(a) = p.get(key) else { return Ok(default) };
    let Some(a) = a.as_array() else { return Err(bad("imageattach", format!("`{key}` must be [x,y]"))) };
    let x = a.first().and_then(Value::as_f64).ok_or_else(|| bad("imageattach", format!("`{key}[0]` must be a number")))?;
    let y = a.get(1).and_then(Value::as_f64).ok_or_else(|| bad("imageattach", format!("`{key}[1]` must be a number")))?;
    if !x.is_finite() || !y.is_finite() || x <= 0.0 || y <= 0.0 { return Err(bad("imageattach", format!("`{key}` must contain positive numbers"))) }
    Ok(Vec2::new(x, y))
}

fn image_json(h: Handle, i: &Image) -> Value {
    let a = i.insert.xy();
    let b = a + i.u.xy() * i.size.x;
    let c = b + i.v.xy() * i.size.y;
    let d = a + i.v.xy() * i.size.y;
    json!({
        "handle": h.hex(), "path": i.path, "name": i.name, "insert": [a.x, a.y],
        "corners": [[a.x,a.y],[b.x,b.y],[c.x,c.y],[d.x,d.y]],
        "pixels": [i.size.x, i.size.y], "clip": i.clip.iter().map(|p| json!([p.x,p.y])).collect::<Vec<_>>(),
        "clipping": i.clipping, "brightness": i.brightness, "contrast": i.contrast, "fade": i.fade,
    })
}

fn run_imageattach(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_param(p, "path").map(str::trim).filter(|t| !t.is_empty()).ok_or_else(|| bad("imageattach", "`path` is required"))?.to_string();
    let at = point_param(p, "at").unwrap_or(Vec2::ZERO);
    let pixels = pair(p, "pixels", Vec2::new(1.0, 1.0))?;
    let width = finite_positive(f64_or(p, "width", pixels.x), "width")?;
    let height = finite_positive(f64_or(p, "height", pixels.y), "height")?;
    let angle = f64_or(p, "rotation", 0.0).to_radians();
    if !angle.is_finite() { return Err(bad("imageattach", "`rotation` must be a number")) }
    let (sn, cs) = angle.sin_cos();
    let u = Vec3::new(cs * width / pixels.x, sn * width / pixels.x, 0.0);
    let v = Vec3::new(-sn * height / pixels.y, cs * height / pixels.y, 0.0);
    let mut i = Image { insert: at.to3(0.0), u, v, size: pixels, path: path.clone(), ..Image::default() };
    for (key, dst) in [("brightness", &mut i.brightness), ("contrast", &mut i.contrast), ("fade", &mut i.fade)] {
        if let Some(n) = p.get(key).and_then(Value::as_u64) {
            *dst = u8::try_from(n).map_err(|_| bad("imageattach", format!("`{key}` must be 0..100")))?;
            if *dst > 100 { return Err(bad("imageattach", format!("`{key}` must be 0..100"))) }
        }
    }
    let h = s.add_entity(EntityKind::Image(i.clone()))?;
    Ok(image_json(h, &i))
}

fn image_handle(s: &Session, p: &Value) -> Result<Handle> {
    if let Some(h) = super::curves::handle_param(p, "handle") { return Ok(h) }
    let hs = targets(s, p)?;
    hs.first().copied().ok_or_else(|| bad("imageclip", "an image handle or selected image is required"))
}

fn run_imageclip(s: &mut Session, p: &Value) -> Result<Value> {
    let h = image_handle(s, p)?;
    let old = s.doc()?.entity(h).ok_or_else(|| bad("imageclip", "image not found"))?;
    if !matches!(&old.kind, EntityKind::Image(_)) { return Err(bad("imageclip", "selected object is not an image")) }
    let delete = bool_or(p, "delete", false);
    let points = if delete { Vec::new() } else {
        let a = p.get("points").and_then(Value::as_array).ok_or_else(|| bad("imageclip", "`points` is required unless `delete` is true"))?;
        let mut out = Vec::with_capacity(a.len());
        for q in a {
            let x = q.get(0).and_then(Value::as_f64).ok_or_else(|| bad("imageclip", "clip points must be [x,y]"))?;
            let y = q.get(1).and_then(Value::as_f64).ok_or_else(|| bad("imageclip", "clip points must be [x,y]"))?;
            if !x.is_finite() || !y.is_finite() { return Err(bad("imageclip", "clip points must be finite")) }
            out.push(Vec2::new(x, y));
        }
        if out.len() < 2 { return Err(bad("imageclip", "at least two clip points are required")) }
        out
    };
    s.doc_mut()?.modify_entity(h, |e| {
        if let EntityKind::Image(i) = &mut e.kind {
            i.clip = points.clone();
            i.clipping = !delete;
            if delete { i.display &= !4; } else { i.display |= 4; }
        }
    })?;
    let i = match &s.doc()?.entity(h).ok_or_else(|| bad("imageclip", "image not found after update"))?.kind {
        EntityKind::Image(i) => i.clone(),
        _ => return Err(bad("imageclip", "image not found after update")),
    };
    Ok(image_json(h, &i))
}



fn run_imageadjust(s: &mut Session, p: &Value) -> Result<Value> {
    let h = image_handle(s, p)?;
    let old = s.doc()?.entity(h).ok_or_else(|| bad("imageadjust", "image not found"))?;
    let EntityKind::Image(old_i) = &old.kind else { return Err(bad("imageadjust", "selected object is not an image")); };
    let mut next = old_i.clone();
    for (key, dst) in [("brightness", &mut next.brightness), ("contrast", &mut next.contrast), ("fade", &mut next.fade)] {
        if let Some(n) = p.get(key).and_then(Value::as_u64) {
            if n > 100 { return Err(bad("imageadjust", format!("`{key}` must be 0..100"))); }
            *dst = n as u8;
        }
    }
    if let Some(on) = p.get("transparency").and_then(Value::as_bool) {
        if on { next.display |= 8; } else { next.display &= !8; }
    }
    let visible = p.get("visible").and_then(Value::as_bool);
    s.doc_mut()?.modify_entity(h, |e| {
        e.kind = EntityKind::Image(next.clone());
        if let Some(v) = visible { e.common.visible = v; }
    })?;
    s.touch();
    let mut out = image_json(h, &next);
    out["transparency"] = json!((next.display & 8) != 0);
    out["visible"] = json!(s.doc()?.entity(h).is_some_and(|e| e.common.visible));
    Ok(out)
}

fn run_imagetransform(s: &mut Session, p: &Value) -> Result<Value> {
    let h = image_handle(s, p)?;
    let old = s.doc()?.entity(h).ok_or_else(|| bad("imagetransform", "image not found"))?;
    let EntityKind::Image(old_i) = &old.kind else { return Err(bad("imagetransform", "selected object is not an image")); };
    let mut next = old_i.clone();
    let old_width = old_i.u.xy().len() * old_i.size.x;
    let old_height = old_i.v.xy().len() * old_i.size.y;
    let width = p.get("width").and_then(Value::as_f64).unwrap_or(old_width);
    let height = p.get("height").and_then(Value::as_f64).unwrap_or(old_height);
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(bad("imagetransform", "width and height must be positive finite numbers"));
    }
    let current_angle = old_i.u.y.atan2(old_i.u.x).to_degrees();
    let angle = p.get("rotation").and_then(Value::as_f64).unwrap_or(current_angle);
    if !angle.is_finite() { return Err(bad("imagetransform", "rotation must be finite degrees")); }
    if let Some(at) = p.get("at") {
        let a = at.as_array().ok_or_else(|| bad("imagetransform", "`at` must be [x,y]"))?;
        if a.len() != 2 { return Err(bad("imagetransform", "`at` must be [x,y]")); }
        let x = a[0].as_f64().unwrap_or(f64::NAN);
        let y = a[1].as_f64().unwrap_or(f64::NAN);
        if !x.is_finite() || !y.is_finite() { return Err(bad("imagetransform", "`at` coordinates must be finite")); }
        next.insert.x = x;
        next.insert.y = y;
    }
    let (sn, cs) = angle.to_radians().sin_cos();
    next.u = Vec3::new(cs * width / next.size.x, sn * width / next.size.x, 0.0);
    next.v = Vec3::new(-sn * height / next.size.y, cs * height / next.size.y, 0.0);
    s.doc_mut()?.modify_entity(h, |e| e.kind = EntityKind::Image(next.clone()))?;
    s.touch();
    let mut out = image_json(h, &next);
    out["width"] = json!(width);
    out["height"] = json!(height);
    out["rotation"] = json!(angle);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imageattach_and_imageclip_keep_dxf_metadata() {
        let mut s = Session::new();
        let r = s
            .execute("imageattach", &json!({"path": "survey/site.png", "at": [10.0, 20.0], "width": 100.0, "height": 50.0, "pixels": [2000.0, 1000.0], "rotation": 15.0}))
            .unwrap();
        let h = r["handle"].as_str().expect("image handle").to_string();
        assert_eq!(r["path"], "survey/site.png");
        assert_eq!(r["pixels"], json!([2000.0, 1000.0]));
        let c = s.execute("imageclip", &json!({"handle": h, "points": [[0.0, 0.0], [2000.0, 0.0], [2000.0, 1000.0], [0.0, 1000.0]]})).unwrap();
        assert_eq!(c["clipping"], true);
        let d = s.execute("imageclip", &json!({"handle": c["handle"], "delete": true})).unwrap();
        assert_eq!(d["clipping"], false);
        let a = s.execute("imageadjust", &json!({"handle": h, "brightness": 70, "contrast": 60, "fade": 15, "transparency": true})).unwrap();
        assert_eq!(a["brightness"], 70);
        assert_eq!(a["transparency"], true);
        let t = s.execute("imagetransform", &json!({"handle": a["handle"], "at":[5.0,6.0], "width":25.0, "height":10.0, "rotation":30.0})).unwrap();
        assert_eq!(t["width"], 25.0);
        assert_eq!(t["height"], 10.0);
    }
}
