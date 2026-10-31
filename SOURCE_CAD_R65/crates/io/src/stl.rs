//! Conservative STL facet import/export.  STL has no layers, names or attributes, so facets are
//! represented as `3DFACE` entities in model space and all non-facet entities are ignored on export.

use cadcraft_doc::{Common, Drawing, EntityKind, Face3d, Space};
use cadcraft_geom::Vec3;

use crate::{IoError, Result};

const MAX_TRIANGLES: usize = 1_000_000;

fn f32_at(bytes: &[u8], at: usize) -> f64 {
    f32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]) as f64
}

fn add_triangle(d: &mut Drawing, a: Vec3, b: Vec3, c: Vec3) -> Result<()> {
    if !a.is_finite() || !b.is_finite() || !c.is_finite() { return Err(IoError::Format("STL contains non-finite vertex".into())); }
    d.add(&Space::Model, Common::default(), EntityKind::Face3d(Face3d { corners: [a, b, c, c], hidden_edges: 0 }))
        .map(|_| ())
        .map_err(|e| IoError::Format(e.to_string()))
}

/// Import binary STL (preferred when the size is self-consistent) or ASCII STL.
pub fn read_stl(bytes: &[u8]) -> Result<Drawing> {
    let mut d = Drawing::new_metric();
    if bytes.len() >= 84 {
        let count = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
        if count <= MAX_TRIANGLES && 84usize.saturating_add(count.saturating_mul(50)) == bytes.len() {
            for i in 0..count {
                let base = 84 + i * 50 + 12;
                add_triangle(&mut d, Vec3::new(f32_at(bytes, base), f32_at(bytes, base + 4), f32_at(bytes, base + 8)), Vec3::new(f32_at(bytes, base + 12), f32_at(bytes, base + 16), f32_at(bytes, base + 20)), Vec3::new(f32_at(bytes, base + 24), f32_at(bytes, base + 28), f32_at(bytes, base + 32)))?;
            }
            return Ok(d);
        }
    }
    let text = std::str::from_utf8(bytes).map_err(|_| IoError::Format("STL is neither a valid binary STL nor UTF-8 ASCII STL".into()))?;
    let mut vertices = Vec::new();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        if !words.next().is_some_and(|w| w.eq_ignore_ascii_case("vertex")) { continue; }
        let x = words.next().and_then(|v| v.parse::<f64>().ok()).ok_or_else(|| IoError::Format("invalid ASCII STL vertex".into()))?;
        let y = words.next().and_then(|v| v.parse::<f64>().ok()).ok_or_else(|| IoError::Format("invalid ASCII STL vertex".into()))?;
        let z = words.next().and_then(|v| v.parse::<f64>().ok()).ok_or_else(|| IoError::Format("invalid ASCII STL vertex".into()))?;
        vertices.push(Vec3::new(x, y, z));
        if vertices.len() == 3 {
            add_triangle(&mut d, vertices[0], vertices[1], vertices[2])?;
            vertices.clear();
            if d.model.len() >= MAX_TRIANGLES { return Err(IoError::Format("STL exceeds the triangle limit".into())); }
        }
    }
    if vertices.is_empty() && !d.model.is_empty() { Ok(d) } else { Err(IoError::Format("ASCII STL has no complete facets".into())) }
}

fn put_f32(out: &mut Vec<u8>, v: f64) { out.extend_from_slice(&(v as f32).to_le_bytes()); }

/// Export model-space 3DFACE entities as binary STL triangles. Quads are split into two facets.
pub fn write_stl(d: &Drawing) -> Result<Vec<u8>> {
    let mut tris = Vec::new();
    for e in d.model.iter() {
        if let EntityKind::Face3d(f) = &e.kind {
            tris.push([f.corners[0], f.corners[1], f.corners[2]]);
            if f.corners[2] != f.corners[3] { tris.push([f.corners[0], f.corners[2], f.corners[3]]); }
        }
    }
    if tris.len() > MAX_TRIANGLES { return Err(IoError::Format("drawing exceeds the STL triangle limit".into())); }
    let mut out = vec![0u8; 80];
    let header = b"CADCraft STL export";
    out[..header.len()].copy_from_slice(header);
    out.extend_from_slice(&(tris.len() as u32).to_le_bytes());
    for [a, b, c] in tris {
        let n = (b - a).cross(c - a).normalized();
        for v in [n.x, n.y, n.z, a.x, a.y, a.z, b.x, b.y, b.z, c.x, c.y, c.z] { put_f32(&mut out, v); }
        out.extend_from_slice(&0u16.to_le_bytes());
    }
    Ok(out)
}

