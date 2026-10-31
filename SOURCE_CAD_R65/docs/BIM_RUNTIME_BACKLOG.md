# BIM runtime reconciliation backlog

This checklist tracks TBT360_BIM identity/revision/QTO safety after the central post-command reconciler.

## Runtime rule now enforced after every command

The runtime pass runs after groups, constraints and associativity.

For every entity that already carries `TBT360_BIM`:
- `entity=` is normalized to the real handle.
- If non-BIM entity content changed and the command did not already advance revision, revision is advanced once.
- If the command already advanced revision, the runtime does not double-bump.
- Imported/copied/renumbered entities keep their scope/revision but get the real handle.
- The same rule applies to model space, paper-space layouts and block definitions.
- Deleted entities need no mutation: QTO source validation sees them as missing.
- Undo/Redo remains snapshot-based, so runtime changes are in the same undo step.

## Closed by the runtime common-cause fix

These no longer need per-command revision patches when they edit an entity in place:
- Properties, MatchProp, layer reassignment and rename/repoint changes.
- Text, dimension, multileader, hatch, spline, table and annotation edits.
- Image/underlay clipping or visual property edits.
- Viewport geometry/property edits.
- Associative dimension/hatch/array updates.
- Dynamic block reference property/grip edits.
- Region/curve/3D geometry edits that keep the same handle.
- Any future command that changes an existing TBT360_BIM entity through the normal command transaction.

## Still needs explicit source-scope policy

These create new entities and therefore runtime cannot infer which source BIM scope should be inherited.

Shared helper now implemented: inherit scope only when every source has the same TBT360_BIM scope; otherwise leave the derived object unscoped.

Covered by the shared helper:
- CENTERMARK / CENTERLINE.
- DIVIDE / MEASURE markers.
- DIMCONTINUE / DIMBASELINE from the previous BIM dimension.
- CIRCLE TTR / TTT JSON paths.
- WIPEOUT from a source polyline.

Covered additionally:
- QDIM outputs inherit only when all selected source objects share one BIM scope.
- LEADER/MLEADER can inherit explicit source/sources handles through the same helper.
- TANCIRCLE JSON and interactive TTR/TTT paths preserve their tangent source handles and use the same helper.
- REGION command replaces the selected closed object in place, so central runtime revision logic covers it.
- BREAKLINE is point-driven and has no source entity; it intentionally stays unscoped instead of inventing provenance.

Still needs the same helper/policy wired in only where a real source entity exists:
- Other derived construction objects that explicitly reference source handles.
- TXT2MTXT combined output now keeps BIM scope only when all source texts share one scope; mixed sources strip BIM scope from the surviving handle.
- BURST/EXPLODE only for true multi-source merge cases; ordinary exploded pieces are already covered.
- GROUP copy does not create entities and needs no BIM identity work.

## Import/remap policy now centralized

Closed by runtime + adopt/remap normalization:
- XREF load/reload model objects and nested block objects.
- XBIND copies of dependent blocks.
- INSERT_FILE model and nested block entities.
- Clipboard/cross-drawing nested block definitions.
- Any drawing entering a Session is normalized once through DocState::new.
- Every post-command pass re-normalizes real handles for model/layout/block entities.
- A changed block definition automatically invalidates existing BIM INSERT references once, without double-bumping commands that already did it.

Persistence contracts now added:
- DXF roundtrip keeps handle plus full TBT360_BIM floor/discipline/owner/revision/entity payload across two consecutive reopen cycles.
- DWG R2018 roundtrip has a dedicated TBT360_BIM identity regression through the existing DWG bridge.

Real-file regression contracts are now present for WBLOCK -> INSERT, system clipboard DXF and XREF reload. They still need an actual test runner log before being marked PASS.

## QTO-specific remaining work

Closed:
- Measurement extraction for Region and Hatch now uses canonical geometry.
- Linear/radial/ordinate/arc-length Dimension measurement uses renderer numeric measurement; angular dimensions are intentionally excluded from length totals.
- Block-definition changes with stable INSERT handles are handled centrally by runtime revision invalidation.

Closed / foundation complete:
- QTO v4 now has structured source_path provenance with root_handle plus block_member / array_item steps.
- Nested Block and associative Array source paths resolve to world-space geometry through the canonical QTO measurement engine.
- Nested validation/refresh use the same geometry signature rules as top-level sources.
- Nested auto-relink intentionally refuses guessing when a path breaks; no fake model-space handles are created.
- Root INSERT remains the revision/scope anchor, so block-definition runtime invalidation makes nested QTO stale correctly.

Closed additionally:
- QTO UI can create nested v4 provenance records from selected Block/Array root references by enumerating measurable leaf members.
- Real-file XREF reload regression now checks member TBT360_BIM preservation, remapped entity identity and one-time root INSERT revision invalidation.
- System clipboard uses a real DXF writer/reader regression to prove scope/revision preservation and pasted identity remap.
- WBLOCK -> INSERT real-file regression proves BIM member persistence and EXPLODE scope propagation.

Remaining:
- Direct click-to-pick one nested rendered member (current UI selects a root Block/Array then enumerates its measurable leaves).
- Multi-source derived entity provenance beyond the shared source-scope helper.
- Optional explicit nested-path relink UX if a user chooses the replacement path manually.
- Count remains existence/scope based, not coordinate based.

## BIM workspace remaining work

Closed:
- Workspace load is transactional for persisted workspace state: a failed load restores the previous snapshot.
- Invalid-load regression verifies the existing workspace is unchanged.
- Successful "Mở workspace BIM" immediately reapplies native CAD viewport layout.

Remaining:
- Renderer/view filtering still needs end-to-end visual regression after workspace reopen.

## Integration/build remaining work

- Compare shared baseline files from feature branches #12-#21 against the unified branch.
- cargo metadata --locked.
- cargo check --workspace --locked.
- cargo test --workspace --locked.
- End-to-end Create -> Modify -> Undo -> Redo -> Save -> Reopen regression.
- DXF and DWG persistence gates for handle, TBT360_BIM XDATA, QTO source link and workspace.
