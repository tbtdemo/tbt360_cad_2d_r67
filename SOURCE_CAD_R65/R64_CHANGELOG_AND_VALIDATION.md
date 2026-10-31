# TBT360 R64 — Grip hit-testing precision

Baseline: user-provided R63 source archive. The historical `R53_SOURCE` directory name in the archive is preserved for compatibility; it does **not** indicate that the changes are based on an older release.

## Actual implementation

- `crates/ui-egui/src/canvas/grips.rs`: `grip_at` now picks the nearest eligible grip in screen space rather than the first matching selection handle, keeping selection/grip order stable for exact ties.
- Rejects pointer positions outside viewport, invalid pointer coordinates, and invalid viewport scale. Ignores non-finite projected grips.
- Preserves visibility, layer locked, selection size, and existing grip rectangle semantics.
- `crates/ui-egui/src/canvas/tests.rs`: adds `grip_pick_prefers_nearest_and_rejects_outside_canvas` to test overlap, outside/NaN pointers, and invalid scale.

## Evidence and limitations

- Confirmed edited source markers and ZIP CRC in this environment.
- The Rust test was **written, not executed**; Rust/Cargo toolchain is not installed in this runtime.
- No Windows EXE built. No running `.bat` launcher or Windows UI claim.
- Intersection OSNAP, Grip Edit, Undo/Redo/Save/Reopen and other requested 3D/renderer/parser capabilities are not certified complete by this batch.

## Windows verification (developer machine)

From this source root with Rust 1.95 or newer:

`cargo test -p cadcraft-ui-egui grip_pick_prefers_nearest_and_rejects_outside_canvas`

Then run the actual desktop UI and confirm overlapping grip click, Shift-click, Move/Edit, Undo/Redo and Save/Reopen before production release.
