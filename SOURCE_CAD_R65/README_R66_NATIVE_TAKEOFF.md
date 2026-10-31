# TBT360 CAD R65 — Native Takeoff deepening (source patch)

This release retains the R65 CAD source and extends `apps/cadcraft/src/takeoff.rs` inside the original Rust/egui CAD application. **No external QTO program is launched.** The native button remains `Đo bóc / Takeoff`.

## Implemented in source
- Read the current canonical drawing model, grouped by layer.
- LINE 3D lengths; ARC lengths; 2D lightweight polylines (including bulge circular-arc segments); 3D polyline lengths; closed 2D polyline areas; CIRCLE areas.
- Expose unsupported/unmeasured entity count per layer (no false zero-valued QTO claim).
- CSV copy with quoted/escaped layer names and columns for each measurable type.
- 4 native unit tests for an open segment, rectangle area/perimeter, arc bulge, and invalid coordinates.

## Blocked before production
- Rust/Cargo unavailable in authoring environment: no compiled EXE and no runtime screenshot; new unit tests have **not run**.
- Drawing units are not automatically mapped to m/m². No BIM entity classification, material weights, IFC takeoff or production rebar schedule yet.
- Existing CAD native Undo/Redo and Save/Reopen rely on the canonical CAD model; their integration with the new QTO display must be tested after compiling.
- No new Windows executable was built. The 0.14.0 portable CAD binary is not included to misrepresent it as this new source.

## Windows verification
1. Install Rust 1.95+ / MSVC Build Tools on a development runner (not on the end-user machine).
2. `cargo test -p cadcraft --locked takeoff::tests -- --nocapture`
3. `cargo build --release -p cadcraft --locked`
4. Open CAD; draw a 10×8 closed polyline; open the native Takeoff window; expect perimeter 36 drawing units and area 80 drawing units squared. Change a vertex and verify quantities update; Undo/Redo, save DXF, reopen and verify again. Capture a screenshot of the native EXE.
5. Package the **newly built** `cadcraft.exe` with BAT launcher and required runtime; do not reuse prior binaries.
