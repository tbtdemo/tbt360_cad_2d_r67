# TBT360 CAD2027 R47 — All-in-one hardening status

This matrix separates implemented capability from foundations and blockers. Static/source evidence only; no Windows compile/runtime claim is made in this environment.

| Workstream | R47 status | Evidence / boundary |
|---|---|---|
| Rigid 6DoF ICP | IMPLEMENTED (bounded) | `POINTCLOUD.ICPRIGID`: iterative nearest-neighbour pairing + Horn quaternion rigid fit, optional apply to native POINT entities. O(N×M), bounded sample; not yet production-scale indexed ICP. |
| Persistent octree / streaming | PARTIAL+ | `POINTCLOUD.OCTREEBUILD` persists leaf-code XData and document metadata; existing adaptive octree stats retained. Disk-page streaming/LRU/mmap and incremental external-file paging are not implemented. |
| 3D face/edge associativity | IMPLEMENTED (edge-index scope) | `DIM3D.FACEASSOC` + `DIM3D.FACEREFRESH`; native LINE/TEXT annotation follows `3DFACE handle + edge index`. Topology-changing remesh can invalidate edge index. |
| Hidden-line / documentation | PARTIAL+ | `HIDDENLINE3D/HLR3D` creates native 2D LINE boundary/silhouette projection from selected facets. It is orthographic and view-aware but does not perform exact z-buffer occlusion. |
| XREF polygon clipping | PARTIAL+ | XCLIP accepts 3+ points and preserves exact polygon coordinates in `TBT360_XCLIP` XData. Renderer currently clips conservatively to polygon bounds; exact point-in-polygon clipping is still open. |
| Dynamic Block authoring | PARTIAL | Existing `BLOCKS.DYNAMIC.AUTHOR` normalized authoring draft + validation/reference/grip edit retained and surfaced. Autodesk evaluation-graph object writing remains intentionally unsupported, so full DWG Dynamic Block authoring is not claimed. |
| Annotation workflow | PARTIAL+ | Existing MTEDIT/DDEDIT, FIELD/UPDATEFIELD, DIMSTYLE and annotation commands retained; surfaced together in Tool Sets. Full AutoCAD FIELD grammar, full annotative per-scale positions and advanced MLeader parity remain open. |
| Hatch / Table workflow | PARTIAL+ | Existing HATCH/HATCHEDIT, TABLE formula/recalc/edit/export and Data Link surfaced together. Full hatch pattern/MPolygon UI, Excel write-back and Autodesk FIELD-equivalent table formulas remain open. |
| Array auto dependency | IMPLEMENTED (path arrays) | `ARRAYPATHAUTO` hooks the command execution pipeline: after a successful undoable drawing command, associative path arrays auto-resample in the same undo transaction. Failures are isolated and echoed. |
| Solid kernel foundation | PARTIAL foundation | `SOLIDAUDIT3D` checks faceted shell manifold/boundary topology and signed-volume estimate. This is not B-Rep/ACIS and does not claim general 3D boolean topology. |
| Surface kernel foundation | PARTIAL foundation | `SURFACEPATCH` creates a bilinear sampled native 3DFACE grid. It is not NURBS and has no exact trim/continuity kernel. |
| PBR pipeline foundation | PARTIAL foundation | `PBRMATERIAL` persists baseColor/metallic/roughness/opacity/texture/normal-map metadata as `TBT360_PBR`. Current renderer does not yet consume all PBR channels. |
| LAS/LAZ/E57/RCP/RCS | BLOCKED / NOT IMPLEMENTED | No fake support added. Requires real parser/SDK/license-compatible adapter and binary/runtime validation. |
| Production ray tracing | NOT IMPLEMENTED | Existing render preview remains raster/faceted. No ray/path-traced production renderer is claimed. |
| Exact ACIS/NURBS / solid Boolean | BLOCKED / NOT IMPLEMENTED | Requires a real commercial/open kernel integration and extensive topology/regression tests. Faceted foundations above do not substitute for it. |

## User-visible R47 additions

- Modeling Tool Sets: rigid ICP, persistent octree, hidden-line projection, face-edge associative dimensions, solid/shell audit, faceted surface patch, PBR metadata.
- Drafting Tool Sets: Dynamic Block authoring/validation, XREF Clip, production annotation/table group, and automatic path-array dependency.
