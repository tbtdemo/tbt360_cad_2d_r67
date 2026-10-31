# TBT360 CAD2027 R47 — All-in-one hardening batch

R47 intentionally combines the requested workstreams in one source batch rather than splitting them into small versions.

## Implemented in this batch

1. Bounded rigid 6DoF point-cloud ICP with Horn quaternion fitting and optional persistent apply.
2. Persistent point-cloud octree leaf metadata foundation.
3. Associative 3DFACE edge dimensions and refresh.
4. Orthographic facet hidden-line/silhouette documentation output as native 2D LINE entities.
5. Polygon XCLIP metadata support; rectangle remains exact and polygon currently uses conservative render bounds.
6. Event-style post-command automatic associative path-array refresh under `ARRAYPATHAUTO` in the same undo transaction.
7. Faceted bilinear surface-patch generator.
8. Faceted closed-shell/solid audit with boundary/non-manifold edges and signed-volume estimate.
9. Persistent PBR material metadata for base color, metallic, roughness, opacity and texture/normal-map references.
10. Existing Dynamic Block authoring draft/validation and Annotation/Hatch/Table workflows surfaced directly in Tool Sets instead of being hidden.

## Technical truth boundary

R47 does not rename faceted geometry as ACIS/B-Rep or sampled patches as NURBS. It does not claim exact polygon XREF rendering, exact z-buffer hidden-line removal, disk-streamed octrees, native LAS/LAZ/E57/RCP/RCS, Autodesk evaluation-graph writing, Autodesk binary PC3/CTB/STB decode, or production ray tracing. Those remain explicit follow-on kernel/adapter tasks.

## Validation boundary

The current environment has no Rust/Cargo/Windows compiler, therefore R47 validation is source/static wiring only. No compile/runtime PASS is claimed for the new R47 code.
