# TBT360 CAD2027 R49 — Nested XCLIP Intersection Hardening

## Baseline
Based strictly on supplied R48 source, which itself was derived from R47. This patch changes the existing renderer implementation rather than introducing a parallel rendering system.

## Source changes
- Propagate `poly_clips: Vec<Arc<PolyClip>>` through INSERT expansion instead of replacing the parent clipping polygon.
- Apply each inherited polygon sequentially to polylines, filled triangles, and infinite line/ray segments.
- Reject point primitives lying outside *any* inherited polygon.
- Preserve rectangular bounding intersection and clipping order; each polygon is expressed in world coordinates after INSERT transform.
- Add `nested_polygon_xclip_intersects_parent_and_child_boundaries` integration regression in `crates/render/tests/it/xclip.rs`.
- Preserve all existing UI commands and baseline source; no new command declarations or aliases are added.

## Verification boundary
Static code-structure checks were performed. **Rust/Cargo are not installed** in this environment; compilation, Rust tests, full lifecycle regression, Windows graphics QA, and executable build were **not run**. The included Windows EXE is the unchanged v18 baseline from the original test-now archive, not an R49 binary.

## Limitations
- Polygon clipping depends on the existing PolyClip triangulation and robustness rules; malformed/self-intersecting polygons are not certified.
- No integration runtime evidence for rotated/scaled nested inserts, multi-copy arrays, font glyph clipping, or perspective 3D clipping.
- Exact z-buffer occlusion, B-Rep/NURBS/booleans, disk streaming and full PBR remain outside this batch.

## Required validation on Windows with Rust toolchain
From `R49_SOURCE`, run `cargo test -p cadcraft-render --test it` after checking target/package names against Cargo.toml; compile/run the actual UI and visually verify nested XREF scenes. Failures are release blockers for a new binary.
