# TBT360 R65 — Grip selection-limit correctness

## Source change

- `crates/ui-egui/src/canvas/grips.rs`: removed arbitrary `.take(200)` in the *real* `grip_at` path. The existing `Session::shows_grips` remains the authoritative `GRIPS`/`GRIPOBJLIMIT` gate.
- `crates/ui-egui/src/canvas/tests.rs`: new regression `grip_pick_respects_configured_unlimited_selection`, with 220 selected LINE entities and a target grip on the 220th.
- Added `BUILD_AND_PACKAGE_WINDOWS_R65.bat` (developer machine, needs Rust 1.95+) and `CHAY_TBT360_R65.bat` (requires *new* `TBT360.exe`). A BAT alone does not make the source runnable.

## Verification limits

- Changes were checked structurally; ZIP integrity and SHA256 can be verified on the delivered archive.
- Rust/Cargo and Windows desktop UI were not available in the current environment: **no Rust tests run, no new Windows EXE, no functional Windows PASS**.
- No change to tabs/buttons/layout. Existing mouse/Grip engine preserved.
- Still open: direct end-to-end OSNAP, Grip Edit/Move, Undo/Redo, Save/Reopen, camera picking, B-Rep, NURBS, rendering, XREF, point-cloud and SDK dependent work.
