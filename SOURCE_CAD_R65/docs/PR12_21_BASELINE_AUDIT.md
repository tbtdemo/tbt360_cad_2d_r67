# PR #12-#21 shared-baseline audit

Unified branch: `integration/cad360-unified-r66`

This audit compares the original feature PR heads against the unified integration branch.

## Shared baseline confirmed identical

The following shared files are byte-for-byte identical across the feature heads and unified branch where expected:

- `SOURCE_CAD_R65/apps/cadcraft/src/bim_domain.rs`
- `SOURCE_CAD_R65/apps/cadcraft/src/bim_sync.rs`
- `SOURCE_CAD_R65/crates/ui-egui/src/canvas.rs`
- `SOURCE_CAD_R65/crates/ui-egui/src/canvas/draw.rs`
- `SOURCE_CAD_R65/crates/ui-egui/src/canvas/widgets.rs`
- `SOURCE_CAD_R65/crates/ui-egui/src/lib.rs`
- `backend/postgres/BIM_COLLABORATION_SCHEMA.sql`
- `backend/sqlite/BIM_OFFLINE_CACHE_SCHEMA.sql`

`bim_collab.rs` intentionally differs because the unified branch contains later BIM workspace/runtime/QTO integration work.

## Feature modules confirmed present byte-for-byte

- PR #12: `bim_federation.rs`
- PR #13: `bim_api.rs`
- PR #14: `bim_offline.rs`
- PR #15: `bim_workset.rs`
- PR #16: `bim_access.rs`
- PR #18: `bim_coordinates.rs`
- PR #19: `bim_viewport.rs`
- PR #20: `bim_coordination.rs`
- PR #21: `bim_clash.rs`

Each module blob SHA on the unified branch equals the corresponding feature-branch blob SHA.

## Main wiring

`apps/cadcraft/src/main.rs` intentionally differs from every individual feature branch because unified includes all modules together:

- bim_presence
- bim_federation
- bim_api
- bim_offline
- bim_workset
- bim_access
- bim_coordinates
- bim_viewport
- bim_coordination
- bim_clash
- takeoff

No feature module from PR #12-#21 is missing from the unified module declarations.

## Workflow note

`.github/workflows/bim-collaboration.yml` intentionally differs on unified because the build gate was extended to `integration/cad360-unified-r66` and now includes Cargo metadata/workspace check/test gates. The original feature PR workflow blob is otherwise common across PR #12-#21.

## Database migrations confirmed present byte-for-byte

- `001_bim_building_zone_level.sql`
- `002_bim_hierarchical_permissions.sql`
- `003_bim_workset_publish.sql`
- `004_bim_offline_queue.sql`
- `005_bim_coordination_quality.sql`
- `006_bim_federated_model_links.sql`

## Build gate status

GitHub Actions jobs are currently created but fail before step 1 on both `windows-latest` and `ubuntu-latest`.
Observed behavior:
- jobs exist;
- `steps=[]`;
- job logs are unavailable / return BlobNotFound;
- therefore these runs are runner/provisioning failures, not Rust compile/test failures.

Static compile audit remains active until a runner produces actual Cargo logs.
