//! PDFATTACH / PDFCLIP metadata support.
//!
//! The engine does not yet decode PDF page raster/vector contents. We therefore keep a
//! PDFUNDERLAY record as an `Unknown` entity with stable placement and clip metadata. R32 renders
//! that placement/clip frame in the viewport while the DXF writer preserves the record; a future
//! page-content renderer can consume the same metadata without changing the command API.

use cadcraft_doc::{EntityKind, Handle, RawTag, Unknown};
use cadcraft_geom::Vec2;
use serde_json::{Value, json};

use super::{CommandSpec, bad, bool_or, point_param, str_param, targets};
use crate::{Result, Session};

pub(crate) fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new("pdfattach", "Attach PDF Underlay", run_pdfattach)
            .menu(&["Insert", "PDF Underlay..."])
            .alias(&["pdfa", "attachpdf"])
            .params("{path, page?: number (default 1), at?: [x,y], width?: number, height?: number, rotation?: degrees} → {handle, path, page, bounds, rendered: false}"),
        CommandSpec::new("pdfclip", "Clip PDF Underlay", run_pdfclip)
            .menu(&["Modify", "Clip", "PDF Underlay"])
            .alias(&["pdfc"])
            .params("{handle?, points?: [[x,y],...], delete?: bool} → {handle, clipping, points, rendered: false}"),
        CommandSpec::new("underlayclip", "Clip Vector Underlay", run_underlayclip)
            .menu(&["Modify", "Clip", "Vector Underlay"])
            .alias(&["dwfclip", "dgnclip", "wmfclip", "epsclip", "uclip"])
            .params("{handle?, points?: [[x,y],...], delete?: bool} → {handle, type, clipping, points, rendered: false}"),
        CommandSpec::new("underlayinfo", "Underlay Information", run_underlayinfo)
            .menu(&["Inquiry", "Underlay Information"])
            .alias(&["uinfo"])
            .params("{handle?} → stable path/page/placement/clip metadata; rendered is false until a host renderer is connected")
            .noundo(),
    ]
}

fn number(p: &Value, key: &str, default: f64) -> Result<f64> {
    let n = p.get(key).and_then(Value::as_f64).unwrap_or(default);
    if n.is_finite() && n > 0.0 { Ok(n) } else { Err(bad("pdfattach", format!("`{key}` must be a positive number"))) }
}

fn tags_for(p: &Value, path: &str, subclass: &str) -> Result<Vec<RawTag>> {
    let at = point_param(p, "at").unwrap_or(Vec2::ZERO);
    let width = number(p, "width", 1.0)?;
    let height = number(p, "height", 1.0)?;
    let page = p.get("page").and_then(Value::as_u64).unwrap_or(1).clamp(1, 100_000);
    let angle = p.get("rotation").and_then(Value::as_f64).unwrap_or(0.0).to_radians();
    if !angle.is_finite() { return Err(bad("pdfattach", "`rotation` must be a number")) }
    let (sn, cs) = angle.sin_cos();
    let u = Vec2::new(cs * width, sn * width);
    let v = Vec2::new(-sn * height, cs * height);
    Ok(vec![
        RawTag { code: 100, value: subclass.into() },
        RawTag { code: 1, value: path.into() },
        RawTag { code: 90, value: page.to_string() },
        RawTag { code: 10, value: at.x.to_string() },
        RawTag { code: 20, value: at.y.to_string() },
        RawTag { code: 30, value: "0".into() },
        RawTag { code: 11, value: u.x.to_string() },
        RawTag { code: 21, value: u.y.to_string() },
        RawTag { code: 12, value: v.x.to_string() },
        RawTag { code: 22, value: v.y.to_string() },
        RawTag { code: 13, value: width.to_string() },
        RawTag { code: 23, value: height.to_string() },
        RawTag { code: 280, value: "0".into() },
    ])
}

fn summary(h: Handle, u: &Unknown) -> Value {
    let path = u.tags.iter().find(|t| t.code == 1).map(|t| t.value.clone()).unwrap_or_default();
    let page = u.tags.iter().find(|t| t.code == 90).and_then(|t| t.value.parse::<u64>().ok()).unwrap_or(1);
    let clipping = u.tags.iter().find(|t| t.code == 280).is_some_and(|t| t.value != "0");
    // X/Y values are parallel DXF tags. Do not match by value: repeated X coordinates
    // are normal in rectangular clips and matching by text would duplicate the first Y.
    let xs = u.tags.iter().filter(|t| t.code == 14).filter_map(|t| t.value.parse::<f64>().ok());
    let ys = u.tags.iter().filter(|t| t.code == 24).filter_map(|t| t.value.parse::<f64>().ok());
    let points: Vec<Value> = xs.zip(ys).map(|(x, y)| json!([x, y])).collect();
    json!({"handle": h.hex(), "type": u.dxf_type.clone(), "path": path, "page": page, "clipping": clipping, "points": points, "rendered": false, "metadataOnly": true})
}

fn run_pdfattach(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_param(p, "path").map(str::trim).filter(|x| !x.is_empty()).ok_or_else(|| bad("pdfattach", "`path` is required"))?;
    let u = Unknown { dxf_type: "PDFUNDERLAY".into(), tags: tags_for(p, path, "AcDbPdfUnderlay")? };
    let h = s.add_entity(EntityKind::Unknown(u.clone()))?;
    Ok(summary(h, &u))
}

/// ATTACH dispatch for raster and vector underlay metadata. Files are not decoded here; the
/// command records their placement so a renderer can be added without changing the command API.
pub(crate) fn run_attach(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_param(p, "path").map(str::trim).filter(|x| !x.is_empty()).ok_or_else(|| bad("attach", "`path` is required"))?;
    let ext = std::path::Path::new(path).extension().and_then(|x| x.to_str()).unwrap_or_default().to_ascii_lowercase();
    if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "tif" | "tiff") {
        return s.execute("imageattach", p);
    }
    if ext == "pdf" {
        return s.execute("pdfattach", p);
    }
    let (kind, subclass) = match ext.as_str() {
        "dwf" | "dwfx" => ("DWFUNDERLAY", "AcDbDwfUnderlay"),
        "dgn" => ("DGNUNDERLAY", "AcDbDgnUnderlay"),
        "wmf" => ("WMFUNDERLAY", "AcDbWmfUnderlay"),
        "eps" => ("EPSUNDERLAY", "AcDbEpsUnderlay"),
        _ => return Err(bad("attach", "supported formats are PDF, PNG, JPG, BMP, TIF, DWF, DGN, WMF and EPS")),
    };
    let u = Unknown { dxf_type: kind.into(), tags: tags_for(p, path, subclass)? };
    let h = s.add_entity(EntityKind::Unknown(u.clone()))?;
    Ok(json!({"handle": h.hex(), "path": path, "format": ext, "rendered": false, "metadataOnly": true}))
}

fn handle(s: &Session, p: &Value) -> Result<Handle> {
    if let Some(h) = super::curves::handle_param(p, "handle") { return Ok(h) }
    targets(s, p)?.first().copied().ok_or_else(|| bad("pdfclip", "a PDF underlay handle or selection is required"))
}

fn run_pdfclip(s: &mut Session, p: &Value) -> Result<Value> {
    let h = handle(s, p)?;
    let e = s.doc()?.entity(h).ok_or_else(|| bad("pdfclip", "underlay not found"))?;
    if !matches!(&e.kind, EntityKind::Unknown(u) if u.dxf_type.eq_ignore_ascii_case("PDFUNDERLAY")) { return Err(bad("pdfclip", "selected object is not a PDF underlay")) }
    let delete = bool_or(p, "delete", false);
    let points = if delete { Vec::new() } else { clip_points(p, "pdfclip")? };
    update_clip(s, h, &points, delete, "pdfclip")?;
    let u = match &s.doc()?.entity(h).ok_or_else(|| bad("pdfclip", "underlay not found after update"))?.kind {
        EntityKind::Unknown(u) => u.clone(),
        _ => return Err(bad("pdfclip", "underlay not found after update")),
    };
    Ok(summary(h, &u))
}

fn run_underlayclip(s: &mut Session, p: &Value) -> Result<Value> {
    let h = handle(s, p)?;
    let e = s.doc()?.entity(h).ok_or_else(|| bad("underlayclip", "underlay not found"))?;
    let supported = matches!(&e.kind, EntityKind::Unknown(u) if matches!(u.dxf_type.to_ascii_uppercase().as_str(), "DWFUNDERLAY" | "DGNUNDERLAY" | "WMFUNDERLAY" | "EPSUNDERLAY"));
    if !supported { return Err(bad("underlayclip", "selected object is not a DWF, DGN, WMF or EPS underlay; use PDFCLIP for PDF or IMAGECLIP for raster images")) }
    let delete = bool_or(p, "delete", false);
    let points = if delete { Vec::new() } else { clip_points(p, "underlayclip")? };
    update_clip(s, h, &points, delete, "underlayclip")?;
    let u = match &s.doc()?.entity(h).ok_or_else(|| bad("underlayclip", "underlay not found after update"))?.kind {
        EntityKind::Unknown(u) => u.clone(),
        _ => return Err(bad("underlayclip", "underlay not found after update")),
    };
    Ok(summary(h, &u))
}

fn clip_points(p: &Value, cmd: &str) -> Result<Vec<Vec2>> {
    let a = p.get("points").and_then(Value::as_array).ok_or_else(|| bad(cmd, "`points` is required unless `delete` is true"))?;
    let mut out = Vec::with_capacity(a.len());
    for q in a {
        let q = q.as_array().ok_or_else(|| bad(cmd, "clip points must be [x,y]"))?;
        let x = q.first().and_then(Value::as_f64).ok_or_else(|| bad(cmd, "clip points must be [x,y]"))?;
        let y = q.get(1).and_then(Value::as_f64).ok_or_else(|| bad(cmd, "clip points must be [x,y]"))?;
        if !x.is_finite() || !y.is_finite() { return Err(bad(cmd, "clip points must be finite")) }
        out.push(Vec2::new(x, y));
    }
    if out.len() < 3 { return Err(bad(cmd, "at least three clip points are required for a polygon")) }
    Ok(out)
}

fn update_clip(s: &mut Session, h: Handle, points: &[Vec2], delete: bool, cmd: &str) -> Result<()> {
    s.doc_mut()?.modify_entity(h, |e| {
        if let EntityKind::Unknown(u) = &mut e.kind {
            u.tags.retain(|t| t.code != 14 && t.code != 24 && t.code != 280);
            u.tags.push(RawTag { code: 280, value: (!delete).to_string() });
            for q in points {
                u.tags.push(RawTag { code: 14, value: q.x.to_string() });
                u.tags.push(RawTag { code: 24, value: q.y.to_string() });
            }
        }
    }).map_err(|e| bad(cmd, e.to_string()))
}

fn run_underlayinfo(s: &mut Session, p: &Value) -> Result<Value> {
    let h = handle(s, p)?;
    let e = s.doc()?.entity(h).ok_or_else(|| bad("underlayinfo", "underlay not found"))?;
    match &e.kind {
        EntityKind::Unknown(u) if u.dxf_type.to_ascii_uppercase().ends_with("UNDERLAY") => Ok(summary(h, u)),
        _ => Err(bad("underlayinfo", "selected object is not a supported vector or PDF underlay")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdfattach_and_clip_keep_metadata() {
        let mut s = Session::new();
        let r = s.execute("pdfattach", &json!({"path":"plan.pdf", "page":2, "width":100.0, "height":70.0})).unwrap();
        let h = r["handle"].as_str().unwrap().to_string();
        assert_eq!(r["page"], 2);
        let c = s.execute("pdfclip", &json!({"handle":h, "points":[[0,0],[100,0],[100,70],[0,70]]})).unwrap();
        assert_eq!(c["clipping"], true);
        assert_eq!(c["points"], json!([[0.0,0.0],[100.0,0.0],[100.0,70.0],[0.0,70.0]]));
        assert_eq!(c["rendered"], false);
    }

    #[test]
    fn attach_dispatches_supported_exchange_underlays_without_decoding_them() {
        let mut s = Session::new();
        for (path, format) in [("survey.dwf", "dwf"), ("map.dgn", "dgn"), ("detail.wmf", "wmf"), ("stamp.eps", "eps")] {
            let r = s.execute("attach", &json!({"path": path, "width": 10.0, "height": 5.0})).unwrap();
            assert_eq!(r["format"], format);
            assert_eq!(r["metadataOnly"], true);
            let c = s.execute("underlayclip", &json!({"handle": r["handle"], "points":[[0,0],[10,0],[10,5],[0,5]]})).unwrap();
            assert_eq!(c["type"].as_str().unwrap().ends_with("UNDERLAY"), true);
            assert_eq!(c["points"], json!([[0.0,0.0],[10.0,0.0],[10.0,5.0],[0.0,5.0]]));
            let info = s.execute("uinfo", &json!({"handle": r["handle"]})).unwrap();
            assert_eq!(info["path"], path);
        }
    }

    #[test]
    fn underlay_clip_rejects_non_polygon_and_preserves_repeated_x_vertices() {
        let mut s = Session::new();
        let r = s.execute("attach", &json!({"path":"map.dgn"})).unwrap();
        let h = r["handle"].clone();
        assert!(s.execute("dgnclip", &json!({"handle":h, "points":[[0,0],[1,1]]})).is_err());
        let c = s.execute("dgnclip", &json!({"handle":h, "points":[[0,0],[10,0],[10,5],[0,5]]})).unwrap();
        assert_eq!(c["points"], json!([[0.0,0.0],[10.0,0.0],[10.0,5.0],[0.0,5.0]]));
        let d = s.execute("dgnclip", &json!({"handle":h, "delete":true})).unwrap();
        assert_eq!(d["clipping"], false);
        assert_eq!(d["points"], json!([]));
    }
}
