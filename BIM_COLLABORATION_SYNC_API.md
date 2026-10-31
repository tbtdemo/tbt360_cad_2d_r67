# BIM Collaboration Sync API Contract

Target integration: TBT.Auth (OpenIddict) + TBT Gateway + PostgreSQL.

## Authentication
- Desktop obtains access token from TBT.Auth.
- CAD never stores raw password.
- Offline mode keeps only local profile/account id and cached authorization snapshot.
- Online API receives Bearer access token and derives account id server-side.

## Core endpoints

### GET /api/bim/projects
Projects current account can access.

### GET /api/bim/projects/{projectId}/bootstrap
Returns one consistent bootstrap revision:
- project coordinates
- floors
- members
- scope permissions
- view presets
- scope progress
- project revision

### GET /api/bim/projects/{projectId}/changes?afterRevision={revision}
Incremental remote changes for offline cache.

### POST /api/bim/projects/{projectId}/changesets
Client sends:
- deviceId
- baseProjectRevision
- local change set id
- create/update/delete items
- per-object baseRevision/localRevision

Server behavior:
1. Authorize project/floor/discipline edit scope.
2. Validate object lease when required.
3. Compare object base revision.
4. Apply non-conflicting changes transactionally.
5. Return accepted revisions and conflicts.
6. Increment project revision.

### POST /api/bim/projects/{projectId}/leases/{objectId}
Acquire/renew soft edit lease.

### DELETE /api/bim/projects/{projectId}/leases/{objectId}
Release lease.

### PUT /api/bim/projects/{projectId}/presence
Short-lived presence:
- device id
- floor
- discipline
- selected object
- cursor world position

Presence is not authoritative design data and may use Redis later.

### GET /api/bim/projects/{projectId}/presence
Returns currently active users.

### CRUD /api/bim/projects/{projectId}/issues
Coordination issues and assignments.

### PUT /api/bim/projects/{projectId}/progress/{floorId}/{discipline}
Update design completion percentage/status.

## Conflict response
A conflict item contains:
- objectId/sourceHandle
- baseRevision
- localRevision
- remoteRevision
- localPayload
- remotePayload
- remoteUpdatedBy

Desktop offers:
- Keep mine (new change based on remote revision)
- Accept theirs
- Manual merge
- Duplicate as new object when geometry cannot be merged safely

No silent last-write-wins for geometry.

## Offline cache
Recommended local store: SQLite.

Tables mirror:
- project
- floors
- permissions
- objects
- pending_change_sets
- pending_change_items
- issues
- view_presets
- sync_checkpoint

Each local transaction that modifies BIM-scoped geometry also writes a change item in the same SQLite transaction.

## Multi-viewport
All viewports share one object cache but have separate:
- floor filter
- discipline filter
- owner filter
- overlay alpha
- camera
- edit/reference mode

Only the active edit viewport may create/modify geometry. Reference viewports are read-only by default.

## Safety rules
- Server is authority for permissions and revisions.
- Local cached permission expiry must be bounded.
- Offline edits outside cached edit scope are rejected.
- Entity ownership does not imply permanent exclusive ownership; project role/scope permission governs editing.
- Lease expiry prevents abandoned locks.
- Delete is tombstoned/revisioned before eventual cleanup.
