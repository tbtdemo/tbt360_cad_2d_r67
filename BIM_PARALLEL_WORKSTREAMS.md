# BIM Parallel Workstreams

Nhánh tích hợp: `integration/bim-collaboration`
Nguồn tách ban đầu: `feature/bim-collaboration-workspaces`

## Nhánh 1 — Core Coordinates
Branch: `feature/bim-core-coordinates`
Owner scope:
- Shared Coordinates
- Survey Point
- Project Base Point
- Building
- Zone
- Level/Elevation
- Transform local -> building/zone -> project/shared
Không sở hữu: auth, sync, renderer, issue.

## Nhánh 2 — Viewport / Renderer
Branch: `feature/bim-viewport-renderer`
Owner scope:
- Native VPORTS 1/2/3/4
- Edit/Reference/Coordination/Progress viewport role
- Primary vs Reference render classification
- Overlay opacity
- Read-only reference behavior
Không sở hữu: server sync, DB schema, auth policy.

## Nhánh 3 — Auth / Permissions
Branch: `feature/bim-auth-permissions`
Owner scope:
- Project roles
- Hierarchical scope permissions
- Viewer -> Owner sharing
- Building/Zone/Floor/Discipline/Workset authorization
- View/Edit/Publish rights
Không sở hữu: authentication transport implementation.

## Nhánh 4 — Offline / Sync
Branch: `feature/bim-offline-sync`
Owner scope:
- Offline queue
- SQLite cache contract
- Checkpoint
- Pull/Push state
- Reconnect
- Conflict states
- Lease/presence transport contract
Không sở hữu: BIM geometry/rendering.

## Nhánh 5 — Progress / Issues / Audit
Branch: `feature/bim-progress-issues-audit`
Owner scope:
- Project progress
- Coordination Issue
- Severity/status
- Audit
- Model health
- Publish quality gate
Không sở hữu: actual 3D clash engine.

## Nhánh 6 — Workset / Publish
Branch: `feature/bim-workset-publish`
Owner scope:
- Workset
- Working revision
- Review revision
- Published revision
- Consumed revision
- Publish/consume lifecycle
Không sở hữu: permission evaluation or sync transport.

## Merge gate
Mỗi branch chỉ merge vào integration khi:
1. cargo check --locked -p cadcraft PASS.
2. BIM module tests PASS.
3. Không thay Cargo.lock.
4. Không thêm dependency mới nếu chưa có phê duyệt.
5. Không chạm `main`.
6. Không phá các test BIM cũ.
7. PR giữ draft cho đến khi CI xanh.

## Merge order đề xuất
1. Core Coordinates
2. Auth / Permissions
3. Workset / Publish
4. Offline / Sync
5. Progress / Issues / Audit
6. Viewport / Renderer

Viewport merge cuối vì phụ thuộc scope cuối cùng của Floor/Discipline/Workset/Owner.

Sau khi 6 nhánh vào integration:
- chạy cargo check --workspace --locked
- chạy cargo test --workspace --locked --no-fail-fast
- chạy BIM branch tests
- smoke test CAD + BIM tab
- tuyệt đối chưa merge vào main nếu integration chưa xanh.


## Nhánh 7 — IFC / Clash 3D
Branch: `feature/bim-ifc-clash`
Owner scope:
- IFC/3D clash domain
- 3D AABB broadphase
- bounded candidate generation
- Hard/Clearance/Duplicate clash classes
Không coi giao cắt 2D là clash 3D chính thức.

## Nhánh 8 — Realtime Presence
Branch: `feature/bim-realtime-presence`
Owner scope:
- Heartbeat
- Cursor/selection presence
- expiring object lease
- prune disconnected users
- reconnect-safe realtime state

## Merge order cập nhật
1. Core Coordinates
2. Auth / Permissions
3. Workset / Publish
4. Offline / Sync
5. Realtime Presence
6. Progress / Issues / Audit
7. IFC / Clash
8. Viewport / Renderer

Viewport vẫn merge cuối vì phải tiêu thụ scope cuối cùng của coordinates, auth, workset và collaboration state.
