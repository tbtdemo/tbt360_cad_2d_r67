# TBT360 CAD2027 — R48 XCLIP Rendering Hardening

Baseline: user-supplied R47 source (`d5a9630f...`). This batch deliberately addresses a bounded production gap rather than claiming a new CAD kernel.

## Code changes

- Fix XData parsing: R47 emitted `[1001,1070,1000]` but rendering previously searched only consecutive `[1001,1000]`, so XCLIP boundaries could silently be ignored.
- Reuse existing CADCraft `PolyClip` for arbitrary concave polygon clipping of XREF polylines, triangles/fills, points and infinite-line projections; work in world space after INSERT transform.
- Apply rectangular ancestor bounds and polygon intersection sequentially.
- Remove the full `TBT360_XCLIP` application XData section when editing/deleting instead of leaving orphaned 1070/1000 tags.
- Register previously unregistered XCLIP renderer tests in integration test suite and add a concave U-shaped boundary regression.
- Preserve 547/323 R47 command baseline: no commands/UI removed or added by this narrowly scoped patch.

## Verification boundary

Only Python static/source wiring audit and archive/checksum verification were run here. **Rust, Cargo, and a Windows build runner are not installed**. The new regression tests are authored and registered, but **not executed**. No R48 EXE or runtime PASS exists. The runtime bundled here is explicitly the unchanged v18 baseline, not R48. Full build, visual QA and lifecycle gate remain required.

## Remaining limitations

- Multiple nested polygon XCLIP scopes are not composited as exact polygon intersections; an inner polygon may replace a parent polygon (rectangular bounds still intersect). Nested rotated references need end-to-end verification.
- PolyClip accepts simple non-degenerate polygon boundaries; malformed/self-intersecting outlines are not certified for exact clipping.
- Generic B-Rep/NURBS/boolean, GPU hidden-line occlusion, disk-streamed octree, full PBR and proprietary cloud-format parsers are not implemented in this batch.
- R47 static audit numbers are **baseline claims**, not evidence of R48 compilation.

## Windows testing

Run `CHAY_BASELINE_V18.bat` only if you want to inspect the existing v18 app. To run R48 itself, build the included modified Rust source on a development machine and execute the registered tests (`cargo test -p cadcraft-render --test it`, `cargo test -p cadcraft-engine --test xref` subject to actual test-target discovery). Do not represent v18 as R48.
