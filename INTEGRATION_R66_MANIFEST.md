# CAD360 Unified R66 Integration Manifest

Branch: `integration/cad360-unified-r66`

Baseline:
- main: 7f2180f69fa9b561181e4d92258158cb26916701

Integrated workstreams:
- PR #11 BIM Realtime Presence: b01fb6a14137246fc77907c8ca989d486c84a59b
- PR #12 BIM Federated Models: c6300ef92d6433a0ce3ec905a0cb80946f26454b
- PR #13 BIM API Contracts: 063bffe44b9769ca5080181dacbc5192671aae91
- PR #14 BIM Offline Sync: c7530949e8066811f61085426d2e7d52ec32bfab
- PR #15 BIM Workset Publish: f86ad91d0d3e436b6354cc3db037989dbb746365
- PR #16 BIM Auth Permissions: 3e0109f7d160259554b55b86ad067f78a07daaed
- PR #17 QTO Native R134 v2: f73764d4bf0efb59c9cb12ff4d6ab2805aa18ee6
- PR #18 BIM Core Coordinates: 26ddf223d773be157437f1e9b1b9d2862b553379
- PR #19 BIM Viewport Renderer: 404415651e56e5a05ef34a775b9bad9cc1fd1a68
- PR #20 BIM Coordination / Audit: 92754310891d26bcc17ea735d3aa25b1e8219bc6
- PR #21 BIM IFC Clash: 0373286e6078ff6def36a001a497edf5e55c081f

Integration notes:
- PR #11 was merged normally into the unified branch.
- Remaining BIM branches share the same BIM baseline files, so their unique modules/migrations were integrated explicitly to avoid false add/add conflicts.
- main.rs now wires all BIM modules and keeps the native QTO split-view.
- takeoff.rs is the latest QTO Native implementation from PR #17.
- Old PRs are retained for traceability.
