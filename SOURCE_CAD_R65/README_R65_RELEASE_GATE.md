# TBT360 R65 CAD native QTO — release gate

- Source base: R65, native Rust/egui; QTO is displayed inside CAD with `Đo bóc / Takeoff`.
- Added: finite coordinate validation for LINE/polyline bulge metrics, native unit regression; Windows Github Actions workflow.
- Run build on Windows GitHub Actions: `TBT360 R65 native QTO Windows portable` -> `workflow_dispatch`.
- Workflow must PASS both native QTO tests and UI grip regression before building EXE.
- Download Actions artifact `TBT360_R65_PORTABLE_WINDOWS_X64`, extract and double-click `CHAY_CAD_R65_PORTABLE.bat`.
- Artifact includes actual newly-built EXE **only on build success**.
- This source ZIP itself DOES NOT include a new native Windows EXE; it cannot run the newly-integrated QTO without successful build.
- Quantity results are in drawing units, no independent validated IFC/steel/PDF takeoff. Existing workspaces are separate references and are NOT represented as native functionality.
- Release-blockers: missing Rust toolchain on this environment; no authorized CAD repository connected; native UI screenshot and Save/Reopen cannot be verified here.
