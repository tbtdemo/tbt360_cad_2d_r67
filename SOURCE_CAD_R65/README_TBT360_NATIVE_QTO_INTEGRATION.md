# TBT360 CAD R65 — Native QTO integration source

- Baseline: latest CAD2027 R65 source in currently available CAD2027 series; not R46 or V18.
- `apps/cadcraft/src/main.rs`: native button **Đo bóc / Takeoff**.
- `apps/cadcraft/src/takeoff.rs`: inside the SAME egui CAD window; reads canonical drawing `session.doc().model`, shows entity count, summed LINE 3D length, summed CIRCLE area by layer and copies CSV.
- Unsupported: PDF multi-page, original web QTO workspace, IFC, BIM BREP volume, save/reopen quantities independent of CAD, unit conversion, native compiled screenshot. These are **not** claimed delivered.
- No Rust/Cargo toolchain on this execution host: no compile or Windows binary was created.
- Existing upstream CAD functionality and UI source are preserved; added a small button above existing canvas, not redesigned whole interface.
- Before Windows release, run `cargo test -p cadcraft --locked` and `cargo build --release -p cadcraft --locked` in a Windows x64 runner with Rust >= 1.95, then inspect CAD, press Takeoff, create LINE/CIRCLE, check quantities, Undo/Redo, Save/Reopen, take screenshots, and package real EXE.
- EXE 0.14.0 is NOT included or relabeled as R65, because it does not contain this code.
