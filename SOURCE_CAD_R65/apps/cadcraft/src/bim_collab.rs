//! Preliminary BIM collaboration workspace for CAD 360 Viet Nam - TBT.
//! This module intentionally owns only coordination metadata/UI. CAD geometry stays in the
//! canonical CadApp/Session and will be bound to these scopes incrementally.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Discipline {
    Architecture,
    Structure,
    Electrical,
    Plumbing,
    Hvac,
    Coordination,
}

impl Discipline {
    pub const ALL: [Self; 6] = [
        Self::Architecture,
        Self::Structure,
        Self::Electrical,
        Self::Plumbing,
        Self::Hvac,
        Self::Coordination,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Architecture => "Kiến trúc",
            Self::Structure => "Kết cấu",
            Self::Electrical => "Điện",
            Self::Plumbing => "Nước",
            Self::Hvac => "Điều hòa/Thông gió",
            Self::Coordination => "Phối hợp",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Architecture => "ARCH", Self::Structure => "STRUCT", Self::Electrical => "ELEC",
            Self::Plumbing => "PLUMB", Self::Hvac => "HVAC", Self::Coordination => "COORD",
        }
    }

    fn from_label(value: &str) -> Option<Self> {
        match value {
            "Kiến trúc" | "ARCH" => Some(Self::Architecture),
            "Kết cấu" | "STRUCT" => Some(Self::Structure),
            "Điện" | "ELEC" => Some(Self::Electrical),
            "Nước" | "PLUMB" => Some(Self::Plumbing),
            "Điều hòa/Thông gió" | "HVAC" => Some(Self::Hvac),
            "Phối hợp" | "COORD" => Some(Self::Coordination),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionMode {
    Offline,
    Online,
}

impl ConnectionMode {
    fn label(self) -> &'static str {
        match self {
            Self::Offline => "Offline cá nhân",
            Self::Online => "Online cộng tác",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewLayout {
    One,
    Two,
    Three,
    Four,
}

impl ViewLayout {
    fn slots(self) -> usize {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Three => 3,
            Self::Four => 4,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::One => "1 cửa sổ",
            Self::Two => "2 cửa sổ",
            Self::Three => "3 cửa sổ",
            Self::Four => "4 cửa sổ",
        }
    }

    fn code(self) -> &'static str {
        match self { Self::One=>"1", Self::Two=>"2", Self::Three=>"3", Self::Four=>"4" }
    }

    fn parse(value:&str)->Option<Self> {
        match value { "1"=>Some(Self::One), "2"=>Some(Self::Two), "3"=>Some(Self::Three), "4"=>Some(Self::Four), _=>None }
    }
}

#[derive(Clone, Debug)]
pub struct Floor {
    pub id: u64,
    pub name: String,
    /// BIM absolute/project elevation in metres.
    pub elevation_m: f64,
    pub height_m: f64,
}

impl Floor {
    pub fn world_z(&self, local_z_m: f64) -> f64 {
        self.elevation_m + local_z_m
    }
}

#[derive(Clone, Debug)]
pub struct UserAccess {
    pub account_id: String,
    pub display_name: String,
    pub discipline: Discipline,
    pub can_edit: bool,
    pub can_view_other_disciplines: bool,
    pub can_view_other_floors: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthStatus {
    SignedOut,
    OfflineProfile,
    OnlineAuthenticated,
}

#[derive(Clone, Debug)]
pub struct AuthSession {
    pub status: AuthStatus,
    pub account_id: String,
    pub display_name: String,
    pub email: String,
}

#[derive(Clone, Debug, Default)]
pub struct ModelHealth {
    pub total_entities: usize,
    pub unscoped_entities: usize,
    pub missing_owner: usize,
    pub invalid_floor_refs: usize,
    pub pending_changes: usize,
    pub conflicts: usize,
    pub open_issues: usize,
}

impl ModelHealth {
    pub fn is_clean(&self) -> bool {
        self.unscoped_entities == 0
            && self.missing_owner == 0
            && self.invalid_floor_refs == 0
            && self.pending_changes == 0
            && self.conflicts == 0
            && self.open_issues == 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectSyncState {
    Clean,
    Local,
    Pending,
    Conflict,
    Locked,
}

impl ObjectSyncState {
    fn label(self) -> &'static str {
        match self {
            Self::Clean => "Đã đồng bộ",
            Self::Local => "Local",
            Self::Pending => "Chờ đồng bộ",
            Self::Conflict => "Xung đột",
            Self::Locked => "Đang khóa",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeStatus {
    NotStarted,
    InProgress,
    Review,
    Complete,
}

impl ScopeStatus {
    fn label(self) -> &'static str {
        match self {
            Self::NotStarted => "Chưa bắt đầu",
            Self::InProgress => "Đang thực hiện",
            Self::Review => "Đang kiểm tra",
            Self::Complete => "Hoàn thành",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScopeProgress {
    pub floor_id: u64,
    pub discipline: Discipline,
    pub percent: u8,
    pub status: ScopeStatus,
    pub updated_by: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueStatus {
    Open,
    InProgress,
    Resolved,
}

impl IssueStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Open => "Mở",
            Self::InProgress => "Đang xử lý",
            Self::Resolved => "Đã xử lý",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AuditEntry {
    pub sequence: u64,
    pub account_id: String,
    pub action: String,
    pub floor_id: Option<u64>,
    pub discipline: Option<Discipline>,
    pub handle: Option<cadcraft_doc::Handle>,
    pub detail: String,
}

#[derive(Clone, Debug)]
pub struct CoordinationCandidate {
    pub floor_id: u64,
    pub a: cadcraft_doc::Handle,
    pub discipline_a: Discipline,
    pub b: cadcraft_doc::Handle,
    pub discipline_b: Discipline,
}

#[derive(Clone, Debug)]
pub struct CoordinationIssue {
    pub id: u64,
    pub floor_id: u64,
    pub source_discipline: Discipline,
    pub target_discipline: Discipline,
    pub handle: Option<cadcraft_doc::Handle>,
    pub assigned_to: String,
    pub description: String,
    pub status: IssueStatus,
}

#[derive(Clone, Debug)]
pub struct AccessRule {
    pub account_id: String,
    pub floor_id: Option<u64>,
    pub discipline: Option<Discipline>,
    pub can_view: bool,
    pub can_edit: bool,
}

#[derive(Clone, Debug)]
pub struct ShareGrant {
    /// The account allowed to see/edit another account's design scope.
    pub viewer_account_id: String,
    /// Owner of the shared design objects.
    pub owner_account_id: String,
    pub floor_id: Option<u64>,
    pub discipline: Option<Discipline>,
    pub can_view: bool,
    pub can_edit: bool,
}



#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Edit,
    Reference,
}

impl ViewMode {
    fn label(self) -> &'static str {
        match self {
            Self::Edit => "Chỉnh sửa",
            Self::Reference => "Tham chiếu",
        }
    }

    fn code(self) -> &'static str { match self { Self::Edit=>"edit", Self::Reference=>"reference" } }

    fn parse(value:&str)->Option<Self> { match value { "edit"=>Some(Self::Edit), "reference"=>Some(Self::Reference), _=>None } }
}

#[derive(Clone, Debug)]
pub struct ViewPreset {
    pub name: String,
    pub account_id: String,
    pub layout: ViewLayout,
    pub slots: Vec<ViewSlot>,
}

#[derive(Clone, Debug)]
pub struct ViewSlot {
    pub floor_id: u64,
    pub discipline: Discipline,
    pub owner_account_id: Option<String>,
    pub overlay_opacity: f32,
    pub show_other_users: bool,
    pub mode: ViewMode,
}

#[derive(Clone, Debug)]
pub struct ProjectCoordinates {
    pub origin_x_m: f64,
    pub origin_y_m: f64,
    pub origin_z_m: f64,
    pub rotation_deg: f64,
}

impl Default for ProjectCoordinates {
    fn default() -> Self {
        Self { origin_x_m: 0.0, origin_y_m: 0.0, origin_z_m: 0.0, rotation_deg: 0.0 }
    }
}

const XDATA_APP: &str = "TBT360_BIM";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BimEntityMeta {
    pub floor_id: u64,
    pub discipline: Discipline,
    pub owner_account_id: String,
    pub revision: u64,
    pub entity_handle: Option<cadcraft_doc::Handle>,
}

impl BimEntityMeta {
    fn discipline_code(&self) -> &'static str {
        match self.discipline {
            Discipline::Architecture => "ARCH",
            Discipline::Structure => "STRUCT",
            Discipline::Electrical => "ELEC",
            Discipline::Plumbing => "PLUMB",
            Discipline::Hvac => "HVAC",
            Discipline::Coordination => "COORD",
        }
    }

    fn from_discipline_code(code: &str) -> Option<Discipline> {
        match code {
            "ARCH" => Some(Discipline::Architecture),
            "STRUCT" => Some(Discipline::Structure),
            "ELEC" => Some(Discipline::Electrical),
            "PLUMB" => Some(Discipline::Plumbing),
            "HVAC" => Some(Discipline::Hvac),
            "COORD" => Some(Discipline::Coordination),
            _ => None,
        }
    }

    fn payload(&self) -> String {
        let mut payload=format!("floor={};discipline={};owner={};revision={}",self.floor_id,self.discipline_code(),self.owner_account_id.replace(';',"_"),self.revision);
        if let Some(handle)=self.entity_handle { payload.push_str(&format!(";entity={}",handle.hex())); }
        payload
    }

    fn parse(payload: &str) -> Option<Self> {
        let mut floor_id = None;
        let mut discipline = None;
        let mut owner_account_id = None;
        let mut revision = None;
        let mut entity_handle = None;
        for part in payload.split(';') {
            let (key, value) = part.split_once('=')?;
            match key {
                "floor" => floor_id = value.parse().ok(),
                "discipline" => discipline = Self::from_discipline_code(value),
                "owner" => owner_account_id = Some(value.to_string()),
                "revision" => revision = value.parse().ok(),
                "entity" => entity_handle = cadcraft_doc::Handle::parse_hex(value),
                _ => {}
            }
        }
        Some(Self { floor_id: floor_id?, discipline: discipline?, owner_account_id: owner_account_id?, revision: revision?, entity_handle })
    }
}

fn set_bim_meta(common: &mut cadcraft_doc::Common, meta: &BimEntityMeta) {
    let mut out = Vec::with_capacity(common.xdata.len() + 2);
    let mut skip_payload = false;
    for tag in &common.xdata {
        if tag.code == 1001 && tag.value == XDATA_APP {
            skip_payload = true;
            continue;
        }
        if skip_payload && tag.code >= 1000 && tag.code <= 1071 {
            skip_payload = false;
            continue;
        }
        skip_payload = false;
        out.push(tag.clone());
    }
    out.push(cadcraft_doc::RawTag { code: 1001, value: XDATA_APP.into() });
    out.push(cadcraft_doc::RawTag { code: 1000, value: meta.payload() });
    common.xdata = out;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityScope {
    pub floor_id: u64,
    pub discipline: String,
    pub owner_account_id: String,
    pub revision: u64,
    pub entity_handle: Option<cadcraft_doc::Handle>,
}

pub fn entity_scope(app: &cadcraft_ui_egui::CadApp, handle: cadcraft_doc::Handle) -> Option<EntityScope> {
    let doc=app.session.doc().ok()?;
    let entity=doc.model.get(handle)?;
    let meta=bim_meta(&entity.common)?;
    Some(EntityScope {
        floor_id: meta.floor_id,
        discipline: meta.discipline.code().to_string(),
        owner_account_id: meta.owner_account_id,
        revision: meta.revision,
        entity_handle: meta.entity_handle,
    })
}

fn bim_meta(common: &cadcraft_doc::Common) -> Option<BimEntityMeta> {
    common.xdata.windows(2).find_map(|pair| {
        if pair[0].code == 1001 && pair[0].value == XDATA_APP && pair[1].code == 1000 {
            BimEntityMeta::parse(&pair[1].value)
        } else {
            None
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeOperation {
    Create,
    Update,
    Delete,
}

#[derive(Clone, Debug)]
pub struct ChangeItem {
    pub handle: cadcraft_doc::Handle,
    pub operation: ChangeOperation,
    pub base_revision: u64,
    pub local_revision: u64,
}

#[derive(Clone, Debug)]
pub struct ChangeSet {
    pub id: u64,
    pub account_id: String,
    pub items: Vec<ChangeItem>,
    pub synced: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConflictResolution {
    KeepMine,
    AcceptTheirs,
    DuplicateMine,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncConflict {
    pub handle: cadcraft_doc::Handle,
    pub base_revision: u64,
    pub local_revision: u64,
    pub remote_revision: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Presence {
    pub account_id: String,
    pub floor_id: u64,
    pub discipline: Discipline,
    pub selected_handle: Option<cadcraft_doc::Handle>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectLease {
    pub handle: cadcraft_doc::Handle,
    pub account_id: String,
    pub revision: u64,
}


fn detect_conflict(item: &ChangeItem, remote_revision: u64) -> Option<SyncConflict> {
    if remote_revision == item.base_revision {
        None
    } else {
        Some(SyncConflict {
            handle: item.handle,
            base_revision: item.base_revision,
            local_revision: item.local_revision,
            remote_revision,
        })
    }
}

fn slot_matches(meta: &BimEntityMeta, slot: &ViewSlot) -> bool {
    if meta.floor_id != slot.floor_id || meta.discipline != slot.discipline {
        return false;
    }
    match &slot.owner_account_id {
        Some(owner) => &meta.owner_account_id == owner,
        None => true,
    }
}

pub struct State {
    pub connection: ConnectionMode,
    pub layout: ViewLayout,
    pub current_account_id: String,
    pub coordinates: ProjectCoordinates,
    pub floors: Vec<Floor>,
    pub users: Vec<UserAccess>,
    pub views: Vec<ViewSlot>,
    pub view_presets: Vec<ViewPreset>,
    pub dirty_local_changes: usize,
    pub active_floor_id: u64,
    pub active_discipline: Discipline,
    pub pending_changes: Vec<ChangeSet>,
    next_change_set_id: u64,
    pub presence: Vec<Presence>,
    pub leases: Vec<ObjectLease>,
    pub conflicts: Vec<SyncConflict>,
    pub auth: AuthSession,
    pub access_rules: Vec<AccessRule>,
    pub share_grants: Vec<ShareGrant>,
    pub progress: Vec<ScopeProgress>,
    pub issues: Vec<CoordinationIssue>,
    pub coordination_candidates: Vec<CoordinationCandidate>,
    pub audit_log: Vec<AuditEntry>,
    next_audit_sequence: u64,
    next_issue_id: u64,
    pub project_id: String,
    pub project_name: String,
    remote_revisions: Vec<(cadcraft_doc::Handle, u64)>,
    work_session_active: bool,
    work_session_known_handles: Vec<cadcraft_doc::Handle>,
}

impl Default for State {
    fn default() -> Self {
        let floors = vec![
            Floor { id: 1, name: "Tầng 1".into(), elevation_m: 0.0, height_m: 3.6 },
            Floor { id: 2, name: "Tầng 2".into(), elevation_m: 3.6, height_m: 3.6 },
            Floor { id: 3, name: "Tầng 3".into(), elevation_m: 7.2, height_m: 3.6 },
        ];
        let users = vec![
            UserAccess { account_id: "local-owner".into(), display_name: "Người dùng hiện tại".into(), discipline: Discipline::Architecture, can_edit: true, can_view_other_disciplines: true, can_view_other_floors: true },
            UserAccess { account_id: "struct-01".into(), display_name: "Kỹ sư kết cấu".into(), discipline: Discipline::Structure, can_edit: false, can_view_other_disciplines: false, can_view_other_floors: true },
            UserAccess { account_id: "mep-elec-01".into(), display_name: "Kỹ sư điện".into(), discipline: Discipline::Electrical, can_edit: false, can_view_other_disciplines: false, can_view_other_floors: true },
            UserAccess { account_id: "mep-water-01".into(), display_name: "Kỹ sư nước".into(), discipline: Discipline::Plumbing, can_edit: false, can_view_other_disciplines: false, can_view_other_floors: true },
        ];
        let views = vec![
            ViewSlot { floor_id: 1, discipline: Discipline::Architecture, owner_account_id: Some("local-owner".into()), overlay_opacity: 1.0, show_other_users: false, mode: ViewMode::Edit },
            ViewSlot { floor_id: 1, discipline: Discipline::Structure, owner_account_id: None, overlay_opacity: 0.35, show_other_users: true, mode: ViewMode::Reference },
            ViewSlot { floor_id: 1, discipline: Discipline::Electrical, owner_account_id: None, overlay_opacity: 0.35, show_other_users: true, mode: ViewMode::Reference },
            ViewSlot { floor_id: 1, discipline: Discipline::Plumbing, owner_account_id: None, overlay_opacity: 0.35, show_other_users: true, mode: ViewMode::Reference },
        ];
        Self {
            connection: ConnectionMode::Offline,
            layout: ViewLayout::Two,
            current_account_id: "local-owner".into(),
            coordinates: ProjectCoordinates::default(),
            floors,
            users,
            views,
            view_presets: Vec::new(),
            dirty_local_changes: 0,
            active_floor_id: 1,
            active_discipline: Discipline::Architecture,
            pending_changes: Vec::new(),
            next_change_set_id: 1,
            presence: Vec::new(),
            leases: Vec::new(),
            conflicts: Vec::new(),
            auth: AuthSession {
                status: AuthStatus::OfflineProfile,
                account_id: "local-owner".into(),
                display_name: "Người dùng hiện tại".into(),
                email: String::new(),
            },
            access_rules: vec![
                AccessRule { account_id: "local-owner".into(), floor_id: None, discipline: None, can_view: true, can_edit: true },
                AccessRule { account_id: "struct-01".into(), floor_id: None, discipline: Some(Discipline::Structure), can_view: true, can_edit: true },
                AccessRule { account_id: "mep-elec-01".into(), floor_id: None, discipline: Some(Discipline::Electrical), can_view: true, can_edit: true },
                AccessRule { account_id: "mep-water-01".into(), floor_id: None, discipline: Some(Discipline::Plumbing), can_view: true, can_edit: true },
            ],
            share_grants: vec![
                ShareGrant { viewer_account_id: "local-owner".into(), owner_account_id: "struct-01".into(), floor_id: None, discipline: Some(Discipline::Structure), can_view: true, can_edit: false },
                ShareGrant { viewer_account_id: "local-owner".into(), owner_account_id: "mep-elec-01".into(), floor_id: None, discipline: Some(Discipline::Electrical), can_view: true, can_edit: false },
                ShareGrant { viewer_account_id: "local-owner".into(), owner_account_id: "mep-water-01".into(), floor_id: None, discipline: Some(Discipline::Plumbing), can_view: true, can_edit: false },
            ],
            progress: vec![
                ScopeProgress { floor_id: 1, discipline: Discipline::Architecture, percent: 20, status: ScopeStatus::InProgress, updated_by: "local-owner".into() },
                ScopeProgress { floor_id: 1, discipline: Discipline::Structure, percent: 10, status: ScopeStatus::InProgress, updated_by: "struct-01".into() },
                ScopeProgress { floor_id: 1, discipline: Discipline::Electrical, percent: 0, status: ScopeStatus::NotStarted, updated_by: "mep-elec-01".into() },
                ScopeProgress { floor_id: 1, discipline: Discipline::Plumbing, percent: 0, status: ScopeStatus::NotStarted, updated_by: "mep-water-01".into() },
            ],
            issues: Vec::new(),
            coordination_candidates: Vec::new(),
            audit_log: Vec::new(),
            next_audit_sequence: 1,
            next_issue_id: 1,
            project_id: "project-local-001".into(),
            project_name: "Dự án BIM".into(),
            remote_revisions: Vec::new(),
            work_session_active: false,
            work_session_known_handles: Vec::new(),
        }
    }
}

impl State {
    fn save_view_preset(&mut self, name: String) {
        let name = name.trim().to_string();
        if name.is_empty() { return; }
        let preset = ViewPreset {
            name: name.clone(),
            account_id: self.current_account_id.clone(),
            layout: self.layout,
            slots: self.views.clone(),
        };
        if let Some(existing) = self.view_presets.iter_mut().find(|p| p.name == name && p.account_id == self.current_account_id) {
            *existing = preset;
        } else {
            self.view_presets.push(preset);
        }
        self.audit("VIEW_PRESET_SAVED", Some(self.active_floor_id), Some(self.active_discipline), None, name);
    }

    fn apply_view_preset(&mut self, name: &str) -> bool {
        let Some(preset) = self.view_presets.iter().find(|p| p.name == name && p.account_id == self.current_account_id).cloned() else { return false };
        self.layout = preset.layout;
        self.views = preset.slots;
        self.audit("VIEW_PRESET_APPLIED", Some(self.active_floor_id), Some(self.active_discipline), None, name.to_string());
        true
    }

    fn active_native_view_index(&self, app: &cadcraft_ui_egui::CadApp) -> usize {
        let tiles = app.session.model_tiles();
        if tiles.is_empty() {
            0
        } else {
            tiles.iter().position(|t| t.active).unwrap_or(0).min(self.views.len().saturating_sub(1))
        }
    }

    fn sync_scope_from_active_view(&mut self, app: &cadcraft_ui_egui::CadApp) {
        let index = self.active_native_view_index(app);
        if let Some(slot) = self.views.get(index) {
            self.active_floor_id = slot.floor_id;
            self.active_discipline = slot.discipline;
        }
    }

    fn can_edit_active_view(&self, app: &cadcraft_ui_egui::CadApp) -> bool {
        let index = self.active_native_view_index(app);
        self.views.get(index).is_some_and(|slot| {
            slot.mode == ViewMode::Edit
                && self.has_scope_access(&self.current_account_id, slot.floor_id, slot.discipline, true)
        })
    }

    fn apply_native_viewports(&self, app: &mut cadcraft_ui_egui::CadApp) -> Result<(), String> {
        let params = match self.layout {
            ViewLayout::One => serde_json::json!({ "option": "single" }),
            ViewLayout::Two => serde_json::json!({ "option": "2", "arrangement": "vertical" }),
            ViewLayout::Three => serde_json::json!({ "option": "3", "arrangement": "right" }),
            ViewLayout::Four => serde_json::json!({ "option": "4" }),
        };
        app.session.execute("-vports", &params).map(|_| ()).map_err(|e| e.to_string())
    }

    fn start_work_session(&mut self, app: &cadcraft_ui_egui::CadApp) {
        self.work_session_known_handles = app.session.doc().map(|d| d.model.handles()).unwrap_or_default();
        self.work_session_active = true;
    }

    fn stop_work_session(&mut self) {
        self.work_session_active = false;
        self.work_session_known_handles.clear();
    }

    fn tag_new_entities_in_work_session(&mut self, app: &mut cadcraft_ui_egui::CadApp) {
        if !self.work_session_active || self.auth.status == AuthStatus::SignedOut || !self.can_edit_active_view(app) {
            return;
        }
        let current_handles = app.session.doc().map(|d| d.model.handles()).unwrap_or_default();
        let new_handles = current_handles.iter().copied().filter(|h| !self.work_session_known_handles.contains(h)).collect::<Vec<_>>();
        if new_handles.is_empty() {
            return;
        }
        let floor_id = self.active_floor_id;
        let discipline = self.active_discipline;
        let owner = self.current_account_id.clone();
        if let Ok(doc) = app.session.doc_mut() {
            for handle in &new_handles {
                let existing_revision = doc.model.get(*handle).and_then(|e| bim_meta(&e.common)).map(|m| m.revision).unwrap_or(0);
                if doc.model.modify(*handle, |entity| {
                    set_bim_meta(&mut entity.common, &BimEntityMeta {
                        floor_id,
                        discipline,
                        owner_account_id: owner.clone(),
                        revision: existing_revision + 1,
                        entity_handle: Some(*handle),
                    });
                }) {
                    self.queue_update(*handle, existing_revision, existing_revision + 1);
                }
            }
        }
        self.work_session_known_handles = current_handles;
    }

    fn can_publish(&self, app: &cadcraft_ui_egui::CadApp) -> bool {
        self.auth.status != AuthStatus::SignedOut && self.model_health(app).is_clean()
    }

    fn model_health(&self, app: &cadcraft_ui_egui::CadApp) -> ModelHealth {
        let mut health = ModelHealth::default();
        if let Ok(doc) = app.session.doc() {
            for entity in doc.model.iter() {
                health.total_entities += 1;
                match bim_meta(&entity.common) {
                    None => health.unscoped_entities += 1,
                    Some(meta) => {
                        if meta.owner_account_id.trim().is_empty() {
                            health.missing_owner += 1;
                        }
                        if !self.floors.iter().any(|f| f.id == meta.floor_id) {
                            health.invalid_floor_refs += 1;
                        }
                    }
                }
            }
        }
        health.pending_changes = self.pending_changes.iter().filter(|c| !c.synced).map(|c| c.items.len()).sum();
        health.conflicts = self.conflicts.len();
        health.open_issues = self.issues.iter().filter(|i| i.status != IssueStatus::Resolved).count();
        health
    }

    fn object_sync_state(&self, handle: cadcraft_doc::Handle) -> ObjectSyncState {
        if self.conflicts.iter().any(|c| c.handle == handle) {
            return ObjectSyncState::Conflict;
        }
        if self.leases.iter().any(|l| l.handle == handle && l.account_id != self.current_account_id) {
            return ObjectSyncState::Locked;
        }
        if self.pending_changes.iter().any(|c| !c.synced && c.items.iter().any(|i| i.handle == handle)) {
            return if self.connection == ConnectionMode::Online { ObjectSyncState::Pending } else { ObjectSyncState::Local };
        }
        ObjectSyncState::Clean
    }

    fn scope_entity_count(&self, app: &cadcraft_ui_egui::CadApp, floor_id: u64, discipline: Discipline) -> usize {
        let Ok(doc) = app.session.doc() else { return 0 };
        doc.model.iter().filter(|entity| {
            bim_meta(&entity.common).is_some_and(|m| m.floor_id == floor_id && m.discipline == discipline)
        }).count()
    }

    fn scope_progress(&self, floor_id: u64, discipline: Discipline) -> Option<&ScopeProgress> {
        self.progress.iter().find(|p| p.floor_id == floor_id && p.discipline == discipline)
    }

    fn scan_2d_coordination_candidates(&mut self, app: &cadcraft_ui_egui::CadApp, floor_id: u64, max_pairs: usize) {
        self.coordination_candidates.clear();
        let Ok(doc) = app.session.doc() else { return };
        let scoped = doc.model.iter().filter_map(|entity| {
            let meta = bim_meta(&entity.common)?;
            (meta.floor_id == floor_id).then_some((entity.clone(), meta))
        }).collect::<Vec<_>>();
        let mut checked = 0usize;
        for i in 0..scoped.len() {
            for j in (i + 1)..scoped.len() {
                if checked >= max_pairs { return; }
                checked += 1;
                let (a, ma) = &scoped[i];
                let (b, mb) = &scoped[j];
                if ma.discipline == mb.discipline { continue; }
                let ba = cadcraft_doc::entity_bounds(doc, a.as_ref(), 0);
                let bb = cadcraft_doc::entity_bounds(doc, b.as_ref(), 0);
                if ba.is_empty() || bb.is_empty() || !ba.intersects(&bb) { continue; }
                self.coordination_candidates.push(CoordinationCandidate {
                    floor_id,
                    a: a.handle,
                    discipline_a: ma.discipline,
                    b: b.handle,
                    discipline_b: mb.discipline,
                });
            }
        }
    }

    fn audit(&mut self, action: impl Into<String>, floor_id: Option<u64>, discipline: Option<Discipline>, handle: Option<cadcraft_doc::Handle>, detail: impl Into<String>) {
        let entry = AuditEntry {
            sequence: self.next_audit_sequence,
            account_id: self.current_account_id.clone(),
            action: action.into(),
            floor_id,
            discipline,
            handle,
            detail: detail.into(),
        };
        self.next_audit_sequence += 1;
        self.audit_log.push(entry);
        if self.audit_log.len() > 5000 {
            let remove = self.audit_log.len() - 5000;
            self.audit_log.drain(0..remove);
        }
    }

    fn ensure_progress_scope(&mut self, floor_id: u64, discipline: Discipline) {
        if self.progress.iter().any(|p| p.floor_id == floor_id && p.discipline == discipline) {
            return;
        }
        self.progress.push(ScopeProgress {
            floor_id,
            discipline,
            percent: 0,
            status: ScopeStatus::NotStarted,
            updated_by: self.current_account_id.clone(),
        });
    }

    fn create_issue_from_selection(&mut self, handle: Option<cadcraft_doc::Handle>, target_discipline: Discipline) {
        self.issues.push(CoordinationIssue {
            id: self.next_issue_id,
            floor_id: self.active_floor_id,
            source_discipline: self.active_discipline,
            target_discipline,
            handle,
            assigned_to: self.users.iter().find(|u| u.discipline == target_discipline).map(|u| u.account_id.clone()).unwrap_or_default(),
            description: "Kiểm tra phối hợp tại vị trí/đối tượng được chọn".into(),
            status: IssueStatus::Open,
        });
        let issue_id = self.next_issue_id;
        self.next_issue_id += 1;
        self.audit("COORDINATION_ISSUE_CREATED", Some(self.active_floor_id), Some(self.active_discipline), handle, format!("Issue {} -> {}", issue_id, target_discipline.label()));
    }

    fn queue_update(&mut self, handle: cadcraft_doc::Handle, base_revision: u64, local_revision: u64) {
        let change = ChangeSet {
            id: self.next_change_set_id,
            account_id: self.current_account_id.clone(),
            items: vec![ChangeItem { handle, operation: ChangeOperation::Update, base_revision, local_revision }],
            synced: false,
        };
        self.next_change_set_id += 1;
        self.pending_changes.push(change);
        self.dirty_local_changes = self.pending_changes.iter().filter(|c| !c.synced).map(|c| c.items.len()).sum();
        self.audit("ENTITY_UPDATE_QUEUED", Some(self.active_floor_id), Some(self.active_discipline), Some(handle), format!("Revision {} -> {}", base_revision, local_revision));
    }

    fn mark_all_synced(&mut self) {
        for change in &mut self.pending_changes {
            change.synced = true;
        }
        self.dirty_local_changes = 0;
        self.audit("SYNC_ALL_CONFIRMED", Some(self.active_floor_id), Some(self.active_discipline), None, "All pending local changes marked synced");
    }

    fn remote_revision(&self, handle: cadcraft_doc::Handle) -> u64 {
        self.remote_revisions.iter().find(|(h, _)| *h == handle).map(|(_, r)| *r).unwrap_or(0)
    }

    fn set_remote_revision(&mut self, handle: cadcraft_doc::Handle, revision: u64) {
        if let Some((_, r)) = self.remote_revisions.iter_mut().find(|(h, _)| *h == handle) {
            *r = revision;
        } else {
            self.remote_revisions.push((handle, revision));
        }
    }

    fn resolve_conflict(&mut self, handle: cadcraft_doc::Handle, resolution: ConflictResolution) {
        let Some(conflict) = self.conflicts.iter().find(|c| c.handle == handle).cloned() else { return };
        match resolution {
            ConflictResolution::KeepMine => {
                if let Some(change) = self.pending_changes.iter_mut().find(|c| !c.synced && c.items.iter().any(|i| i.handle == handle)) {
                    for item in &mut change.items {
                        if item.handle == handle {
                            item.base_revision = conflict.remote_revision;
                            item.local_revision = conflict.remote_revision + 1;
                        }
                    }
                }
                self.audit("CONFLICT_KEEP_MINE", Some(self.active_floor_id), Some(self.active_discipline), Some(handle), format!("Remote r{} -> local r{}", conflict.remote_revision, conflict.remote_revision + 1));
            }
            ConflictResolution::AcceptTheirs => {
                for change in &mut self.pending_changes {
                    if change.items.iter().any(|i| i.handle == handle) {
                        change.synced = true;
                    }
                }
                self.set_remote_revision(handle, conflict.remote_revision);
                self.audit("CONFLICT_ACCEPT_THEIRS", Some(self.active_floor_id), Some(self.active_discipline), Some(handle), format!("Accepted remote revision {}", conflict.remote_revision));
            }
            ConflictResolution::DuplicateMine => {
                self.audit("CONFLICT_DUPLICATE_MINE", Some(self.active_floor_id), Some(self.active_discipline), Some(handle), "Geometry duplication requires CAD clone command integration");
            }
        }
        self.conflicts.retain(|c| c.handle != handle);
        self.dirty_local_changes = self.pending_changes.iter().filter(|c| !c.synced).map(|c| c.items.len()).sum();
    }

    fn sync_pending_changes(&mut self) {
        self.conflicts.clear();
        let mut accepted = Vec::new();
        for change in &self.pending_changes {
            if change.synced { continue; }
            let mut change_conflicts = false;
            for item in &change.items {
                let remote = self.remote_revision(item.handle);
                if let Some(conflict) = detect_conflict(item, remote) {
                    self.conflicts.push(conflict);
                    change_conflicts = true;
                } else {
                    accepted.push((change.id, item.handle, item.local_revision));
                }
            }
            if change_conflicts {
                continue;
            }
        }
        for (change_id, handle, revision) in accepted {
            self.set_remote_revision(handle, revision);
            if let Some(change) = self.pending_changes.iter_mut().find(|c| c.id == change_id) {
                change.synced = true;
            }
        }
        self.dirty_local_changes = self.pending_changes.iter().filter(|c| !c.synced).map(|c| c.items.len()).sum();
    }

    fn snapshot_json(&self) -> serde_json::Value {
        let floors = self.floors.iter().map(|f| serde_json::json!({
            "id": f.id, "name": f.name, "elevation_m": f.elevation_m, "height_m": f.height_m
        })).collect::<Vec<_>>();
        let views = self.views.iter().map(|v| serde_json::json!({
            "floor_id": v.floor_id,
            "discipline": v.discipline.label(),
            "owner_account_id": v.owner_account_id,
            "overlay_opacity": v.overlay_opacity,
            "show_other_users": v.show_other_users,
            "mode": v.mode.label()
        })).collect::<Vec<_>>();
        let view_presets = self.view_presets.iter().map(|p| serde_json::json!({
            "name": p.name,
            "account_id": p.account_id,
            "layout": p.layout.slots(),
            "slots": p.slots.iter().map(|v| serde_json::json!({
                "floor_id": v.floor_id,
                "discipline": v.discipline.label(),
                "owner_account_id": v.owner_account_id,
                "overlay_opacity": v.overlay_opacity,
                "show_other_users": v.show_other_users,
                "mode": v.mode.label()
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>();
        let access_rules = self.access_rules.iter().map(|r| serde_json::json!({
            "account_id": r.account_id,
            "floor_id": r.floor_id,
            "discipline": r.discipline.map(Discipline::label),
            "can_view": r.can_view,
            "can_edit": r.can_edit
        })).collect::<Vec<_>>();
        let share_grants = self.share_grants.iter().map(|g| serde_json::json!({
            "viewer_account_id": g.viewer_account_id,
            "owner_account_id": g.owner_account_id,
            "floor_id": g.floor_id,
            "discipline": g.discipline.map(Discipline::label),
            "can_view": g.can_view,
            "can_edit": g.can_edit
        })).collect::<Vec<_>>();
        let progress = self.progress.iter().map(|p| serde_json::json!({
            "floor_id": p.floor_id,
            "discipline": p.discipline.label(),
            "percent": p.percent,
            "status": p.status.label(),
            "updated_by": p.updated_by
        })).collect::<Vec<_>>();
        let issues = self.issues.iter().map(|i| serde_json::json!({
            "id": i.id,
            "floor_id": i.floor_id,
            "source_discipline": i.source_discipline.label(),
            "target_discipline": i.target_discipline.label(),
            "handle": i.handle.map(|h| h.hex()),
            "assigned_to": i.assigned_to,
            "description": i.description,
            "status": i.status.label()
        })).collect::<Vec<_>>();
        serde_json::json!({
            "version": 2,
            "project_id": self.project_id,
            "project_name": self.project_name,
            "coordinates": {
                "origin_x_m": self.coordinates.origin_x_m,
                "origin_y_m": self.coordinates.origin_y_m,
                "origin_z_m": self.coordinates.origin_z_m,
                "rotation_deg": self.coordinates.rotation_deg
            },
            "layout": self.layout.slots(),
            "floors": floors,
            "views": views,
            "view_presets": view_presets,
            "access_rules": access_rules,
            "share_grants": share_grants,
            "active_floor_id": self.active_floor_id,
            "active_discipline": self.active_discipline.label(),
            "progress": progress,
            "issues": issues
        })
    }

    fn load_snapshot_json_in_place(&mut self,value:&serde_json::Value)->Result<(),String> {
        if value.get("version").and_then(serde_json::Value::as_u64) != Some(2) { return Err("Phiên bản BIM workspace không được hỗ trợ".into()); }
        self.project_id = value.get("project_id").and_then(serde_json::Value::as_str).unwrap_or("project-local-001").trim().to_string();
        if self.project_id.is_empty() { return Err("BIM workspace thiếu project_id".into()); }
        self.project_name = value.get("project_name").and_then(serde_json::Value::as_str).unwrap_or("Dự án BIM").trim().to_string();
        if let Some(c) = value.get("coordinates") {
            let ox=c.get("origin_x_m").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
            let oy=c.get("origin_y_m").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
            let oz=c.get("origin_z_m").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
            let rot=c.get("rotation_deg").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
            if ![ox,oy,oz,rot].into_iter().all(f64::is_finite) { return Err("Shared coordinates không hợp lệ".into()); }
            self.coordinates.origin_x_m=ox;
            self.coordinates.origin_y_m=oy;
            self.coordinates.origin_z_m=oz;
            self.coordinates.rotation_deg=rot;
        }
        if let Some(items) = value.get("floors").and_then(serde_json::Value::as_array) {
            let mut floors = Vec::new();
            let mut floor_ids = std::collections::BTreeSet::new();
            for item in items {
                let id = item.get("id").and_then(serde_json::Value::as_u64).ok_or_else(|| "Floor thiếu id".to_string())?;
                if id==0 || !floor_ids.insert(id) { return Err("Floor id không hợp lệ hoặc bị trùng".into()); }
                let name = item.get("name").and_then(serde_json::Value::as_str).unwrap_or("Tầng").trim().to_string();
                let elevation_m = item.get("elevation_m").and_then(serde_json::Value::as_f64).unwrap_or(0.0);
                let height_m = item.get("height_m").and_then(serde_json::Value::as_f64).unwrap_or(3.6);
                if name.is_empty() || !elevation_m.is_finite() || !height_m.is_finite() || height_m<=0.0 { return Err("Floor không hợp lệ".into()); }
                floors.push(Floor { id, name, elevation_m, height_m });
            }
            if floors.is_empty() { return Err("BIM workspace phải có ít nhất một tầng".into()); }
            self.floors = floors;
        }
        self.layout = match value.get("layout").and_then(serde_json::Value::as_u64).unwrap_or(2) {
            1 => ViewLayout::One,
            3 => ViewLayout::Three,
            4 => ViewLayout::Four,
            _ => ViewLayout::Two,
        };
        if let Some(items) = value.get("views").and_then(serde_json::Value::as_array) {
            let mut views = Vec::new();
            for item in items.iter().take(4) {
                let floor_id = item.get("floor_id").and_then(serde_json::Value::as_u64).unwrap_or(1);
                if !self.floors.iter().any(|f|f.id==floor_id) { return Err("View tham chiếu tầng không tồn tại".into()); }
                let discipline = item.get("discipline").and_then(serde_json::Value::as_str).and_then(Discipline::from_label).unwrap_or(Discipline::Architecture);
                let owner_account_id = item.get("owner_account_id").and_then(serde_json::Value::as_str).map(str::to_string);
                let overlay_opacity = item.get("overlay_opacity").and_then(serde_json::Value::as_f64).unwrap_or(0.35).clamp(0.05, 1.0) as f32;
                let show_other_users = item.get("show_other_users").and_then(serde_json::Value::as_bool).unwrap_or(true);
                let mode = if item.get("mode").and_then(serde_json::Value::as_str) == Some("Chỉnh sửa") { ViewMode::Edit } else { ViewMode::Reference };
                views.push(ViewSlot { floor_id, discipline, owner_account_id, overlay_opacity, show_other_users, mode });
            }
            if !views.is_empty() {
                while views.len() < 4 {
                    views.push(ViewSlot { floor_id: 1, discipline: Discipline::Coordination, owner_account_id: None, overlay_opacity: 0.35, show_other_users: true, mode: ViewMode::Reference });
                }
                self.views = views;
            }
        }
        if let Some(items) = value.get("view_presets").and_then(serde_json::Value::as_array) {
            let mut presets = Vec::new();
            for item in items {
                let name = item.get("name").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
                let account_id = item.get("account_id").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
                let layout = match item.get("layout").and_then(serde_json::Value::as_u64).unwrap_or(2) {
                    1 => ViewLayout::One, 3 => ViewLayout::Three, 4 => ViewLayout::Four, _ => ViewLayout::Two
                };
                let mut slots = Vec::new();
                if let Some(slot_items) = item.get("slots").and_then(serde_json::Value::as_array) {
                    for slot in slot_items.iter().take(4) {
                        slots.push(ViewSlot {
                            floor_id: slot.get("floor_id").and_then(serde_json::Value::as_u64).unwrap_or(1),
                            discipline: slot.get("discipline").and_then(serde_json::Value::as_str).and_then(Discipline::from_label).unwrap_or(Discipline::Architecture),
                            owner_account_id: slot.get("owner_account_id").and_then(serde_json::Value::as_str).map(str::to_string),
                            overlay_opacity: slot.get("overlay_opacity").and_then(serde_json::Value::as_f64).unwrap_or(0.35).clamp(0.05, 1.0) as f32,
                            show_other_users: slot.get("show_other_users").and_then(serde_json::Value::as_bool).unwrap_or(true),
                            mode: if slot.get("mode").and_then(serde_json::Value::as_str) == Some("Chỉnh sửa") { ViewMode::Edit } else { ViewMode::Reference },
                        });
                    }
                }
                if !name.is_empty() && !account_id.is_empty() && !slots.is_empty() {
                    while slots.len() < 4 {
                        slots.push(ViewSlot { floor_id: 1, discipline: Discipline::Coordination, owner_account_id: None, overlay_opacity: 0.35, show_other_users: true, mode: ViewMode::Reference });
                    }
                    presets.push(ViewPreset { name, account_id, layout, slots });
                }
            }
            self.view_presets = presets;
        }
        if let Some(items) = value.get("access_rules").and_then(serde_json::Value::as_array) {
            self.access_rules = items.iter().filter_map(|item| {
                Some(AccessRule {
                    account_id: item.get("account_id")?.as_str()?.to_string(),
                    floor_id: item.get("floor_id").and_then(serde_json::Value::as_u64),
                    discipline: item.get("discipline").and_then(serde_json::Value::as_str).and_then(Discipline::from_label),
                    can_view: item.get("can_view").and_then(serde_json::Value::as_bool).unwrap_or(false),
                    can_edit: item.get("can_edit").and_then(serde_json::Value::as_bool).unwrap_or(false),
                })
            }).collect();
        }
        if let Some(items) = value.get("share_grants").and_then(serde_json::Value::as_array) {
            self.share_grants = items.iter().filter_map(|item| {
                Some(ShareGrant {
                    viewer_account_id: item.get("viewer_account_id")?.as_str()?.to_string(),
                    owner_account_id: item.get("owner_account_id")?.as_str()?.to_string(),
                    floor_id: item.get("floor_id").and_then(serde_json::Value::as_u64),
                    discipline: item.get("discipline").and_then(serde_json::Value::as_str).and_then(Discipline::from_label),
                    can_view: item.get("can_view").and_then(serde_json::Value::as_bool).unwrap_or(false),
                    can_edit: item.get("can_edit").and_then(serde_json::Value::as_bool).unwrap_or(false),
                })
            }).collect();
        }
        self.active_floor_id = value.get("active_floor_id").and_then(serde_json::Value::as_u64).unwrap_or_else(|| self.floors.first().map(|f| f.id).unwrap_or(1));
        if !self.floors.iter().any(|f|f.id==self.active_floor_id) { return Err("active_floor_id không tồn tại".into()); }
        self.active_discipline = value.get("active_discipline").and_then(serde_json::Value::as_str).and_then(Discipline::from_label).unwrap_or(Discipline::Architecture);
        if let Some(items) = value.get("progress").and_then(serde_json::Value::as_array) {
            self.progress = items.iter().filter_map(|item| {
                let discipline = item.get("discipline")?.as_str().and_then(Discipline::from_label)?;
                let percent = item.get("percent").and_then(serde_json::Value::as_u64).unwrap_or(0).min(100) as u8;
                let status = match item.get("status").and_then(serde_json::Value::as_str) {
                    Some("Đang thực hiện") => ScopeStatus::InProgress,
                    Some("Đang kiểm tra") => ScopeStatus::Review,
                    Some("Hoàn thành") => ScopeStatus::Complete,
                    _ => ScopeStatus::NotStarted,
                };
                Some(ScopeProgress {
                    floor_id: item.get("floor_id")?.as_u64()?,
                    discipline,
                    percent,
                    status,
                    updated_by: item.get("updated_by").and_then(serde_json::Value::as_str).unwrap_or_default().to_string(),
                })
            }).collect();
        }
        if let Some(items) = value.get("issues").and_then(serde_json::Value::as_array) {
            self.issues = items.iter().filter_map(|item| {
                let handle = item.get("handle").and_then(serde_json::Value::as_str).and_then(cadcraft_doc::Handle::parse_hex);
                Some(CoordinationIssue {
                    id: item.get("id")?.as_u64()?,
                    floor_id: item.get("floor_id")?.as_u64()?,
                    source_discipline: item.get("source_discipline")?.as_str().and_then(Discipline::from_label)?,
                    target_discipline: item.get("target_discipline")?.as_str().and_then(Discipline::from_label)?,
                    handle,
                    assigned_to: item.get("assigned_to").and_then(serde_json::Value::as_str).unwrap_or_default().to_string(),
                    description: item.get("description").and_then(serde_json::Value::as_str).unwrap_or_default().to_string(),
                    status: match item.get("status").and_then(serde_json::Value::as_str) {
                        Some("Đang xử lý") => IssueStatus::InProgress,
                        Some("Đã xử lý") => IssueStatus::Resolved,
                        _ => IssueStatus::Open,
                    },
                })
            }).collect();
            self.next_issue_id = self.issues.iter().map(|i| i.id).max().unwrap_or(0) + 1;
        }
        Ok(())
    }

    fn login_offline(&mut self, account_id: String, display_name: String) {
        self.auth.status = AuthStatus::OfflineProfile;
        self.auth.account_id = account_id.clone();
        self.auth.display_name = display_name;
        self.auth.email.clear();
        self.current_account_id = account_id;
        self.connection = ConnectionMode::Offline;
    }

    fn login_online_stub(&mut self, account_id: String, display_name: String, email: String) {
        self.auth.status = AuthStatus::OnlineAuthenticated;
        self.auth.account_id = account_id.clone();
        self.auth.display_name = display_name;
        self.auth.email = email;
        self.current_account_id = account_id;
        self.connection = ConnectionMode::Online;
    }

    fn sign_out(&mut self) {
        self.auth.status = AuthStatus::SignedOut;
        self.auth.account_id.clear();
        self.auth.display_name.clear();
        self.auth.email.clear();
    }

    fn has_scope_access(&self, account_id: &str, floor_id: u64, discipline: Discipline, edit: bool) -> bool {
        self.access_rules.iter().any(|rule| {
            rule.account_id == account_id
                && rule.floor_id.is_none_or(|f| f == floor_id)
                && rule.discipline.is_none_or(|d| d == discipline)
                && if edit { rule.can_edit } else { rule.can_view }
        })
    }

    fn has_share_access(&self, viewer: &str, owner: &str, floor_id: u64, discipline: Discipline, edit: bool) -> bool {
        if viewer == owner {
            return self.has_scope_access(viewer, floor_id, discipline, edit);
        }
        self.share_grants.iter().any(|grant| {
            grant.viewer_account_id == viewer
                && grant.owner_account_id == owner
                && grant.floor_id.is_none_or(|f| f == floor_id)
                && grant.discipline.is_none_or(|d| d == discipline)
                && if edit { grant.can_edit } else { grant.can_view }
        })
    }

    fn acquire_lease(&mut self, handle: cadcraft_doc::Handle, revision: u64) -> Result<(), String> {
        if let Some(existing) = self.leases.iter().find(|l| l.handle == handle) {
            if existing.account_id != self.current_account_id {
                return Err(format!("Đối tượng đang được {} chỉnh sửa", existing.account_id));
            }
            return Ok(());
        }
        self.leases.push(ObjectLease { handle, account_id: self.current_account_id.clone(), revision });
        Ok(())
    }

    fn release_lease(&mut self, handle: cadcraft_doc::Handle) {
        self.leases.retain(|l| !(l.handle == handle && l.account_id == self.current_account_id));
    }

    fn update_presence(&mut self, selected: Option<cadcraft_doc::Handle>) {
        let floor_id = self.active_floor_id;
        let discipline = self.active_discipline;
        if let Some(p) = self.presence.iter_mut().find(|p| p.account_id == self.current_account_id) {
            p.floor_id = floor_id;
            p.discipline = discipline;
            p.selected_handle = selected;
        } else {
            self.presence.push(Presence {
                account_id: self.current_account_id.clone(),
                floor_id,
                discipline,
                selected_handle: selected,
            });
        }
    }

    pub fn active_user(&self) -> Option<&UserAccess> {
        self.users.iter().find(|u| u.account_id == self.current_account_id)
    }

    pub fn can_view(&self, target: &UserAccess, target_floor: u64, active_floor: u64) -> bool {
        let Some(me) = self.active_user() else { return false };
        if target.account_id == me.account_id {
            return target_floor == active_floor || me.can_view_other_floors;
        }
        let broad = (me.can_view_other_disciplines || target.discipline == me.discipline)
            && (target_floor == active_floor || me.can_view_other_floors);
        broad
            && self.has_scope_access(&me.account_id, target_floor, target.discipline, false)
            && self.has_share_access(&me.account_id, &target.account_id, target_floor, target.discipline, false)
    }

    fn load_snapshot_json(&mut self,value:&serde_json::Value)->Result<(),String> {
        let backup=self.snapshot_json();
        match self.load_snapshot_json_in_place(value) {
            Ok(())=>Ok(()),
            Err(err)=>{
                let _=self.load_snapshot_json_in_place(&backup);
                Err(err)
            }
        }
    }

    fn viewport_entity_count(&self, app: &cadcraft_ui_egui::CadApp, slot: &ViewSlot) -> usize {
        let Ok(doc) = app.session.doc() else { return 0 };
        doc.model.iter().filter(|entity| {
            let Some(meta) = bim_meta(&entity.common) else { return false };
            if !slot_matches(&meta, slot) { return false; }
            let Some(target) = self.users.iter().find(|u| u.account_id == meta.owner_account_id) else { return false };
            if !slot.show_other_users && meta.owner_account_id != self.current_account_id { return false; }
            self.can_view(target, meta.floor_id, self.active_floor_id)
        }).count()
    }

    pub fn world_position(&self, floor_id: u64, local_x_m: f64, local_y_m: f64, local_z_m: f64) -> Option<[f64; 3]> {
        let floor = self.floors.iter().find(|f| f.id == floor_id)?;
        let angle = self.coordinates.rotation_deg.to_radians();
        let cos = angle.cos();
        let sin = angle.sin();
        let rotated_x = local_x_m * cos - local_y_m * sin;
        let rotated_y = local_x_m * sin + local_y_m * cos;
        Some([
            self.coordinates.origin_x_m + rotated_x,
            self.coordinates.origin_y_m + rotated_y,
            self.coordinates.origin_z_m + floor.world_z(local_z_m),
        ])
    }
}

pub fn enforce_selection_scope(app: &mut cadcraft_ui_egui::CadApp, state: &State) {
    let index = state.active_native_view_index(app);
    let Some(slot) = state.views.get(index) else { return };
    if slot.mode == ViewMode::Reference || !state.has_scope_access(&state.current_account_id, slot.floor_id, slot.discipline, true) {
        if !app.session.selection().is_empty() {
            app.session.set_selection(Vec::new());
        }
        return;
    }
    let Some(filter) = app.tile_render_filters.get(index).filter(|f| f.enabled) else { return };
    let current = app.session.selection();
    let allowed = current.into_iter().filter(|h| filter.primary_handles.binary_search(h).is_ok()).collect::<Vec<_>>();
    if allowed != app.session.selection() {
        app.session.set_selection(allowed);
    }
}

pub fn sync_render_filters(app: &mut cadcraft_ui_egui::CadApp, state: &State) {
    let Ok(doc) = app.session.doc() else {
        app.tile_render_filters.clear();
        return;
    };
    let has_bim_meta = doc.model.iter().any(|entity| bim_meta(&entity.common).is_some());
    if !has_bim_meta {
        app.tile_render_filters.clear();
        return;
    }

    let mut filters = Vec::with_capacity(state.views.len());
    for slot in &state.views {
        let mut primary_handles = Vec::new();
        let mut reference_handles = Vec::new();
        for entity in doc.model.iter() {
            let Some(meta) = bim_meta(&entity.common) else {
                reference_handles.push(entity.handle);
                continue;
            };
            let owner_matches = slot.owner_account_id.as_ref().is_none_or(|owner| owner == &meta.owner_account_id);
            if meta.floor_id == slot.floor_id && meta.discipline == slot.discipline && owner_matches {
                primary_handles.push(entity.handle);
                continue;
            }
            if meta.floor_id != slot.floor_id {
                continue;
            }
            let Some(target) = state.users.iter().find(|u| u.account_id == meta.owner_account_id) else { continue };
            if (!slot.show_other_users && meta.owner_account_id != state.current_account_id)
                || !state.can_view(target, meta.floor_id, state.active_floor_id)
            {
                continue;
            }
            reference_handles.push(entity.handle);
        }
        primary_handles.sort_unstable();
        primary_handles.dedup();
        reference_handles.sort_unstable();
        reference_handles.dedup();
        filters.push(cadcraft_ui_egui::TileRenderFilter {
            enabled: true,
            primary_handles,
            reference_handles,
            reference_opacity: slot.overlay_opacity,
        });
    }
    app.tile_render_filters = filters;
}

pub fn panel(ui: &mut egui::Ui, app: &mut cadcraft_ui_egui::CadApp, state: &mut State) {
    ui.heading("BIM Cộng tác");
    ui.label("Khung sơ bộ cho thiết kế nhiều bộ môn, nhiều tầng/cao độ và phối hợp online/offline trong cùng một file dự án.");

    {
        let health = state.model_health(app);
        ui.group(|ui| {
            ui.strong("Model Health");
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Entity: {}", health.total_entities));
                ui.label(format!("Chưa gắn BIM: {}", health.unscoped_entities));
                ui.label(format!("Thiếu owner: {}", health.missing_owner));
                ui.label(format!("Sai tầng: {}", health.invalid_floor_refs));
                ui.label(format!("Pending: {}", health.pending_changes));
                ui.label(format!("Conflict: {}", health.conflicts));
                ui.label(format!("Issue mở: {}", health.open_issues));
                if health.is_clean() {
                    ui.colored_label(egui::Color32::LIGHT_GREEN, "Model sạch");
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "Cần xử lý trước khi publish");
                }
                let can_publish = state.can_publish(app);
                if ui.add_enabled(can_publish, egui::Button::new("Publish BIM / Đồng bộ chính thức")).clicked() {
                    state.audit("PROJECT_PUBLISHED", Some(state.active_floor_id), Some(state.active_discipline), None, format!("Project {} published from CAD workspace", state.project_id));
                }
            });
        });
    }

    ui.group(|ui| {
        ui.strong("Dự án BIM");
        ui.horizontal(|ui| {
            ui.label("Mã dự án:");
            ui.text_edit_singleline(&mut state.project_id);
            ui.label("Tên dự án:");
            ui.text_edit_singleline(&mut state.project_name);
        });
        ui.horizontal(|ui| {
            if ui.button("Lưu workspace BIM").clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("TBT BIM Workspace", &["json"]).set_file_name("TBT360_BIM_WORKSPACE.json").save_file() {
                    if let Ok(data) = serde_json::to_vec_pretty(&state.snapshot_json()) {
                        let _ = std::fs::write(path, data);
                    }
                }
            }
            if ui.button("Mở workspace BIM").clicked() {
                if let Some(path) = rfd::FileDialog::new().add_filter("TBT BIM Workspace", &["json"]).pick_file() {
                    if let Ok(data) = std::fs::read(path) {
                        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&data) {
                            match state.load_snapshot_json(&value) {
                                Ok(())=>{
                                    if let Err(e)=state.apply_native_viewports(app) { eprintln!("BIM viewport: {e}"); }
                                }
                                Err(e)=>eprintln!("BIM workspace: {e}"),
                            }
                        }
                    }
                }
            }
        });
    });

    ui.group(|ui| {
        ui.strong("Đăng nhập / Phiên làm việc");
        match state.auth.status {
            AuthStatus::SignedOut => {
                ui.label("Chưa đăng nhập. Online sẽ nối TBT.Auth/OpenIddict; không lưu mật khẩu trong file CAD.");
            }
            AuthStatus::OfflineProfile => {
                ui.label(format!("Offline: {} ({})", state.auth.display_name, state.auth.account_id));
            }
            AuthStatus::OnlineAuthenticated => {
                ui.label(format!("Online: {} — {}", state.auth.display_name, state.auth.email));
            }
        }
        ui.horizontal(|ui| {
            if ui.button("Dùng profile offline").clicked() {
                state.login_offline("local-owner".into(), "Người dùng hiện tại".into());
            }
            if ui.button("Mô phỏng đăng nhập TBT.Auth").clicked() {
                state.login_online_stub("local-owner".into(), "Người dùng hiện tại".into(), "user@tbt.local".into());
            }
            if ui.button("Đăng xuất").clicked() {
                state.sign_out();
            }
        });
    });

    ui.horizontal(|ui| {
        ui.label("Chế độ:");
        for mode in [ConnectionMode::Offline, ConnectionMode::Online] {
            ui.selectable_value(&mut state.connection, mode, mode.label());
        }
        ui.separator();
        egui::ComboBox::from_id_salt("bim_current_account")
            .selected_text(state.users.iter().find(|u| u.account_id == state.current_account_id).map(|u| u.display_name.as_str()).unwrap_or("Chưa đăng nhập"))
            .show_ui(ui, |ui| {
                for user in &state.users {
                    ui.selectable_value(&mut state.current_account_id, user.account_id.clone(), &user.display_name);
                }
            });
        if state.connection == ConnectionMode::Offline {
            ui.label(format!("Thay đổi cục bộ chưa đồng bộ: {}", state.dirty_local_changes));
        }
    });

    ui.separator();
    state.sync_scope_from_active_view(app);
    ui.group(|ui| {
        ui.strong("Scope thiết kế hiện hành");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("bim_active_floor")
                .selected_text(state.floors.iter().find(|f| f.id == state.active_floor_id).map(|f| f.name.as_str()).unwrap_or("Chưa chọn tầng"))
                .show_ui(ui, |ui| {
                    for floor in &state.floors {
                        ui.selectable_value(&mut state.active_floor_id, floor.id, &floor.name);
                    }
                });
            egui::ComboBox::from_id_salt("bim_active_discipline")
                .selected_text(state.active_discipline.label())
                .show_ui(ui, |ui| {
                    for discipline in Discipline::ALL {
                        ui.selectable_value(&mut state.active_discipline, discipline, discipline.label());
                    }
                });
        });

        state.tag_new_entities_in_work_session(app);
        ui.horizontal(|ui| {
            if !state.work_session_active {
                let can_edit = state.can_edit_active_view(app);
                if ui.add_enabled(can_edit, egui::Button::new("Bắt đầu phiên thiết kế theo scope này")).clicked() {
                    state.start_work_session(app);
                }
                if !can_edit {
                    ui.label("Viewport hiện tại chỉ tham chiếu hoặc tài khoản không có quyền sửa");
                }
            } else if ui.button("Kết thúc phiên thiết kế").clicked() {
                state.stop_work_session();
            }
            ui.label(if state.work_session_active {
                "Đang tự động gắn Floor/Bộ môn/Owner cho đối tượng mới"
            } else {
                "Chưa bật tự động scope"
            });
        });

        let selected = app.session.selection().to_vec();
        state.update_presence(selected.first().copied());
        ui.label(format!("Đối tượng đang chọn: {}", selected.len()));
        if let Some(handle) = selected.first().copied() {
            ui.label(format!("Trạng thái đồng bộ đối tượng {}: {}", handle.hex(), state.object_sync_state(handle).label()));
        }
        if ui.button("Gán tầng / bộ môn / chủ sở hữu cho đối tượng đang chọn").clicked() && !selected.is_empty() {
            let owner = state.current_account_id.clone();
            let floor_id = state.active_floor_id;
            let discipline = state.active_discipline;
            if let Ok(doc) = app.session.doc_mut() {
                for handle in selected {
                    let mut previous_revision = 0;
                    if let Some(entity) = doc.model.get(handle) {
                        previous_revision = bim_meta(&entity.common).map(|m| m.revision).unwrap_or(0);
                    }
                    let next_revision = previous_revision + 1;
                    match state.acquire_lease(handle, previous_revision) {
                        Ok(()) => {
                            if doc.model.modify(handle, |entity| {
                                set_bim_meta(&mut entity.common, &BimEntityMeta {
                                    floor_id,
                                    discipline,
                                    owner_account_id: owner.clone(),
                                    revision: next_revision,
                                    entity_handle: Some(handle),
                                });
                            }) {
                                state.queue_update(handle, previous_revision, next_revision);
                            }
                            state.release_lease(handle);
                        }
                        Err(message) => {
                            eprintln!("BIM lock: {message}");
                        }
                    }
                }
            }
        }

        if state.connection == ConnectionMode::Online && state.dirty_local_changes > 0 && ui.button("Đồng bộ thay đổi với server (mô phỏng)").clicked() {
            state.sync_pending_changes();
        }
        if !state.pending_changes.is_empty() {
            ui.label(format!("ChangeSet local: {} | thay đổi chưa đồng bộ: {}", state.pending_changes.len(), state.dirty_local_changes));
        }
        if !state.leases.is_empty() {
            ui.label(format!("Đối tượng đang khóa mềm: {}", state.leases.len()));
        }
        if !state.conflicts.is_empty() {
            ui.colored_label(egui::Color32::LIGHT_RED, format!("Xung đột cần xử lý: {}", state.conflicts.len()));
            let conflicts = state.conflicts.clone();
            egui::Grid::new("bim_conflicts").striped(true).show(ui, |ui| {
                ui.strong("Đối tượng"); ui.strong("Base"); ui.strong("Local"); ui.strong("Remote"); ui.strong("Xử lý"); ui.end_row();
                for conflict in conflicts {
                    ui.label(conflict.handle.hex());
                    ui.label(conflict.base_revision.to_string());
                    ui.label(conflict.local_revision.to_string());
                    ui.label(conflict.remote_revision.to_string());
                    ui.horizontal(|ui| {
                        if ui.button("Giữ của tôi").clicked() { state.resolve_conflict(conflict.handle, ConflictResolution::KeepMine); }
                        if ui.button("Nhận bản server").clicked() { state.resolve_conflict(conflict.handle, ConflictResolution::AcceptTheirs); }
                        if ui.button("Tạo bản sao").clicked() { state.resolve_conflict(conflict.handle, ConflictResolution::DuplicateMine); }
                    });
                    ui.end_row();
                }
            });
        }
    });

    ui.separator();
    ui.collapsing("Tọa độ dự án / BIM", |ui| {
        egui::Grid::new("bim_coordinates").show(ui, |ui| {
            ui.label("Origin X (m)"); ui.add(egui::DragValue::new(&mut state.coordinates.origin_x_m)); ui.end_row();
            ui.label("Origin Y (m)"); ui.add(egui::DragValue::new(&mut state.coordinates.origin_y_m)); ui.end_row();
            ui.label("Origin Z (m)"); ui.add(egui::DragValue::new(&mut state.coordinates.origin_z_m)); ui.end_row();
            ui.label("Góc xoay dự án (°)"); ui.add(egui::DragValue::new(&mut state.coordinates.rotation_deg)); ui.end_row();
        });
        ui.label("Mọi đối tượng BIM sẽ được lưu theo project coordinates + tầng/cao độ + tọa độ local.");
    });

    ui.collapsing("Tầng / Cao độ", |ui| {
        egui::Grid::new("bim_floors").striped(true).show(ui, |ui| {
            ui.strong("Tên tầng"); ui.strong("Cao độ (m)"); ui.strong("Chiều cao tầng (m)"); ui.end_row();
            for floor in &mut state.floors {
                ui.text_edit_singleline(&mut floor.name);
                ui.add(egui::DragValue::new(&mut floor.elevation_m).speed(0.1));
                ui.add(egui::DragValue::new(&mut floor.height_m).speed(0.1).range(0.1..=100.0));
                ui.end_row();
            }
        });
        if ui.button("+ Thêm tầng").clicked() {
            let next_id = state.floors.iter().map(|f| f.id).max().unwrap_or(0) + 1;
            let elevation = state.floors.last().map(|f| f.elevation_m + f.height_m).unwrap_or(0.0);
            state.floors.push(Floor { id: next_id, name: format!("Tầng {}", next_id), elevation_m: elevation, height_m: 3.6 });
        }
    });

    ui.separator();
    ui.collapsing("Project Navigator BIM", |ui| {
        ui.label("Theo dõi nhanh từng tầng/bộ môn, số đối tượng và mức độ hoàn thành; click để chuyển scope làm việc.");
        egui::Grid::new("bim_project_navigator").striped(true).show(ui, |ui| {
            ui.strong("Tầng"); ui.strong("Bộ môn"); ui.strong("Đối tượng"); ui.strong("Tiến độ"); ui.strong("Trạng thái"); ui.strong("Thao tác"); ui.end_row();
            let floors = state.floors.clone();
            for floor in floors {
                for discipline in Discipline::ALL {
                    let count = state.scope_entity_count(app, floor.id, discipline);
                    let (percent, status) = state.scope_progress(floor.id, discipline)
                        .map(|p| (p.percent, p.status.label()))
                        .unwrap_or((0, ScopeStatus::NotStarted.label()));
                    ui.label(&floor.name);
                    ui.label(discipline.label());
                    ui.label(count.to_string());
                    ui.label(format!("{}%", percent));
                    ui.label(status);
                    if ui.button(format!("Mở {} / {}", floor.name, discipline.label())).clicked() {
                        state.active_floor_id = floor.id;
                        state.active_discipline = discipline;
                        if let Some(slot) = state.views.first_mut() {
                            slot.floor_id = floor.id;
                            slot.discipline = discipline;
                            slot.mode = ViewMode::Edit;
                        }
                    }
                    ui.end_row();
                }
            }
        });
    });

    ui.separator();
    ui.collapsing("Tiến độ thiết kế toàn dự án", |ui| {
        let floor_ids = state.floors.iter().map(|f| f.id).collect::<Vec<_>>();
        for floor_id in floor_ids {
            for discipline in Discipline::ALL {
                state.ensure_progress_scope(floor_id, discipline);
            }
        }
        egui::Grid::new("bim_progress").striped(true).show(ui, |ui| {
            ui.strong("Tầng"); ui.strong("Bộ môn"); ui.strong("%"); ui.strong("Trạng thái"); ui.strong("Cập nhật bởi"); ui.end_row();
            for progress in &mut state.progress {
                ui.label(state.floors.iter().find(|f| f.id == progress.floor_id).map(|f| f.name.as_str()).unwrap_or("?"));
                ui.label(progress.discipline.label());
                ui.add(egui::Slider::new(&mut progress.percent, 0..=100));
                let new_status = if progress.percent == 0 { ScopeStatus::NotStarted } else if progress.percent >= 100 { ScopeStatus::Complete } else if progress.status == ScopeStatus::Review { ScopeStatus::Review } else { ScopeStatus::InProgress };
                progress.status = new_status;
                egui::ComboBox::from_id_salt(("scope_status", progress.floor_id, progress.discipline.label()))
                    .selected_text(progress.status.label())
                    .show_ui(ui, |ui| {
                        for status in [ScopeStatus::NotStarted, ScopeStatus::InProgress, ScopeStatus::Review, ScopeStatus::Complete] {
                            ui.selectable_value(&mut progress.status, status, status.label());
                        }
                    });
                ui.label(&progress.updated_by);
                ui.end_row();
            }
        });
    });

    ui.separator();
    ui.collapsing("Bố cục cá nhân / View Preset", |ui| {
        ui.horizontal(|ui| {
            if ui.button("Lưu preset hiện tại").clicked() {
                let name = format!("{}-{}-{}V", state.current_account_id, state.active_floor_id, state.layout.slots());
                state.save_view_preset(name);
            }
            ui.label(format!("Preset của tài khoản: {}", state.view_presets.iter().filter(|p| p.account_id == state.current_account_id).count()));
        });
        let names = state.view_presets.iter().filter(|p| p.account_id == state.current_account_id).map(|p| p.name.clone()).collect::<Vec<_>>();
        for name in names {
            ui.horizontal(|ui| {
                ui.label(&name);
                if ui.button("Áp dụng").clicked() {
                    state.apply_view_preset(&name);
                }
            });
        }
    });

    ui.horizontal(|ui| {
        ui.label("Chia màn hình:");
        for layout in [ViewLayout::One, ViewLayout::Two, ViewLayout::Three, ViewLayout::Four] {
            ui.selectable_value(&mut state.layout, layout, layout.label());
        }
        if ui.button("Áp dụng viewport CAD thật").clicked() {
            if let Err(e) = state.apply_native_viewports(app) {
                eprintln!("BIM viewport: {e}");
            }
        }
    });
    ui.label("Bố cục dùng trực tiếp VPORTS của engine CAD; mỗi tile giữ camera riêng. Scope BIM của từng tile được quản lý ở cấu hình bên dưới.");

    let slots = state.layout.slots();
    egui::Grid::new("bim_view_grid").num_columns(if slots == 1 { 1 } else { 2 }).spacing([8.0, 8.0]).show(ui, |ui| {
        for i in 0..slots {
            let visible_count = state.viewport_entity_count(app, &state.views[i]);
            let slot = &mut state.views[i];
            ui.group(|ui| {
                ui.set_min_width(320.0);
                ui.strong(format!("Viewport {}", i + 1));
                egui::ComboBox::from_id_salt(("floor", i))
                    .selected_text(state.floors.iter().find(|f| f.id == slot.floor_id).map(|f| f.name.as_str()).unwrap_or("Chưa chọn tầng"))
                    .show_ui(ui, |ui| {
                        for floor in &state.floors {
                            ui.selectable_value(&mut slot.floor_id, floor.id, &floor.name);
                        }
                    });
                egui::ComboBox::from_id_salt(("discipline", i))
                    .selected_text(slot.discipline.label())
                    .show_ui(ui, |ui| {
                        for discipline in Discipline::ALL {
                            ui.selectable_value(&mut slot.discipline, discipline, discipline.label());
                        }
                    });
                ui.checkbox(&mut slot.show_other_users, "Hiển thị thiết kế của người khác");
                egui::ComboBox::from_id_salt(("view_mode", i))
                    .selected_text(slot.mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut slot.mode, ViewMode::Edit, ViewMode::Edit.label());
                        ui.selectable_value(&mut slot.mode, ViewMode::Reference, ViewMode::Reference.label());
                    });
                ui.add(egui::Slider::new(&mut slot.overlay_opacity, 0.05..=1.0).text("Độ đậm overlay"));
                ui.label(format!("Đối tượng phù hợp bộ lọc: {}", visible_count));
                ui.label(if i == 0 { "Viewport làm việc chính" } else { "Viewport phối hợp/xem trực tiếp" });
            });
            if slots > 1 && i % 2 == 1 { ui.end_row(); }
        }
    });

    ui.separator();
    ui.collapsing("Kiểm tra giao cắt 2D sơ bộ", |ui| {
        ui.label("Chỉ dùng bounding box 2D của entity cùng tầng để tìm ứng viên cần kiểm tra; không phải clash BIM 3D chính thức.");
        if ui.button("Quét tầng hiện tại").clicked() {
            state.scan_2d_coordination_candidates(app, state.active_floor_id, 20_000);
        }
        ui.label(format!("Ứng viên giao cắt: {}", state.coordination_candidates.len()));
        egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
            egui::Grid::new("bim_coord_candidates").striped(true).show(ui, |ui| {
                ui.strong("A"); ui.strong("Bộ môn A"); ui.strong("B"); ui.strong("Bộ môn B"); ui.end_row();
                for c in state.coordination_candidates.iter().take(200) {
                    ui.label(c.a.hex());
                    ui.label(c.discipline_a.label());
                    ui.label(c.b.hex());
                    ui.label(c.discipline_b.label());
                    ui.end_row();
                }
            });
        });
    });

    ui.collapsing("Issue phối hợp / Coordination", |ui| {
        let selected = app.session.selection().first().copied();
        ui.horizontal(|ui| {
            if ui.button("+ Issue sang Kết cấu").clicked() { state.create_issue_from_selection(selected, Discipline::Structure); }
            if ui.button("+ Issue sang Điện").clicked() { state.create_issue_from_selection(selected, Discipline::Electrical); }
            if ui.button("+ Issue sang Nước").clicked() { state.create_issue_from_selection(selected, Discipline::Plumbing); }
            if ui.button("+ Issue sang HVAC").clicked() { state.create_issue_from_selection(selected, Discipline::Hvac); }
        });
        egui::Grid::new("bim_issues").striped(true).show(ui, |ui| {
            ui.strong("ID"); ui.strong("Tầng"); ui.strong("Nguồn"); ui.strong("Đích"); ui.strong("Đối tượng"); ui.strong("Phụ trách"); ui.strong("Trạng thái"); ui.end_row();
            for issue in &mut state.issues {
                ui.label(issue.id.to_string());
                ui.label(state.floors.iter().find(|f| f.id == issue.floor_id).map(|f| f.name.as_str()).unwrap_or("?"));
                ui.label(issue.source_discipline.label());
                ui.label(issue.target_discipline.label());
                if let Some(handle) = issue.handle {
                    let response = ui.button(handle.hex());
                    if response.clicked() {
                        app.session.set_selection(vec![handle]);
                    }
                } else {
                    ui.label("-");
                }
                ui.text_edit_singleline(&mut issue.assigned_to);
                egui::ComboBox::from_id_salt(("issue_status", issue.id))
                    .selected_text(issue.status.label())
                    .show_ui(ui, |ui| {
                        for status in [IssueStatus::Open, IssueStatus::InProgress, IssueStatus::Resolved] {
                            ui.selectable_value(&mut issue.status, status, status.label());
                        }
                    });
                ui.end_row();
            }
        });
    });

    ui.collapsing("Người đang làm việc / Presence", |ui| {
        egui::Grid::new("bim_presence").striped(true).show(ui, |ui| {
            ui.strong("Tài khoản"); ui.strong("Tầng"); ui.strong("Bộ môn"); ui.strong("Đang chọn"); ui.end_row();
            for p in &state.presence {
                ui.label(&p.account_id);
                ui.label(state.floors.iter().find(|f| f.id == p.floor_id).map(|f| f.name.as_str()).unwrap_or("?"));
                ui.label(p.discipline.label());
                ui.label(p.selected_handle.map(|h| h.hex()).unwrap_or_else(|| "-".into()));
                ui.end_row();
            }
        });
    });

    ui.collapsing("Nhật ký thay đổi / Audit", |ui| {
        ui.label(format!("Số sự kiện đang giữ: {}", state.audit_log.len()));
        egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
            egui::Grid::new("bim_audit_log").striped(true).show(ui, |ui| {
                ui.strong("#"); ui.strong("Tài khoản"); ui.strong("Hành động"); ui.strong("Tầng"); ui.strong("Bộ môn"); ui.strong("Đối tượng"); ui.strong("Chi tiết"); ui.end_row();
                for e in state.audit_log.iter().rev().take(300) {
                    ui.label(e.sequence.to_string());
                    ui.label(&e.account_id);
                    ui.label(&e.action);
                    ui.label(e.floor_id.and_then(|id| state.floors.iter().find(|f| f.id == id).map(|f| f.name.clone())).unwrap_or_else(|| "-".into()));
                    ui.label(e.discipline.map(Discipline::label).unwrap_or("-"));
                    ui.label(e.handle.map(|h| h.hex()).unwrap_or_else(|| "-".into()));
                    ui.label(&e.detail);
                    ui.end_row();
                }
            });
        });
    });

    ui.collapsing("Chia sẻ thiết kế: ai được xem của ai", |ui| {
        egui::Grid::new("bim_share_grants").striped(true).show(ui, |ui| {
            ui.strong("Người xem"); ui.strong("Chủ dữ liệu"); ui.strong("Tầng"); ui.strong("Bộ môn"); ui.strong("Xem"); ui.strong("Sửa"); ui.end_row();
            for grant in &mut state.share_grants {
                ui.label(&grant.viewer_account_id);
                ui.label(&grant.owner_account_id);
                ui.label(grant.floor_id.and_then(|id| state.floors.iter().find(|f| f.id == id).map(|f| f.name.clone())).unwrap_or_else(|| "Tất cả".into()));
                ui.label(grant.discipline.map(Discipline::label).unwrap_or("Tất cả"));
                ui.checkbox(&mut grant.can_view, "");
                ui.checkbox(&mut grant.can_edit, "");
                ui.end_row();
            }
        });
    });

    ui.collapsing("Rule quyền chi tiết theo tầng / bộ môn", |ui| {
        egui::Grid::new("bim_scope_rules").striped(true).show(ui, |ui| {
            ui.strong("Tài khoản"); ui.strong("Tầng"); ui.strong("Bộ môn"); ui.strong("Xem"); ui.strong("Sửa"); ui.end_row();
            for rule in &mut state.access_rules {
                ui.label(&rule.account_id);
                ui.label(rule.floor_id.and_then(|id| state.floors.iter().find(|f| f.id == id).map(|f| f.name.clone())).unwrap_or_else(|| "Tất cả".into()));
                ui.label(rule.discipline.map(Discipline::label).unwrap_or("Tất cả"));
                ui.checkbox(&mut rule.can_view, "");
                ui.checkbox(&mut rule.can_edit, "");
                ui.end_row();
            }
        });
    });

    ui.collapsing("Quyền tài khoản", |ui| {
        egui::Grid::new("bim_access").striped(true).show(ui, |ui| {
            ui.strong("Tài khoản"); ui.strong("Bộ môn"); ui.strong("Sửa"); ui.strong("Xem bộ môn khác"); ui.strong("Xem tầng khác"); ui.end_row();
            for user in &mut state.users {
                ui.label(&user.display_name);
                ui.label(user.discipline.label());
                ui.checkbox(&mut user.can_edit, "");
                ui.checkbox(&mut user.can_view_other_disciplines, "");
                ui.checkbox(&mut user.can_view_other_floors, "");
                ui.end_row();
            }
        });
    });

    ui.colored_label(
        egui::Color32::YELLOW,
        "Giai đoạn sơ bộ: viewport hiện mới quản lý scope/tầng/bộ môn/opacity. Renderer đa viewport, server sync và khóa đối tượng sẽ được nối vào các cấu trúc này ở bước tiếp theo."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_mode_is_read_only_by_definition() {
        assert_eq!(ViewMode::Reference.label(), "Tham chiếu");
        assert_eq!(ViewMode::Edit.label(), "Chỉnh sửa");
    }

    #[test]
    fn slot_filter_distinguishes_primary_from_reference_scope() {
        let primary = BimEntityMeta { floor_id: 1, discipline: Discipline::Architecture, owner_account_id: "local-owner".into(), revision: 1, entity_handle: Some(cadcraft_doc::Handle(1)) };
        let other = BimEntityMeta { floor_id: 1, discipline: Discipline::Structure, owner_account_id: "struct-01".into(), revision: 1, entity_handle: Some(cadcraft_doc::Handle(2)) };
        assert!(slot_matches(&primary, &State::default().views[0]));
        assert!(!slot_matches(&other, &State::default().views[0]));
    }

    #[test]
    fn reference_view_is_not_editable_even_with_account_permission() {
        let state = State::default();
        assert_eq!(state.views[0].mode, ViewMode::Edit);
        assert_eq!(state.views[1].mode, ViewMode::Reference);
    }

    #[test]
    fn view_preset_is_scoped_per_account_and_restores_layout() {
        let mut state = State::default();
        state.layout = ViewLayout::Four;
        state.views[2].floor_id = 3;
        state.save_view_preset("Quad".into());
        state.layout = ViewLayout::One;
        state.views[2].floor_id = 1;
        assert!(state.apply_view_preset("Quad"));
        assert_eq!(state.layout, ViewLayout::Four);
        assert_eq!(state.views[2].floor_id, 3);
    }

    #[test]
    fn view_layout_maps_to_expected_native_viewport_count() {
        assert_eq!(ViewLayout::One.slots(), 1);
        assert_eq!(ViewLayout::Two.slots(), 2);
        assert_eq!(ViewLayout::Three.slots(), 3);
        assert_eq!(ViewLayout::Four.slots(), 4);
    }

    #[test]
    fn work_session_starts_and_stops_without_losing_scope() {
        let state = State::default();
        assert!(!state.work_session_active);
        assert_eq!(state.active_floor_id, 1);
        assert_eq!(state.active_discipline, Discipline::Architecture);
    }

    #[test]
    fn publish_requires_clean_model_and_authenticated_session() {
        let state = State::default();
        assert_ne!(state.auth.status, AuthStatus::SignedOut);
    }

    #[test]
    fn model_health_reports_non_clean_state_when_pending_or_issue_exists() {
        let mut health = ModelHealth::default();
        assert!(health.is_clean());
        health.pending_changes = 1;
        assert!(!health.is_clean());
        health.pending_changes = 0;
        health.open_issues = 1;
        assert!(!health.is_clean());
    }

    #[test]
    fn object_sync_state_prioritizes_conflict_and_lock() {
        let mut state = State::default();
        let h = cadcraft_doc::Handle(90);
        assert_eq!(state.object_sync_state(h), ObjectSyncState::Clean);
        state.pending_changes.push(ChangeSet {
            id: 1,
            account_id: "local-owner".into(),
            items: vec![ChangeItem { handle: h, operation: ChangeOperation::Update, base_revision: 0, local_revision: 1 }],
            synced: false,
        });
        assert_eq!(state.object_sync_state(h), ObjectSyncState::Local);
        state.connection = ConnectionMode::Online;
        assert_eq!(state.object_sync_state(h), ObjectSyncState::Pending);
        state.conflicts.push(SyncConflict { handle: h, base_revision: 0, local_revision: 1, remote_revision: 2 });
        assert_eq!(state.object_sync_state(h), ObjectSyncState::Conflict);
    }

    #[test]
    fn conflict_resolution_keep_mine_rebases_local_change() {
        let mut state = State::default();
        let h = cadcraft_doc::Handle(77);
        state.pending_changes.push(ChangeSet {
            id: 1,
            account_id: "local-owner".into(),
            items: vec![ChangeItem { handle: h, operation: ChangeOperation::Update, base_revision: 2, local_revision: 3 }],
            synced: false,
        });
        state.conflicts.push(SyncConflict { handle: h, base_revision: 2, local_revision: 3, remote_revision: 5 });
        state.resolve_conflict(h, ConflictResolution::KeepMine);
        let item = &state.pending_changes[0].items[0];
        assert_eq!(item.base_revision, 5);
        assert_eq!(item.local_revision, 6);
        assert!(state.conflicts.is_empty());
    }

    #[test]
    fn conflict_resolution_accept_theirs_marks_local_change_synced() {
        let mut state = State::default();
        let h = cadcraft_doc::Handle(78);
        state.pending_changes.push(ChangeSet {
            id: 1,
            account_id: "local-owner".into(),
            items: vec![ChangeItem { handle: h, operation: ChangeOperation::Update, base_revision: 1, local_revision: 2 }],
            synced: false,
        });
        state.conflicts.push(SyncConflict { handle: h, base_revision: 1, local_revision: 2, remote_revision: 4 });
        state.resolve_conflict(h, ConflictResolution::AcceptTheirs);
        assert!(state.pending_changes[0].synced);
        assert_eq!(state.remote_revision(h), 4);
        assert!(state.conflicts.is_empty());
    }

    #[test]
    fn sync_accepts_matching_base_revision_and_rejects_stale_change() {
        let mut state = State::default();
        let h = cadcraft_doc::Handle(50);
        state.set_remote_revision(h, 2);
        state.pending_changes.push(ChangeSet {
            id: 1,
            account_id: "local-owner".into(),
            items: vec![ChangeItem { handle: h, operation: ChangeOperation::Update, base_revision: 2, local_revision: 3 }],
            synced: false,
        });
        state.dirty_local_changes = 1;
        state.sync_pending_changes();
        assert_eq!(state.remote_revision(h), 3);
        assert!(state.pending_changes[0].synced);
        assert!(state.conflicts.is_empty());

        state.pending_changes.push(ChangeSet {
            id: 2,
            account_id: "local-owner".into(),
            items: vec![ChangeItem { handle: h, operation: ChangeOperation::Update, base_revision: 2, local_revision: 4 }],
            synced: false,
        });
        state.dirty_local_changes = 1;
        state.sync_pending_changes();
        assert_eq!(state.conflicts.len(), 1);
        assert!(!state.pending_changes[1].synced);
    }

    #[test]
    fn bim_workspace_rejects_bad_version_duplicate_floor_and_invalid_view_floor() {
        let mut bad_version=State::default().snapshot_json();
        bad_version["version"]=serde_json::json!(99);
        assert!(State::default().load_snapshot_json(&bad_version).is_err());

        let mut duplicate=State::default().snapshot_json();
        duplicate["floors"]=serde_json::json!([
            {"id":1,"name":"Tầng 1","elevation_m":0.0,"height_m":3.6},
            {"id":1,"name":"Tầng 2","elevation_m":3.6,"height_m":3.6}
        ]);
        assert!(State::default().load_snapshot_json(&duplicate).is_err());

        let mut invalid_view=State::default().snapshot_json();
        invalid_view["views"][0]["floor_id"]=serde_json::json!(999);
        assert!(State::default().load_snapshot_json(&invalid_view).is_err());
    }

    #[test]
    fn bim_workspace_invalid_load_is_transactional() {
        let mut state=State::default();
        state.project_id="KEEP-ME".into();
        state.project_name="Keep current workspace".into();
        state.coordinates.origin_x_m=321.0;
        state.views[1].floor_id=2;
        let before=state.snapshot_json();

        let mut invalid=before.clone();
        invalid["project_id"]=serde_json::json!("MUTATED-BEFORE-FAIL");
        invalid["coordinates"]["origin_x_m"]=serde_json::json!(999.0);
        invalid["floors"][0]["height_m"]=serde_json::json!(0.0);

        assert!(state.load_snapshot_json(&invalid).is_err());
        assert_eq!(state.snapshot_json(),before);
    }

    #[test]
    fn bim_workspace_rejects_invalid_floor_height_and_active_floor() {
        let mut invalid_height=State::default().snapshot_json();
        invalid_height["floors"][0]["height_m"]=serde_json::json!(0.0);
        assert!(State::default().load_snapshot_json(&invalid_height).is_err());

        let mut invalid_active=State::default().snapshot_json();
        invalid_active["active_floor_id"]=serde_json::json!(999);
        assert!(State::default().load_snapshot_json(&invalid_active).is_err());
    }

    #[test]
    fn bim_workspace_snapshot_restores_views_permissions_progress_and_issues() {
        let mut state = State::default();
        state.views[1].floor_id = 2;
        state.views[1].overlay_opacity = 0.22;
        state.create_issue_from_selection(Some(cadcraft_doc::Handle(15)), Discipline::Structure);
        let json = state.snapshot_json();
        let mut restored = State::default();
        restored.load_snapshot_json(&json).unwrap();
        assert_eq!(restored.views[1].floor_id, 2);
        assert!((restored.views[1].overlay_opacity - 0.22).abs() < 1e-6);
        assert_eq!(restored.access_rules.len(), state.access_rules.len());
        assert_eq!(restored.progress.len(), state.progress.len());
        assert_eq!(restored.issues.len(), 1);
        assert_eq!(restored.issues[0].handle, Some(cadcraft_doc::Handle(15)));
    }

    #[test]
    fn bim_workspace_snapshot_restores_project_and_floors() {
        let mut state = State::default();
        state.project_id = "P-100".into();
        state.project_name = "Tower A".into();
        state.coordinates.origin_x_m = 123.5;
        let json = state.snapshot_json();
        let mut restored = State::default();
        restored.load_snapshot_json(&json).unwrap();
        assert_eq!(restored.project_id, "P-100");
        assert_eq!(restored.project_name, "Tower A");
        assert!((restored.coordinates.origin_x_m - 123.5).abs() < 1e-12);
        assert_eq!(restored.floors.len(), state.floors.len());
    }

    #[test]
    fn scope_progress_lookup_returns_matching_floor_and_discipline() {
        let state = State::default();
        let p = state.scope_progress(1, Discipline::Architecture).unwrap();
        assert_eq!(p.percent, 20);
        assert_eq!(p.status, ScopeStatus::InProgress);
        assert!(state.scope_progress(99, Discipline::Architecture).is_none());
    }

    #[test]
    fn project_progress_auto_creates_scope_and_completion_status() {
        let mut state = State::default();
        state.ensure_progress_scope(2, Discipline::Hvac);
        let p = state.progress.iter_mut().find(|p| p.floor_id == 2 && p.discipline == Discipline::Hvac).unwrap();
        p.percent = 100;
        p.status = ScopeStatus::Complete;
        assert_eq!(p.status, ScopeStatus::Complete);
    }

    #[test]
    fn coordination_candidate_keeps_floor_and_disciplines() {
        let c = CoordinationCandidate {
            floor_id: 3,
            a: cadcraft_doc::Handle(1),
            discipline_a: Discipline::Electrical,
            b: cadcraft_doc::Handle(2),
            discipline_b: Discipline::Plumbing,
        };
        assert_eq!(c.floor_id, 3);
        assert_ne!(c.discipline_a, c.discipline_b);
    }

    #[test]
    fn audit_log_records_entity_and_issue_events() {
        let mut state = State::default();
        state.queue_update(cadcraft_doc::Handle(1), 0, 1);
        state.create_issue_from_selection(Some(cadcraft_doc::Handle(1)), Discipline::Structure);
        assert_eq!(state.audit_log.len(), 2);
        assert_eq!(state.audit_log[0].action, "ENTITY_UPDATE_QUEUED");
        assert_eq!(state.audit_log[1].action, "COORDINATION_ISSUE_CREATED");
    }

    #[test]
    fn coordination_issue_is_created_in_active_floor_and_discipline() {
        let mut state = State::default();
        state.active_floor_id = 2;
        state.active_discipline = Discipline::Architecture;
        state.create_issue_from_selection(Some(cadcraft_doc::Handle(12)), Discipline::Structure);
        assert_eq!(state.issues.len(), 1);
        assert_eq!(state.issues[0].floor_id, 2);
        assert_eq!(state.issues[0].source_discipline, Discipline::Architecture);
        assert_eq!(state.issues[0].target_discipline, Discipline::Structure);
    }

    #[test]
    fn share_grant_controls_who_can_view_whose_discipline() {
        let state = State::default();
        assert!(state.has_share_access("local-owner", "struct-01", 1, Discipline::Structure, false));
        assert!(!state.has_share_access("local-owner", "struct-01", 1, Discipline::Structure, true));
        assert!(!state.has_share_access("mep-elec-01", "struct-01", 1, Discipline::Structure, false));
    }

    #[test]
    fn detailed_scope_rule_controls_view_and_edit() {
        let mut state = State::default();
        state.access_rules = vec![
            AccessRule { account_id: "u1".into(), floor_id: Some(2), discipline: Some(Discipline::Electrical), can_view: true, can_edit: false }
        ];
        assert!(state.has_scope_access("u1", 2, Discipline::Electrical, false));
        assert!(!state.has_scope_access("u1", 2, Discipline::Electrical, true));
        assert!(!state.has_scope_access("u1", 1, Discipline::Electrical, false));
        assert!(!state.has_scope_access("u1", 2, Discipline::Structure, false));
    }

    #[test]
    fn offline_and_online_login_switch_connection_mode() {
        let mut state = State::default();
        state.login_online_stub("u1".into(), "User One".into(), "u1@example.com".into());
        assert_eq!(state.connection, ConnectionMode::Online);
        assert_eq!(state.auth.status, AuthStatus::OnlineAuthenticated);
        state.login_offline("u2".into(), "User Two".into());
        assert_eq!(state.connection, ConnectionMode::Offline);
        assert_eq!(state.auth.status, AuthStatus::OfflineProfile);
    }

    #[test]
    fn soft_lock_prevents_another_account_from_editing_same_object() {
        let mut state = State::default();
        let handle = cadcraft_doc::Handle(99);
        state.acquire_lease(handle, 1).unwrap();
        state.current_account_id = "struct-01".into();
        assert!(state.acquire_lease(handle, 1).is_err());
    }

    #[test]
    fn presence_tracks_current_user_scope_and_selection() {
        let mut state = State::default();
        state.active_floor_id = 2;
        state.active_discipline = Discipline::Electrical;
        state.update_presence(Some(cadcraft_doc::Handle(7)));
        assert_eq!(state.presence.len(), 1);
        assert_eq!(state.presence[0].floor_id, 2);
        assert_eq!(state.presence[0].discipline, Discipline::Electrical);
        assert_eq!(state.presence[0].selected_handle, Some(cadcraft_doc::Handle(7)));
    }

    #[test]
    fn viewport_filter_matches_floor_discipline_and_optional_owner() {
        let meta = BimEntityMeta { floor_id: 2, discipline: Discipline::Structure, owner_account_id: "struct-01".into(), revision: 1, entity_handle: Some(cadcraft_doc::Handle(2)) };
        let mut slot = ViewSlot { floor_id: 2, discipline: Discipline::Structure, owner_account_id: None, overlay_opacity: 0.3, show_other_users: true, mode: ViewMode::Reference };
        assert!(slot_matches(&meta, &slot));
        slot.floor_id = 1;
        assert!(!slot_matches(&meta, &slot));
        slot.floor_id = 2;
        slot.owner_account_id = Some("other".into());
        assert!(!slot_matches(&meta, &slot));
    }

    #[test]
    fn revision_mismatch_creates_sync_conflict() {
        let item = ChangeItem { handle: cadcraft_doc::Handle(42), operation: ChangeOperation::Update, base_revision: 4, local_revision: 5 };
        assert!(detect_conflict(&item, 4).is_none());
        let conflict = detect_conflict(&item, 6).unwrap();
        assert_eq!(conflict.base_revision, 4);
        assert_eq!(conflict.local_revision, 5);
        assert_eq!(conflict.remote_revision, 6);
    }

    #[test]
    fn entity_scope_contract_keeps_floor_discipline_and_owner() {
        let scope=EntityScope{floor_id:3,discipline:"ELEC".into(),owner_account_id:"u-elec".into(),revision:7,entity_handle:Some(cadcraft_doc::Handle(0x2A))};
        assert_eq!(scope.floor_id,3);
        assert_eq!(scope.discipline,"ELEC");
        assert_eq!(scope.owner_account_id,"u-elec");
        assert_eq!(scope.revision,7);
        assert_eq!(scope.entity_handle,Some(cadcraft_doc::Handle(0x2A)));
    }

    #[test]
    fn bim_metadata_roundtrips_through_entity_xdata() {
        let mut common = cadcraft_doc::Common::default();
        let meta = BimEntityMeta { floor_id: 7, discipline: Discipline::Electrical, owner_account_id: "user-01".into(), revision: 3, entity_handle: Some(cadcraft_doc::Handle(0x2A)) };
        set_bim_meta(&mut common, &meta);
        assert_eq!(bim_meta(&common), Some(meta));
        assert!(common.xdata.iter().any(|t| t.code == 1001 && t.value == XDATA_APP));
    }

    #[test]
    fn offline_changes_are_queued_and_can_be_marked_synced() {
        let mut state = State::default();
        state.queue_update(cadcraft_doc::Handle(10), 2, 3);
        assert_eq!(state.dirty_local_changes, 1);
        assert_eq!(state.pending_changes.len(), 1);
        assert!(!state.pending_changes[0].synced);
        state.mark_all_synced();
        assert_eq!(state.dirty_local_changes, 0);
        assert!(state.pending_changes[0].synced);
    }

    #[test]
    fn floor_elevation_maps_local_to_world_z() {
        let state = State::default();
        assert_eq!(state.world_position(2, 10.0, 20.0, 0.5), Some([10.0, 20.0, 4.1]));
    }

    #[test]
    fn project_rotation_is_applied_to_world_coordinates() {
        let mut state = State::default();
        state.coordinates.rotation_deg = 90.0;
        let p = state.world_position(1, 10.0, 0.0, 0.0).unwrap();
        assert!(p[0].abs() < 1e-10);
        assert!((p[1] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn view_layout_has_expected_slot_count() {
        assert_eq!(ViewLayout::One.slots(), 1);
        assert_eq!(ViewLayout::Two.slots(), 2);
        assert_eq!(ViewLayout::Three.slots(), 3);
        assert_eq!(ViewLayout::Four.slots(), 4);
    }

    #[test]
    fn owner_can_view_other_disciplines_and_floors_when_granted() {
        let state = State::default();
        let target = state.users.iter().find(|u| u.account_id == "struct-01").unwrap();
        assert!(state.can_view(target, 2, 1));
    }

    #[test]
    fn restricted_user_cannot_cross_discipline_without_grant() {
        let mut state = State::default();
        state.current_account_id = "struct-01".into();
        let target = state.users.iter().find(|u| u.account_id == "mep-elec-01").unwrap();
        assert!(!state.can_view(target, 1, 1));
    }
}
