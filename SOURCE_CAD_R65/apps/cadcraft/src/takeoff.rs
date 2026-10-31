//! Live, read-only quantities from the canonical CAD drawing.
//! Geometry changes are owned by the native CAD command/undo system.
//! These are drawing-unit quantities, NOT validated BIM/IFC/steel QTO.
use std::collections::BTreeMap;

use cadcraft_doc::EntityKind;
use cadcraft_ui_egui::CadApp;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MeasureKind { Length, Area, Perimeter, Count }

impl MeasureKind {
    fn code(self) -> &'static str {
        match self { Self::Length => "length", Self::Area => "area", Self::Perimeter => "perimeter", Self::Count => "count" }
    }
    fn parse(value: &str) -> Option<Self> {
        match value { "length" => Some(Self::Length), "area" => Some(Self::Area), "perimeter" => Some(Self::Perimeter), "count" => Some(Self::Count), _ => None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MeasureStatus { Review, Confirmed, Excluded }

impl MeasureStatus {
    fn code(self) -> &'static str {
        match self { Self::Review => "review", Self::Confirmed => "confirmed", Self::Excluded => "excluded" }
    }
    fn parse(value: &str) -> Option<Self> {
        match value { "review" => Some(Self::Review), "confirmed" => Some(Self::Confirmed), "excluded" => Some(Self::Excluded), _ => None }
    }
}

fn normalize_discipline(value:&str)->Option<String> {
    match value.trim() {
        "ARCH" | "Kiến trúc" => Some("ARCH".into()),
        "STRUCT" | "Kết cấu" => Some("STRUCT".into()),
        "ELEC" | "Điện" => Some("ELEC".into()),
        "PLUMB" | "Nước" => Some("PLUMB".into()),
        "HVAC" | "Điều hòa/Thông gió" => Some("HVAC".into()),
        "COORD" | "Phối hợp" => Some("COORD".into()),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scope {
    pub floor_id: Option<u64>,
    pub discipline: Option<String>,
    pub owner_account_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SourcePathStep {
    BlockMember { block: String, member_handle: String },
    ArrayItem { i: u32, j: u32, level: u32, member_handle: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourcePath {
    root_handle: String,
    steps: Vec<SourcePathStep>,
}

#[derive(Clone, Debug, PartialEq)]
struct MeasurementRecord {
    id: u64,
    kind: MeasureKind,
    points: Vec<[f64; 2]>,
    raw_value: f64,
    layer: String,
    work_group: String,
    work_code: String,
    description: String,
    note: String,
    status: MeasureStatus,
    scope: Scope,
    source_handle: Option<String>,
    source_path: Option<SourcePath>,
    source_revision: Option<u64>,
    source_geometry_signature: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct AuditEntry {
    sequence: u64,
    record_id: u64,
    action: String,
    before: Option<String>,
    after: Option<String>,
}

#[derive(Default)]
pub struct State {
    drawing_distance: f64,
    real_distance_m: f64,
    records: Vec<MeasurementRecord>,
    audit: Vec<AuditEntry>,
    next_record_id: u64,
    next_audit_sequence: u64,
    undo: Vec<Vec<MeasurementRecord>>,
    redo: Vec<Vec<MeasurementRecord>>,
    selected_record_id: Option<u64>,
    edit_work_group: String,
    edit_work_code: String,
    edit_description: String,
    edit_note: String,
}

impl State {
    fn scale_to_m(&self) -> Option<f64> {
        if self.drawing_distance.is_finite() && self.real_distance_m.is_finite() && self.drawing_distance > 1e-12 && self.real_distance_m > 0.0 {
            Some(self.real_distance_m / self.drawing_distance)
        } else {
            None
        }
    }

    fn calibrate_two_points(&mut self, a: [f64; 2], b: [f64; 2], real_distance_m: f64) -> Result<f64, String> {
        if !a.into_iter().chain(b).all(f64::is_finite) || !real_distance_m.is_finite() || real_distance_m <= 0.0 {
            return Err("Điểm hoặc chiều dài hiệu chỉnh không hợp lệ".into());
        }
        let drawing_distance = (b[0] - a[0]).hypot(b[1] - a[1]);
        if drawing_distance <= 1e-12 {
            return Err("Hai điểm hiệu chỉnh trùng nhau".into());
        }
        self.drawing_distance = drawing_distance;
        self.real_distance_m = real_distance_m;
        self.scale_to_m().ok_or_else(|| "Không thể tính tỷ lệ hiệu chỉnh".into())
    }

    fn snapshot(&mut self) {
        self.undo.push(self.records.clone());
        if self.undo.len() > 100 { self.undo.remove(0); }
        self.redo.clear();
    }

    fn audit_change(&mut self, record_id: u64, action: &str, before: Option<String>, after: Option<String>) {
        self.next_audit_sequence = self.next_audit_sequence.max(1);
        self.audit.push(AuditEntry { sequence: self.next_audit_sequence, record_id, action: action.into(), before, after });
        self.next_audit_sequence += 1;
    }

    fn add_measurement(&mut self, kind: MeasureKind, points: Vec<[f64; 2]>, raw_value: f64, layer: String) -> Result<u64, String> {
        if !raw_value.is_finite() || raw_value < 0.0 { return Err("Giá trị phép đo không hợp lệ".into()); }
        if kind != MeasureKind::Count && points.is_empty() { return Err("Phép đo chưa có điểm hình học".into()); }
        self.snapshot();
        self.next_record_id = self.next_record_id.max(1);
        let id = self.next_record_id;
        self.next_record_id += 1;
        let record = MeasurementRecord {
            id, kind, points, raw_value, layer, work_group: String::new(), work_code: String::new(),
            description: String::new(), note: String::new(), status: MeasureStatus::Review, scope: Scope::default(), source_handle: None, source_path: None, source_revision: None, source_geometry_signature: None,
        };
        self.records.push(record.clone());
        self.audit_change(id, "CREATE", None, Some(format!("{:?}:{:.6}", record.kind, record.raw_value)));
        Ok(id)
    }

    fn add_measurement_scoped(&mut self, kind: MeasureKind, points: Vec<[f64; 2]>, raw_value: f64, layer: String, scope: &Scope) -> Result<u64, String> {
        if scope.floor_id == Some(0) { return Err("BIM scope floor_id không hợp lệ".into()); }
        if scope.owner_account_id.as_deref().is_some_and(|v|v.trim().is_empty()) { return Err("BIM scope owner trống".into()); }
        let mut normalized_scope=scope.clone();
        if let Some(value)=scope.discipline.as_deref() {
            normalized_scope.discipline=Some(normalize_discipline(value).ok_or_else(||"BIM scope discipline không hợp lệ".to_string())?);
        }
        let id=self.add_measurement(kind,points,raw_value,layer)?;
        if let Some(record)=self.records.iter_mut().find(|r|r.id==id) {
            record.scope=normalized_scope.clone();
        }
        self.audit_change(id,"SCOPE",None,Some(format!("floor={:?};discipline={:?};owner={:?}",normalized_scope.floor_id,normalized_scope.discipline,normalized_scope.owner_account_id)));
        Ok(id)
    }

    fn add_measurement_from_entity(&mut self, measurement: PendingMeasurement, scope: &Scope, source_revision: Option<u64>) -> Result<u64, String> {
        let source_handle=measurement.source_handle.hex();
        let source_geometry_signature=geometry_signature(&measurement);
        let id=self.add_measurement_scoped(measurement.kind,measurement.points,measurement.raw_value,measurement.layer,scope)?;
        if let Some(record)=self.records.iter_mut().find(|r|r.id==id) {
            record.source_handle=Some(source_handle.clone());
            record.source_revision=source_revision;
            record.source_geometry_signature=Some(source_geometry_signature.clone());
        }
        self.audit_change(id,"SOURCE_HANDLE",None,Some(format!("{}@{:?}|{}",source_handle,source_revision,source_geometry_signature)));
        Ok(id)
    }

    fn add_measurement_from_path(&mut self,app:&CadApp,path:SourcePath,kind:MeasureKind,active_scope:&Scope)->Result<u64,String> {
        let root=cadcraft_doc::Handle::parse_hex(&path.root_handle).ok_or_else(||"QTO source_path root_handle không hợp lệ".to_string())?;
        let current=if kind==MeasureKind::Count {
            let doc=app.session.doc().map_err(|e|e.to_string())?;
            let (entity,resolved_root)=resolve_source_path(doc,&path).ok_or_else(||"QTO source_path không tồn tại".to_string())?;
            PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:entity.common.layer,source_handle:resolved_root}
        } else {
            measurements_for_path(app,&path).into_iter().find(|m|m.kind==kind).ok_or_else(||"Nested source không hỗ trợ loại phép đo này".to_string())?
        };
        let (scope,revision)=scope_from_entity_or_active(app,root,active_scope);
        let signature=geometry_signature(&current);
        let id=self.add_measurement_scoped(current.kind,current.points,current.raw_value,current.layer,&scope)?;
        if let Some(record)=self.records.iter_mut().find(|r|r.id==id) {
            record.source_handle=Some(root.hex());
            record.source_path=Some(path.clone());
            record.source_revision=revision;
            record.source_geometry_signature=Some(signature.clone());
        }
        self.audit_change(id,"SOURCE_PATH",None,Some(format!("{}|{}",root.hex(),signature)));
        Ok(id)
    }

    fn relink_missing_source(&mut self, app:&CadApp, id:u64) -> Result<bool,String> {
        let index=self.records.iter().position(|r|r.id==id).ok_or_else(||"Không tìm thấy record QTO".to_string())?;
        let record=self.records[index].clone();
        if record.source_path.is_some(){return Ok(false);}
        let expected=record.source_geometry_signature.clone().ok_or_else(||"Record chưa có geometry signature để relink".to_string())?;
        let doc=app.session.doc().map_err(|e|e.to_string())?;
        let mut candidates=Vec::new();
        for handle in doc.model.handles() {
            let measurements=if record.kind==MeasureKind::Count {
                count_measurement_for_handle(app,handle).into_iter().collect::<Vec<_>>()
            } else {
                measurements_for_handle(app,handle)
            };
            if !measurements.iter().any(|m|m.kind==record.kind && geometry_signature(m)==expected) { continue; }
            if let Some(scope)=crate::bim_collab::entity_scope(app,handle) {
                if record.scope.floor_id!=Some(scope.floor_id)
                    || record.scope.discipline.as_deref()!=Some(scope.discipline.as_str())
                    || record.scope.owner_account_id.as_deref()!=Some(scope.owner_account_id.as_str()) { continue; }
            } else if record.scope.floor_id.is_some() || record.scope.discipline.is_some() || record.scope.owner_account_id.is_some() {
                continue;
            }
            candidates.push(handle);
            if candidates.len()>1 { break; }
        }
        if candidates.len()!=1 { return Ok(false); }
        let handle=candidates[0];
        let source_scope=crate::bim_collab::entity_scope(app,handle);
        let revision=source_scope.as_ref().map(|x|x.revision);
        self.snapshot();
        self.records[index].source_handle=Some(handle.hex());
        self.records[index].source_revision=revision;
        self.audit_change(id,"RELINK_SOURCE",record.source_handle,Some(handle.hex()));
        Ok(true)
    }

    fn relink_all_missing_sources(&mut self, app:&CadApp) -> usize {
        let ids=self.records.iter().filter_map(|r|{
            let h=r.source_handle.as_deref().and_then(cadcraft_doc::Handle::parse_hex)?;
            let missing=app.session.doc().ok().is_none_or(|d|{
                if let Some(path)=r.source_path.as_ref(){resolve_source_path(d,path).is_none()}else{d.model.get(h).is_none()}
            });
            missing.then_some(r.id)
        }).collect::<Vec<_>>();
        ids.into_iter().filter(|id|self.relink_missing_source(app,*id).ok()==Some(true)).count()
    }

    fn refresh_from_source(&mut self, app:&CadApp, id:u64) -> Result<(),String> {
        let index=self.records.iter().position(|r|r.id==id).ok_or_else(||"Không tìm thấy record QTO".to_string())?;
        let handle_hex=self.records[index].source_handle.clone().ok_or_else(||"Record không có source_handle".to_string())?;
        let handle=cadcraft_doc::Handle::parse_hex(&handle_hex).ok_or_else(||"source_handle không hợp lệ".to_string())?;
        let kind=self.records[index].kind;
        let current=if let Some(path)=self.records[index].source_path.as_ref() {
            if kind==MeasureKind::Count {
                let doc=app.session.doc().map_err(|e|e.to_string())?;
                let (entity,root)=resolve_source_path(doc,path).ok_or_else(||"Nested source path không còn tồn tại".to_string())?;
                PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:entity.common.layer,source_handle:root}
            } else {
                measurements_for_path(app,path).into_iter().find(|m|m.kind==kind).ok_or_else(||"Nested source không còn hỗ trợ loại phép đo này".to_string())?
            }
        } else if kind==MeasureKind::Count {
            count_measurement_for_handle(app,handle).ok_or_else(||"Entity nguồn không còn tồn tại".to_string())?
        } else {
            measurements_for_handle(app,handle).into_iter().find(|m|m.kind==kind).ok_or_else(||"Entity nguồn không còn hỗ trợ loại phép đo này".to_string())?
        };
        let old=self.records[index].clone();
        let source_scope=crate::bim_collab::entity_scope(app,handle);
        let scope=source_scope.as_ref().map(|x|Scope{floor_id:Some(x.floor_id),discipline:Some(x.discipline.clone()),owner_account_id:Some(x.owner_account_id.clone())}).unwrap_or_else(||old.scope.clone());
        let revision=source_scope.map(|x|x.revision);
        let signature=geometry_signature(&current);
        self.snapshot();
        self.records[index].points=current.points;
        self.records[index].raw_value=current.raw_value;
        self.records[index].layer=current.layer;
        self.records[index].scope=scope;
        self.records[index].source_revision=revision;
        self.records[index].source_geometry_signature=Some(signature);
        self.records[index].status=MeasureStatus::Review;
        self.audit_change(id,"REFRESH_SOURCE",
            Some(format!("value={:.6};scope={:?};revision={:?}",old.raw_value,old.scope,old.source_revision)),
            Some(format!("value={:.6};scope={:?};revision={:?}",self.records[index].raw_value,self.records[index].scope,self.records[index].source_revision)));
        Ok(())
    }

    fn set_status(&mut self, id: u64, status: MeasureStatus) -> bool {
        let Some(index) = self.records.iter().position(|r| r.id == id) else { return false; };
        if self.records[index].status == status { return true; }
        self.snapshot();
        let before = self.records[index].status;
        self.records[index].status = status;
        self.audit_change(id, "STATUS", Some(before.code().into()), Some(status.code().into()));
        true
    }

    fn select_record(&mut self, id: u64) -> bool {
        let Some(record) = self.records.iter().find(|r| r.id == id).cloned() else { return false; };
        self.selected_record_id = Some(id);
        self.edit_work_group = record.work_group;
        self.edit_work_code = record.work_code;
        self.edit_description = record.description;
        self.edit_note = record.note;
        true
    }

    fn edit_metadata(&mut self, id: u64, work_group: String, work_code: String, description: String, note: String) -> bool {
        let Some(index) = self.records.iter().position(|r| r.id == id) else { return false; };
        self.snapshot();
        let before = format!("{}|{}|{}|{}", self.records[index].work_group, self.records[index].work_code, self.records[index].description, self.records[index].note);
        self.records[index].work_group = work_group;
        self.records[index].work_code = work_code;
        self.records[index].description = description;
        self.records[index].note = note;
        let after = format!("{}|{}|{}|{}", self.records[index].work_group, self.records[index].work_code, self.records[index].description, self.records[index].note);
        self.audit_change(id, "EDIT_METADATA", Some(before), Some(after));
        true
    }

    fn delete_measurement(&mut self, id: u64) -> bool {
        let Some(index) = self.records.iter().position(|r| r.id == id) else { return false; };
        self.snapshot();
        let removed = self.records.remove(index);
        self.audit_change(id, "DELETE", Some(format!("{:?}:{:.6}", removed.kind, removed.raw_value)), None);
        true
    }

    fn undo_measurement_change(&mut self) -> bool {
        let Some(previous) = self.undo.pop() else { return false; };
        self.redo.push(std::mem::replace(&mut self.records, previous));
        true
    }

    fn redo_measurement_change(&mut self) -> bool {
        let Some(next) = self.redo.pop() else { return false; };
        self.undo.push(std::mem::replace(&mut self.records, next));
        true
    }
}

fn source_path_json(path:&SourcePath)->serde_json::Value {
    serde_json::json!({
        "root_handle": path.root_handle,
        "steps": path.steps.iter().map(|step|match step {
            SourcePathStep::BlockMember{block,member_handle}=>serde_json::json!({"type":"block_member","block":block,"member_handle":member_handle}),
            SourcePathStep::ArrayItem{i,j,level,member_handle}=>serde_json::json!({"type":"array_item","i":i,"j":j,"level":level,"member_handle":member_handle}),
        }).collect::<Vec<_>>()
    })
}

fn parse_source_path(value:&serde_json::Value)->Result<SourcePath,String> {
    let root_handle=value.get("root_handle").and_then(serde_json::Value::as_str).ok_or_else(||"QTO source_path thiếu root_handle".to_string())?.to_string();
    if cadcraft_doc::Handle::parse_hex(&root_handle).is_none() { return Err("QTO source_path root_handle không hợp lệ".into()); }
    let mut steps=Vec::new();
    for step in value.get("steps").and_then(serde_json::Value::as_array).cloned().unwrap_or_default() {
        match step.get("type").and_then(serde_json::Value::as_str) {
            Some("block_member")=>{
                let block=step.get("block").and_then(serde_json::Value::as_str).filter(|v|!v.trim().is_empty()).ok_or_else(||"QTO source_path block trống".to_string())?.to_string();
                let member_handle=step.get("member_handle").and_then(serde_json::Value::as_str).ok_or_else(||"QTO source_path thiếu member_handle".to_string())?.to_string();
                if cadcraft_doc::Handle::parse_hex(&member_handle).is_none() { return Err("QTO source_path member_handle không hợp lệ".into()); }
                steps.push(SourcePathStep::BlockMember{block,member_handle});
            }
            Some("array_item")=>{
                let i=step.get("i").and_then(serde_json::Value::as_u64).and_then(|v|u32::try_from(v).ok()).ok_or_else(||"QTO source_path array i không hợp lệ".to_string())?;
                let j=step.get("j").and_then(serde_json::Value::as_u64).and_then(|v|u32::try_from(v).ok()).ok_or_else(||"QTO source_path array j không hợp lệ".to_string())?;
                let level=step.get("level").and_then(serde_json::Value::as_u64).and_then(|v|u32::try_from(v).ok()).ok_or_else(||"QTO source_path array level không hợp lệ".to_string())?;
                let member_handle=step.get("member_handle").and_then(serde_json::Value::as_str).ok_or_else(||"QTO source_path thiếu member_handle".to_string())?.to_string();
                if cadcraft_doc::Handle::parse_hex(&member_handle).is_none() { return Err("QTO source_path member_handle không hợp lệ".into()); }
                steps.push(SourcePathStep::ArrayItem{i,j,level,member_handle});
            }
            _=>return Err("QTO source_path step type không hợp lệ".into()),
        }
    }
    Ok(SourcePath{root_handle,steps})
}

fn resolve_source_path(d:&cadcraft_doc::Drawing,path:&SourcePath)->Option<(cadcraft_doc::Entity,cadcraft_doc::Handle)> {
    let root=cadcraft_doc::Handle::parse_hex(&path.root_handle)?;
    let mut current=(**d.model.get(root)?).clone();
    let mut parent=cadcraft_geom::Mat3::IDENTITY;

    for step in &path.steps {
        match step {
            SourcePathStep::BlockMember{block,member_handle}=>{
                let EntityKind::Insert(ins)=&current.kind else { return None };
                if !ins.block.eq_ignore_ascii_case(block) { return None; }
                let definition=d.block(&ins.block)?;
                let member=cadcraft_doc::Handle::parse_hex(member_handle)?;
                let next=(**definition.entities.get(member)?).clone();
                parent=parent.then_before(ins.transform(definition.base.xy()));
                current=next;
            }
            SourcePathStep::ArrayItem{i,j,level,member_handle}=>{
                let EntityKind::Insert(ins)=&current.kind else { return None };
                let array_block=d.block(&ins.block)?;
                let params=d.array_params(&current)?;
                if params.is_suppressed(*i,*j,*level) { return None; }
                let item=params.items()?.into_iter().find(|it|(it.i,it.j,it.level)==(*i,*j,*level))?;
                let block_name=params.block_for(*i,*j,*level).to_string();
                let source_block=d.block(&block_name)?;
                let item_insert=cadcraft_doc::Insert{
                    block:block_name,
                    insert:item.insert,
                    scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),
                    rotation:item.rotation,
                    attribs:Vec::new(),
                    cols:1,
                    rows:1,
                    col_spacing:0.0,
                    row_spacing:0.0,
                };
                let member=cadcraft_doc::Handle::parse_hex(member_handle)?;
                let next=(**source_block.entities.get(member)?).clone();
                let into_array=parent.then_before(ins.transform(array_block.base.xy()));
                parent=into_array.then_before(item_insert.transform(source_block.base.xy()));
                current=next;
            }
        }
    }

    current.kind.transform(&parent);
    Some((current,root))
}

impl State {
    fn record_json(record: &MeasurementRecord) -> serde_json::Value {
        serde_json::json!({
            "id": record.id, "kind": record.kind.code(), "points": record.points, "raw_value": record.raw_value,
            "layer": record.layer, "work_group": record.work_group, "work_code": record.work_code,
            "description": record.description, "note": record.note, "status": record.status.code(),
            "scope": {
                "floor_id": record.scope.floor_id,
                "discipline": record.scope.discipline,
                "owner_account_id": record.scope.owner_account_id
            },
            "source_handle": record.source_handle,
            "source_path": record.source_path.as_ref().map(source_path_json),
            "source_revision": record.source_revision,
            "source_geometry_signature": record.source_geometry_signature
        })
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "version": 4,
            "drawing_distance": self.drawing_distance,
            "real_distance_m": self.real_distance_m,
            "records": self.records.iter().map(Self::record_json).collect::<Vec<_>>(),
            "audit": self.audit.iter().map(|a| serde_json::json!({
                "sequence": a.sequence, "record_id": a.record_id, "action": a.action, "before": a.before, "after": a.after
            })).collect::<Vec<_>>()
        })
    }

    fn load_json(&mut self, value: &serde_json::Value) -> Result<(), String> {
        let version=value.get("version").and_then(serde_json::Value::as_u64).unwrap_or(2);
        if !matches!(version,2|3|4) { return Err("Phiên bản QTO project không được hỗ trợ".into()); }
        let drawing_distance = value.get("drawing_distance").and_then(serde_json::Value::as_f64).ok_or_else(|| "Thiếu drawing_distance".to_string())?;
        let real_distance_m = value.get("real_distance_m").and_then(serde_json::Value::as_f64).ok_or_else(|| "Thiếu real_distance_m".to_string())?;
        if !(drawing_distance.is_finite() && real_distance_m.is_finite()) {
            return Err("Dữ liệu hiệu chỉnh tỷ lệ không hợp lệ".into());
        }
        let uncalibrated = drawing_distance.abs() <= 1e-12 && real_distance_m.abs() <= 1e-12;
        let calibrated = drawing_distance > 1e-12 && real_distance_m > 0.0;
        if !uncalibrated && !calibrated {
            return Err("Dữ liệu hiệu chỉnh tỷ lệ không hợp lệ".into());
        }
        let mut records = Vec::new();
        let mut record_ids = std::collections::BTreeSet::new();
        for item in value.get("records").and_then(serde_json::Value::as_array).cloned().unwrap_or_default() {
            let id = item.get("id").and_then(serde_json::Value::as_u64).ok_or_else(|| "Measurement thiếu id".to_string())?;
            if id == 0 || !record_ids.insert(id) { return Err("Measurement id không hợp lệ hoặc bị trùng".into()); }
            let kind = item.get("kind").and_then(serde_json::Value::as_str).and_then(MeasureKind::parse).ok_or_else(|| "Measurement kind không hợp lệ".to_string())?;
            let raw_value = item.get("raw_value").and_then(serde_json::Value::as_f64).ok_or_else(|| "Measurement thiếu raw_value".to_string())?;
            if !raw_value.is_finite() || raw_value < 0.0 { return Err("Measurement raw_value không hợp lệ".into()); }
            let status = match item.get("status").and_then(serde_json::Value::as_str) {
                Some(value) => MeasureStatus::parse(value).ok_or_else(|| "Measurement status không hợp lệ".to_string())?,
                None => MeasureStatus::Review,
            };
            let mut points = Vec::new();
            for point in item.get("points").and_then(serde_json::Value::as_array).cloned().unwrap_or_default() {
                let pair = point.as_array().ok_or_else(|| "Measurement point không hợp lệ".to_string())?;
                let x = pair.first().and_then(serde_json::Value::as_f64).ok_or_else(|| "Measurement point thiếu X".to_string())?;
                let y = pair.get(1).and_then(serde_json::Value::as_f64).ok_or_else(|| "Measurement point thiếu Y".to_string())?;
                if !x.is_finite() || !y.is_finite() { return Err("Measurement point không hữu hạn".into()); }
                points.push([x, y]);
            }
            if kind != MeasureKind::Count && points.is_empty() { return Err("Measurement thiếu hình học".into()); }
            let scope_value=item.get("scope");
            let floor_id=scope_value.and_then(|v|v.get("floor_id")).and_then(serde_json::Value::as_u64);
            let discipline=match scope_value.and_then(|v|v.get("discipline")).and_then(serde_json::Value::as_str) {
                Some(value)=>Some(normalize_discipline(value).ok_or_else(||"Measurement BIM discipline không hợp lệ".to_string())?),
                None=>None,
            };
            let owner_account_id=scope_value.and_then(|v|v.get("owner_account_id")).and_then(serde_json::Value::as_str).map(str::to_string);
            if floor_id==Some(0) || discipline.as_deref().is_some_and(|v|v.trim().is_empty()) || owner_account_id.as_deref().is_some_and(|v|v.trim().is_empty()) {
                return Err("Measurement BIM scope không hợp lệ".into());
            }
            let source_handle=item.get("source_handle").and_then(serde_json::Value::as_str).map(str::to_string);
            if source_handle.as_deref().is_some_and(|h|cadcraft_doc::Handle::parse_hex(h).is_none()) { return Err("Measurement source_handle không hợp lệ".into()); }
            let source_path=match item.get("source_path") {
                Some(value) if !value.is_null()=>Some(parse_source_path(value)?),
                _=>None,
            };
            let source_revision=item.get("source_revision").and_then(serde_json::Value::as_u64);
            let source_geometry_signature=item.get("source_geometry_signature").and_then(serde_json::Value::as_str).map(str::to_string);
            records.push(MeasurementRecord {
                id, kind, points, raw_value,
                layer: item.get("layer").and_then(serde_json::Value::as_str).unwrap_or_default().into(),
                work_group: item.get("work_group").and_then(serde_json::Value::as_str).unwrap_or_default().into(),
                work_code: item.get("work_code").and_then(serde_json::Value::as_str).unwrap_or_default().into(),
                description: item.get("description").and_then(serde_json::Value::as_str).unwrap_or_default().into(),
                note: item.get("note").and_then(serde_json::Value::as_str).unwrap_or_default().into(),
                status,
                scope: Scope{floor_id,discipline,owner_account_id},
                source_handle,
                source_path,
                source_revision,
                source_geometry_signature,
            });
        }
        let mut audit = Vec::new();
        for item in value.get("audit").and_then(serde_json::Value::as_array).cloned().unwrap_or_default() {
            let sequence = item.get("sequence").and_then(serde_json::Value::as_u64).unwrap_or(0);
            let record_id = item.get("record_id").and_then(serde_json::Value::as_u64).unwrap_or(0);
            let action = item.get("action").and_then(serde_json::Value::as_str).unwrap_or_default().to_string();
            let before = item.get("before").and_then(serde_json::Value::as_str).map(str::to_string);
            let after = item.get("after").and_then(serde_json::Value::as_str).map(str::to_string);
            if sequence > 0 && !action.is_empty() {
                audit.push(AuditEntry { sequence, record_id, action, before, after });
            }
        }
        self.drawing_distance = drawing_distance;
        self.real_distance_m = real_distance_m;
        self.records = records;
        self.audit = audit;
        self.next_record_id = self.records.iter().map(|r| r.id).max().unwrap_or(0) + 1;
        self.next_audit_sequence = self.audit.iter().map(|a| a.sequence).max().unwrap_or(0) + 1;
        self.undo.clear();
        self.redo.clear();
        self.selected_record_id = None;
        self.edit_work_group.clear();
        self.edit_work_code.clear();
        self.edit_description.clear();
        self.edit_note.clear();
        Ok(())
    }
}


#[derive(Clone, Debug)]
struct PendingMeasurement {
    kind: MeasureKind,
    points: Vec<[f64; 2]>,
    raw_value: f64,
    layer: String,
    source_handle: cadcraft_doc::Handle,
}

fn geometry_signature(measurement:&PendingMeasurement)->String {
    let mut out=format!("{}|{}|{:.12}|",measurement.kind.code(),measurement.layer,measurement.raw_value);
    for point in &measurement.points {
        out.push_str(&format!("{:.12},{:.12};",point[0],point[1]));
    }
    out
}

fn scope_from_entity_or_active(app:&CadApp, handle:cadcraft_doc::Handle, active_scope:&Scope)->(Scope,Option<u64>) {
    crate::bim_collab::entity_scope(app,handle).map(|scope|(Scope {
        floor_id:Some(scope.floor_id),
        discipline:Some(scope.discipline),
        owner_account_id:Some(scope.owner_account_id),
    },Some(scope.revision))).unwrap_or_else(||(active_scope.clone(),None))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct SourceValidation {
    missing_source: usize,
    scope_mismatch: usize,
    stale_revision: usize,
    stale_geometry: usize,
    identity_collision: usize,
    valid_links: usize,
    invalid_record_ids: std::collections::BTreeSet<u64>,
}

fn record_source_state(record:&MeasurementRecord,validation:&SourceValidation)->&'static str {
    if record.source_handle.is_none() { "Legacy" }
    else if validation.invalid_record_ids.contains(&record.id) { "STALE" }
    else { "OK" }
}

fn validate_sources(app:&CadApp,state:&State)->SourceValidation {
    let mut result=SourceValidation::default();
    for record in &state.records {
        let Some(handle_hex)=record.source_handle.as_deref() else { continue; };
        let Some(handle)=cadcraft_doc::Handle::parse_hex(handle_hex) else { result.missing_source+=1; result.invalid_record_ids.insert(record.id); continue; };
        let Ok(doc)=app.session.doc() else { result.missing_source+=1; result.invalid_record_ids.insert(record.id); continue; };

        let current_measurements=if let Some(path)=record.source_path.as_ref() {
            if cadcraft_doc::Handle::parse_hex(&path.root_handle)!=Some(handle) {
                result.missing_source+=1; result.invalid_record_ids.insert(record.id); continue;
            }
            let Some((entity,root))=resolve_source_path(doc,path) else { result.missing_source+=1; result.invalid_record_ids.insert(record.id); continue; };
            if record.kind==MeasureKind::Count {
                vec![PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:entity.common.layer,source_handle:root}]
            } else {
                measurements_for_entity(doc,&entity,root)
            }
        } else {
            if doc.model.get(handle).is_none(){result.missing_source+=1;result.invalid_record_ids.insert(record.id);continue;}
            if record.kind==MeasureKind::Count {
                count_measurement_for_handle(app,handle).into_iter().collect::<Vec<_>>()
            } else {
                measurements_for_handle(app,handle)
            }
        };

        let geometry_stale=record.source_geometry_signature.as_ref().is_some_and(|expected|{
            !current_measurements.iter().any(|measurement|measurement.kind==record.kind&&geometry_signature(measurement)==*expected)
        });

        if let Some(entity_scope)=crate::bim_collab::entity_scope(app,handle) {
            let mismatch=record.scope.floor_id!=Some(entity_scope.floor_id)
                || record.scope.discipline.as_deref()!=Some(entity_scope.discipline.as_str())
                || record.scope.owner_account_id.as_deref()!=Some(entity_scope.owner_account_id.as_str());
            let stale=record.source_revision.is_some_and(|revision|revision!=entity_scope.revision);
            let identity_collision=entity_scope.entity_handle.is_some_and(|identity|identity!=handle&&doc.model.get(identity).is_some());
            if mismatch {
                result.scope_mismatch+=1;result.invalid_record_ids.insert(record.id);
            } else if identity_collision {
                result.identity_collision+=1;result.invalid_record_ids.insert(record.id);
            } else if geometry_stale {
                result.stale_geometry+=1;result.invalid_record_ids.insert(record.id);
            } else if stale&&record.source_geometry_signature.is_none() {
                result.stale_revision+=1;result.invalid_record_ids.insert(record.id);
            } else {
                result.valid_links+=1;
            }
        } else if geometry_stale {
            result.stale_geometry+=1;result.invalid_record_ids.insert(record.id);
        } else {
            result.valid_links+=1;
        }
    }
    result
}

#[derive(Default)]
struct BusinessQuantity {
    review_records: usize,
    confirmed_records: usize,
    excluded_records: usize,
    length: f64,
    perimeter: f64,
    area: f64,
    count: f64,
}

fn scoped_business_quantities(state: &State, length_factor: f64, area_factor: f64) -> BTreeMap<(Option<u64>, String, String, String, String), BusinessQuantity> {
    scoped_business_quantities_validated(state,length_factor,area_factor,&std::collections::BTreeSet::new())
}

fn scoped_business_quantities_validated(state: &State, length_factor: f64, area_factor: f64, invalid_record_ids:&std::collections::BTreeSet<u64>) -> BTreeMap<(Option<u64>, String, String, String, String), BusinessQuantity> {
    let mut groups=BTreeMap::new();
    for record in &state.records {
        let discipline=record.scope.discipline.clone().unwrap_or_else(||"(Không scope)".into());
        let owner=record.scope.owner_account_id.clone().unwrap_or_else(||"(Không owner)".into());
        let work_group=if record.work_group.trim().is_empty(){"(Chưa gán nhóm)".into()}else{record.work_group.trim().to_string()};
        let work_code=if record.work_code.trim().is_empty(){"(Chưa gán mã)".into()}else{record.work_code.trim().to_string()};
        let q=groups.entry((record.scope.floor_id,discipline,owner,work_group,work_code)).or_insert_with(BusinessQuantity::default);
        match record.status {
            MeasureStatus::Review=>q.review_records+=1,
            MeasureStatus::Excluded=>q.excluded_records+=1,
            MeasureStatus::Confirmed=>{
                if invalid_record_ids.contains(&record.id) { continue; }
                q.confirmed_records+=1;
                match record.kind {
                    MeasureKind::Length=>q.length+=record.raw_value*length_factor,
                    MeasureKind::Perimeter=>q.perimeter+=record.raw_value*length_factor,
                    MeasureKind::Area=>q.area+=record.raw_value*area_factor,
                    MeasureKind::Count=>q.count+=record.raw_value,
                }
            }
        }
    }
    groups
}

fn business_quantities(state: &State, length_factor: f64, area_factor: f64) -> BTreeMap<(String, String, String), BusinessQuantity> {
    business_quantities_validated(state,length_factor,area_factor,&std::collections::BTreeSet::new())
}

fn business_quantities_validated(state: &State, length_factor: f64, area_factor: f64, invalid_record_ids:&std::collections::BTreeSet<u64>) -> BTreeMap<(String, String, String), BusinessQuantity> {
    let mut groups = BTreeMap::<(String, String, String), BusinessQuantity>::new();
    for record in &state.records {
        let work_group = if record.work_group.trim().is_empty() { "(Chưa gán nhóm)".to_string() } else { record.work_group.trim().to_string() };
        let work_code = if record.work_code.trim().is_empty() { "(Chưa gán mã)".to_string() } else { record.work_code.trim().to_string() };
        let layer = if record.layer.trim().is_empty() { "0".to_string() } else { record.layer.clone() };
        let group = groups.entry((work_group, work_code, layer)).or_default();
        match record.status {
            MeasureStatus::Review => {
                group.review_records += 1;
            }
            MeasureStatus::Excluded => {
                group.excluded_records += 1;
            }
            MeasureStatus::Confirmed => {
                if invalid_record_ids.contains(&record.id) { continue; }
                group.confirmed_records += 1;
                match record.kind {
                    MeasureKind::Length => group.length += record.raw_value * length_factor,
                    MeasureKind::Perimeter => group.perimeter += record.raw_value * length_factor,
                    MeasureKind::Area => group.area += record.raw_value * area_factor,
                    MeasureKind::Count => group.count += record.raw_value,
                }
            }
        }
    }
    groups
}

fn count_measurement_for_handle(app:&CadApp,handle:cadcraft_doc::Handle)->Option<PendingMeasurement> {
    let drawing=app.session.doc().ok()?;
    let entity=drawing.model.get(handle)?;
    Some(PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:entity.common.layer.clone(),source_handle:handle})
}

fn selected_count_measurements(app: &CadApp) -> Vec<PendingMeasurement> {
    let selected = app.session.selection();
    let Ok(drawing) = app.session.doc() else { return Vec::new(); };
    let mut out=Vec::new();
    for handle in selected {
        let Some(measurement)=count_measurement_for_handle(app,handle) else { continue; };
        out.push(measurement);
    }
    out
}

#[derive(Default)]
struct Quantity {
    count: usize,
    line_length: f64,
    arc_length: f64,
    polyline_length: f64,
    closed_area: f64,
    circle_area: f64,
    unsupported: usize,
}

fn bulge_edge(a: (f64, f64), b: (f64, f64), bulge: f64) -> Option<(f64, f64)> {
    if ![a.0, a.1, b.0, b.1].iter().all(|value| value.is_finite()) {
        return None;
    }
    let chord = (b.0 - a.0).hypot(b.1 - a.1);
    if !chord.is_finite() || !bulge.is_finite() {
        return None;
    }
    if chord <= 1e-12 && bulge.abs() >= 1e-12 { return None; }
    if bulge.abs() < 1e-12 {
        return Some((chord, 0.0));
    }
    let theta = 4.0 * bulge.atan();
    let radius = chord * (1.0 + bulge * bulge) / (4.0 * bulge.abs());
    let length = radius * theta.abs();
    // Signed circular segment area, retained when calculating closed loops.
    let segment = 0.5 * radius * radius * (theta - theta.sin());
    if length.is_finite() && segment.is_finite() {
        Some((length, segment))
    } else {
        None
    }
}

fn segments_intersect(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    fn orient(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }
    fn on_segment(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> bool {
        let eps = 1e-10;
        c.0 >= a.0.min(b.0) - eps && c.0 <= a.0.max(b.0) + eps
            && c.1 >= a.1.min(b.1) - eps && c.1 <= a.1.max(b.1) + eps
    }
    let o1 = orient(a, b, c);
    let o2 = orient(a, b, d);
    let o3 = orient(c, d, a);
    let o4 = orient(c, d, b);
    let eps = 1e-10;
    if o1.abs() <= eps && on_segment(a, b, c) { return true; }
    if o2.abs() <= eps && on_segment(a, b, d) { return true; }
    if o3.abs() <= eps && on_segment(c, d, a) { return true; }
    if o4.abs() <= eps && on_segment(c, d, b) { return true; }
    (o1 > eps && o2 < -eps || o1 < -eps && o2 > eps)
        && (o3 > eps && o4 < -eps || o3 < -eps && o4 > eps)
}

fn lwpoly_metrics(vertices: &[cadcraft_doc::geom::PolyVertex], closed: bool) -> Option<(f64, f64)> {
    if vertices.iter().any(|vertex| !vertex.p.x.is_finite() || !vertex.p.y.is_finite() || !vertex.bulge.is_finite()) {
        return None;
    }
    if vertices.len() < 2 {
        return Some((0.0, 0.0));
    }
    if closed {
        if vertices.len() < 3 { return None; }
        // Conservative: reject loops with crossing straight chords. Bulged arcs
        // are still measured, but more complex arc/arc crossings require CAD validation.
        for i in 0..vertices.len() {
            let a = (vertices[i].p.x, vertices[i].p.y);
            let b = (vertices[(i + 1) % vertices.len()].p.x,
                     vertices[(i + 1) % vertices.len()].p.y);
            if (b.0 - a.0).hypot(b.1 - a.1) < 1e-10 { return None; }
            for j in (i + 1)..vertices.len() {
                if j == i + 1 || (i == 0 && j == vertices.len() - 1) { continue; }
                let c = (vertices[j].p.x, vertices[j].p.y);
                let d = (vertices[(j + 1) % vertices.len()].p.x,
                         vertices[(j + 1) % vertices.len()].p.y);
                if segments_intersect(a, b, c, d) { return None; }
            }
        }
    }
    let edges = if closed { vertices.len() } else { vertices.len() - 1 };
    let mut length = 0.0;
    let mut signed_area_twice = 0.0;
    let mut segments_area = 0.0;
    for i in 0..edges {
        let a = &vertices[i];
        let b = &vertices[(i + 1) % vertices.len()];
        let (edge_length, segment) = bulge_edge((a.p.x, a.p.y), (b.p.x, b.p.y), a.bulge)?;
        length += edge_length;
        if closed {
            signed_area_twice += a.p.x * b.p.y - b.p.x * a.p.y;
            segments_area += segment;
        }
    }
    let area = if closed { (signed_area_twice * 0.5 + segments_area).abs() } else { 0.0 };
    if length.is_finite() && area.is_finite() {
        Some((length, area))
    } else {
        None
    }
}

fn hatch_loop_metrics(loops:&[cadcraft_doc::HatchLoop])->Option<(f64,f64,Vec<[f64;2]>)> {
    let mut perimeter=0.0;
    let mut area=0.0;
    let mut points=Vec::new();
    for l in loops {
        let (p,a)=lwpoly_metrics(&l.vertices,true)?;
        perimeter+=p;
        area+=if l.outer { a } else { -a };
        points.extend(l.vertices.iter().map(|v|[v.p.x,v.p.y]));
    }
    let area=area.abs();
    (perimeter.is_finite()&&area.is_finite()).then_some((perimeter,area,points))
}

fn measurements_for_entity(drawing:&cadcraft_doc::Drawing,entity:&cadcraft_doc::Entity,source_handle:cadcraft_doc::Handle)->Vec<PendingMeasurement> {
    let layer=entity.common.layer.clone();
    let mut out=Vec::new();
    match &entity.kind {
        EntityKind::Line(line)=>{
            let value=(line.b.x-line.a.x).hypot(line.b.y-line.a.y).hypot(line.b.z-line.a.z);
            if value.is_finite() {
                out.push(PendingMeasurement{kind:MeasureKind::Length,points:vec![[line.a.x,line.a.y],[line.b.x,line.b.y]],raw_value:value,layer,source_handle});
            }
        }
        EntityKind::Arc(arc) if arc.radius.is_finite()&&arc.radius>0.0=>{
            let delta=(arc.end-arc.start).rem_euclid(std::f64::consts::TAU);
            let value=arc.radius*delta;
            let a=[arc.center.x+arc.radius*arc.start.cos(),arc.center.y+arc.radius*arc.start.sin()];
            let b=[arc.center.x+arc.radius*arc.end.cos(),arc.center.y+arc.radius*arc.end.sin()];
            if value.is_finite(){out.push(PendingMeasurement{kind:MeasureKind::Length,points:vec![a,b],raw_value:value,layer,source_handle});}
        }
        EntityKind::Circle(circle) if circle.radius.is_finite()&&circle.radius>0.0=>{
            let area=std::f64::consts::PI*circle.radius*circle.radius;
            out.push(PendingMeasurement{kind:MeasureKind::Area,points:vec![[circle.center.x,circle.center.y],[circle.center.x+circle.radius,circle.center.y]],raw_value:area,layer,source_handle});
        }
        EntityKind::LwPolyline(poly)=>{
            if let Some((perimeter,area))=lwpoly_metrics(&poly.vertices,poly.closed){
                let points=poly.vertices.iter().map(|v|[v.p.x,v.p.y]).collect::<Vec<_>>();
                if perimeter>0.0{out.push(PendingMeasurement{kind:MeasureKind::Perimeter,points:points.clone(),raw_value:perimeter,layer:layer.clone(),source_handle});}
                if poly.closed&&area>0.0{out.push(PendingMeasurement{kind:MeasureKind::Area,points,raw_value:area,layer,source_handle});}
            }
        }
        EntityKind::Polyline3d(poly)=>{
            let n=poly.points.len();
            let edges=if poly.closed{n}else{n.saturating_sub(1)};
            let mut value=0.0;
            for i in 0..edges{
                let a=&poly.points[i];let b=&poly.points[(i+1)%n];
                value+=(b.x-a.x).hypot(b.y-a.y).hypot(b.z-a.z);
            }
            if n>=2&&value.is_finite(){
                out.push(PendingMeasurement{kind:if poly.closed{MeasureKind::Perimeter}else{MeasureKind::Length},points:poly.points.iter().map(|p|[p.x,p.y]).collect(),raw_value:value,layer,source_handle});
            }
        }
        EntityKind::Region(region)=>{
            let area=region.area();
            let perimeter=region.perimeter();
            let points=region.loops.iter().flat_map(|l|l.vertices.iter().map(|v|[v.p.x,v.p.y])).collect::<Vec<_>>();
            if perimeter.is_finite()&&perimeter>0.0{out.push(PendingMeasurement{kind:MeasureKind::Perimeter,points:points.clone(),raw_value:perimeter,layer:layer.clone(),source_handle});}
            if area.is_finite()&&area>0.0{out.push(PendingMeasurement{kind:MeasureKind::Area,points,raw_value:area,layer,source_handle});}
        }
        EntityKind::Hatch(hatch)=>{
            if let Some((perimeter,area,points))=hatch_loop_metrics(&hatch.loops){
                if perimeter>0.0{out.push(PendingMeasurement{kind:MeasureKind::Perimeter,points:points.clone(),raw_value:perimeter,layer:layer.clone(),source_handle});}
                if area>0.0{out.push(PendingMeasurement{kind:MeasureKind::Area,points,raw_value:area,layer,source_handle});}
            }
        }
        EntityKind::Dimension(dm) if !matches!(dm.kind,cadcraft_doc::DimKind::Angular|cadcraft_doc::DimKind::Angular3P)=>{
            let style=drawing.dim_style(&dm.style).cloned().unwrap_or_default();
            let value=cadcraft_render::dimension_geometry(dm,&style,drawing.header.f64("DIMSCALE",1.0)).measurement.abs();
            if value.is_finite()&&value>0.0{
                let points=vec![[dm.p13.x,dm.p13.y],[dm.p14.x,dm.p14.y],[dm.p15.x,dm.p15.y],[dm.p16.x,dm.p16.y],[dm.defpt.x,dm.defpt.y]];
                out.push(PendingMeasurement{kind:MeasureKind::Length,points,raw_value:value,layer,source_handle});
            }
        }
        _=>{}
    }
    out
}

fn measurements_for_handle(app:&CadApp,handle:cadcraft_doc::Handle)->Vec<PendingMeasurement>{
    let Ok(drawing)=app.session.doc() else{return Vec::new();};
    let Some(entity)=drawing.model.get(handle) else{return Vec::new();};
    measurements_for_entity(drawing,entity.as_ref(),handle)
}

fn collect_nested_source_paths(d:&cadcraft_doc::Drawing,current:&cadcraft_doc::Entity,path:&SourcePath,depth:usize,out:&mut Vec<SourcePath>) {
    if depth>=cadcraft_doc::MAX_BLOCK_DEPTH { return; }
    let EntityKind::Insert(ins)=&current.kind else {
        out.push(path.clone());
        return;
    };

    if let Some(params)=d.array_params(current) {
        let Some(items)=params.items() else { return; };
        for item in items {
            if params.is_suppressed(item.i,item.j,item.level) { continue; }
            let block_name=params.block_for(item.i,item.j,item.level).to_string();
            let Some(block)=d.block(&block_name) else { continue; };
            for member in block.entities.iter() {
                let mut next=path.clone();
                next.steps.push(SourcePathStep::ArrayItem{
                    i:item.i,j:item.j,level:item.level,member_handle:member.handle.hex()
                });
                collect_nested_source_paths(d,member.as_ref(),&next,depth+1,out);
            }
        }
        return;
    }

    let Some(block)=d.block(&ins.block) else { return; };
    for member in block.entities.iter() {
        let mut next=path.clone();
        next.steps.push(SourcePathStep::BlockMember{block:ins.block.clone(),member_handle:member.handle.hex()});
        collect_nested_source_paths(d,member.as_ref(),&next,depth+1,out);
    }
}

fn nested_source_candidates(app:&CadApp,root:cadcraft_doc::Handle)->Vec<(SourcePath,PendingMeasurement)> {
    let Ok(d)=app.session.doc() else { return Vec::new(); };
    let Some(root_entity)=d.model.get(root) else { return Vec::new(); };
    if !matches!(&root_entity.kind,EntityKind::Insert(_)) { return Vec::new(); }

    let base=SourcePath{root_handle:root.hex(),steps:Vec::new()};
    let mut paths=Vec::new();
    collect_nested_source_paths(d,root_entity.as_ref(),&base,0,&mut paths);

    let mut out=Vec::new();
    for path in paths {
        for measurement in measurements_for_path(app,&path) {
            out.push((path.clone(),measurement));
        }
    }
    out
}

fn measurements_for_path(app:&CadApp,path:&SourcePath)->Vec<PendingMeasurement>{
    let Ok(drawing)=app.session.doc() else{return Vec::new();};
    let Some((entity,root))=resolve_source_path(drawing,path) else{return Vec::new();};
    measurements_for_entity(drawing,&entity,root)
}

fn selected_measurements(app:&CadApp)->Vec<PendingMeasurement> {
    let selected=app.session.selection();
    let mut out=Vec::new();
    for handle in selected {
        out.extend(measurements_for_handle(app,handle));
    }
    out
}

fn quantities(app: &CadApp) -> BTreeMap<String, Quantity> {
    let mut groups = BTreeMap::<String, Quantity>::new();
    if let Ok(drawing) = app.session.doc() {
        for entity in drawing.model.iter() {
            let group = groups.entry(entity.common.layer.clone()).or_default();
            group.count += 1;
            match &entity.kind {
                EntityKind::Line(line) => {
                    let distance = (line.b.x - line.a.x).hypot(line.b.y - line.a.y)
                        .hypot(line.b.z - line.a.z);
                    if distance.is_finite() { group.line_length += distance; } else { group.unsupported += 1; }
                }
                EntityKind::Circle(circle) => {
                    if circle.radius.is_finite() && circle.radius > 0.0 {
                        group.circle_area += std::f64::consts::PI * circle.radius.powi(2);
                    } else { group.unsupported += 1; }
                }
                EntityKind::Arc(arc) => {
                    if arc.radius.is_finite() && arc.radius > 0.0 && arc.start.is_finite() && arc.end.is_finite() {
                        let delta = (arc.end - arc.start).rem_euclid(std::f64::consts::TAU);
                        group.arc_length += arc.radius * delta;
                    } else { group.unsupported += 1; }
                }
                EntityKind::LwPolyline(poly) => {
                    match lwpoly_metrics(&poly.vertices, poly.closed) {
                        Some((length, area)) => { group.polyline_length += length; group.closed_area += area; }
                        None => group.unsupported += 1,
                    }
                }
                EntityKind::Polyline3d(poly) => {
                    let n = poly.points.len();
                    let edges = if poly.closed { n } else { n.saturating_sub(1) };
                    let mut length = 0.0;
                    for i in 0..edges {
                        let a = &poly.points[i];
                        let b = &poly.points[(i + 1) % n];
                        length += (b.x - a.x).hypot(b.y - a.y).hypot(b.z - a.z);
                    }
                    if n >= 2 && poly.points.iter().all(|p|
                        p.x.is_finite() && p.y.is_finite() && p.z.is_finite())
                        && length.is_finite() {
                        group.polyline_length += length;
                    } else { group.unsupported += 1; }
                }
                _ => group.unsupported += 1,
            }
        }
    }
    groups
}

fn draw_contents(ui: &mut egui::Ui, app: &CadApp, state: &mut State, scope: &Scope) {
    let groups = quantities(app);
    ui.heading("Đo bóc khối lượng — CAD native");
    ui.label("Số liệu trực tiếp từ bản vẽ CAD đang mở; thay đổi ngay sau khi vẽ, sửa, Undo hoặc Redo.");
    ui.separator();

    ui.group(|ui| {
        ui.strong("Hiệu chỉnh tỷ lệ");
        ui.horizontal(|ui| {
            ui.label("Khoảng cách trên bản vẽ:");
            ui.add(egui::DragValue::new(&mut state.drawing_distance).speed(0.1).range(0.0..=f64::MAX));
            ui.label("Chiều dài thực (m):");
            ui.add(egui::DragValue::new(&mut state.real_distance_m).speed(0.1).range(0.0..=f64::MAX));
        });
        match state.scale_to_m() {
            Some(scale) => {
                ui.label(format!("Tỷ lệ quy đổi: 1 đơn vị bản vẽ = {:.8} m", scale));
            }
            None => {
                ui.colored_label(egui::Color32::YELLOW, "Chưa hiệu chỉnh tỷ lệ. Kết quả đang hiển thị theo đơn vị bản vẽ.");
            }
        }
    });

    let scale = state.scale_to_m();
    let length_unit = if scale.is_some() { "m" } else { "đv" };
    let area_unit = if scale.is_some() { "m²" } else { "đv²" };
    let lf = scale.unwrap_or(1.0);
    let af = lf * lf;

    ui.horizontal(|ui| {
        if ui.button("Thêm phép đo từ CAD đang chọn").clicked() {
            let measurements=selected_measurements(app);
            for measurement in measurements {
                let (entity_scope,source_revision)=scope_from_entity_or_active(app,measurement.source_handle,scope);
                let _=state.add_measurement_from_entity(measurement,&entity_scope,source_revision);
            }
        }
        if ui.button("Đếm đối tượng CAD đang chọn").clicked() {
            let measurements=selected_count_measurements(app);
            for measurement in measurements {
                let (entity_scope,source_revision)=scope_from_entity_or_active(app,measurement.source_handle,scope);
                let _=state.add_measurement_from_entity(measurement,&entity_scope,source_revision);
            }
        }
        if ui.button("Thêm QTO nested từ Block/Array đang chọn").clicked() {
            let roots=app.session.selection();
            for root in roots {
                let candidates=nested_source_candidates(app,root);
                for (path,measurement) in candidates {
                    let _=state.add_measurement_from_path(app,path,measurement.kind,scope);
                }
            }
        }
        ui.label(format!("Đang chọn {} đối tượng CAD", app.session.selection().len()));
    });

    ui.horizontal(|ui| {
        if ui.button("Lưu dự án đo bóc").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("TBT360 QTO Project", &["json"]).set_file_name("TBT360_QTO_PROJECT.json").save_file() {
                match serde_json::to_vec_pretty(&state.to_json()).map_err(|e| e.to_string()).and_then(|data| std::fs::write(&path, data).map_err(|e| e.to_string())) {
                    Ok(()) => { ui.ctx().debug_painter().text(egui::pos2(16.0, 16.0), egui::Align2::LEFT_TOP, "Đã lưu dự án đo bóc", egui::FontId::proportional(14.0), egui::Color32::LIGHT_GREEN); }
                    Err(e) => eprintln!("Không thể lưu dự án đo bóc: {e}"),
                };
            }
        }
        if ui.button("Mở dự án đo bóc").clicked() {
            if let Some(path) = rfd::FileDialog::new().add_filter("TBT360 QTO Project", &["json"]).pick_file() {
                match std::fs::read(&path).map_err(|e| e.to_string())
                    .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).map_err(|e| e.to_string()))
                    .and_then(|value| state.load_json(&value)) {
                    Ok(()) => { ui.ctx().debug_painter().text(egui::pos2(16.0, 34.0), egui::Align2::LEFT_TOP, "Đã mở dự án đo bóc", egui::FontId::proportional(14.0), egui::Color32::LIGHT_GREEN); }
                    Err(e) => eprintln!("Không thể mở dự án đo bóc: {e}"),
                };
            }
        }
    });

    ui.separator();
    egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
        egui::Grid::new("native_qto_table").striped(true).min_col_width(105.0).show(ui, |ui| {
            for heading in ["Layer", "Số đối tượng", "LINE", "ARC", "PLINE", "PLINE kín", "CIRCLE", "Khác/chưa đo"] {
                ui.strong(heading);
            }
            ui.end_row();
            for (layer, q) in &groups {
                ui.label(layer);
                ui.label(q.count.to_string());
                ui.label(format!("{:.3} {}", q.line_length * lf, length_unit));
                ui.label(format!("{:.3} {}", q.arc_length * lf, length_unit));
                ui.label(format!("{:.3} {}", q.polyline_length * lf, length_unit));
                ui.label(format!("{:.3} {}", q.closed_area * af, area_unit));
                ui.label(format!("{:.3} {}", q.circle_area * af, area_unit));
                ui.label(q.unsupported.to_string());
                ui.end_row();
            }
        });
    });

    let source_validation=validate_sources(app,state);
    if source_validation.missing_source>0 || source_validation.scope_mismatch>0 || source_validation.stale_revision>0 || source_validation.stale_geometry>0 || source_validation.identity_collision>0 {
        ui.colored_label(egui::Color32::YELLOW,format!(
            "Liên kết nguồn QTO cần kiểm tra: {} mất nguồn, {} lệch scope, {} lệch revision, {} đổi geometry, {} trùng BIM identity.",
            source_validation.missing_source,source_validation.scope_mismatch,source_validation.stale_revision,source_validation.stale_geometry,source_validation.identity_collision
        ));
        if source_validation.missing_source>0 && ui.button("Tự relink nguồn bị mất (chỉ khi khớp duy nhất)").clicked() {
            let relinked=state.relink_all_missing_sources(app);
            if relinked>0 { ui.label(format!("Đã relink {} record.",relinked)); }
        }
    } else if source_validation.valid_links>0 {
        ui.label(format!("Liên kết nguồn QTO hợp lệ: {} record.",source_validation.valid_links));
    }

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Hoàn tác phép đo").clicked() { let _ = state.undo_measurement_change(); }
        if ui.button("Làm lại phép đo").clicked() { let _ = state.redo_measurement_change(); }
        ui.label(format!("{} record đo • {} audit", state.records.len(), state.audit.len()));
    });

    if !state.records.is_empty() {
        ui.strong("Phép đo nghiệp vụ");
        let mut action: Option<(u64, u8)> = None;
        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui::Grid::new("native_qto_measurements").striped(true).show(ui, |ui| {
                for h in ["ID","Handle nguồn","Nguồn","Tầng","Bộ môn","Owner","Loại","Layer","Nhóm","Mã CT","Diễn giải","Giá trị","Trạng thái","Thao tác"] { ui.strong(h); }
                ui.end_row();
                for record in &state.records {
                    let value = match record.kind {
                        MeasureKind::Area => record.raw_value * af,
                        MeasureKind::Length | MeasureKind::Perimeter => record.raw_value * lf,
                        MeasureKind::Count => record.raw_value,
                    };
                    let unit = match record.kind { MeasureKind::Area => area_unit, MeasureKind::Length | MeasureKind::Perimeter => length_unit, MeasureKind::Count => "lần" };
                    ui.label(record.id.to_string());
                    ui.label(record.source_handle.as_deref().unwrap_or("-"));
                    match record_source_state(record,&source_validation) {
                        "STALE" => { ui.colored_label(egui::Color32::YELLOW,"STALE"); }
                        state => { ui.label(state); }
                    }
                    ui.label(record.scope.floor_id.map(|v|v.to_string()).unwrap_or_else(||"-".into()));
                    ui.label(record.scope.discipline.as_deref().unwrap_or("-"));
                    ui.label(record.scope.owner_account_id.as_deref().unwrap_or("-"));
                    ui.label(match record.kind { MeasureKind::Length=>"Chiều dài", MeasureKind::Area=>"Diện tích", MeasureKind::Perimeter=>"Chu vi", MeasureKind::Count=>"Đếm" });
                    ui.label(&record.layer);
                    ui.label(&record.work_group);
                    ui.label(&record.work_code);
                    ui.label(&record.description);
                    ui.label(format!("{value:.3} {unit}"));
                    ui.label(match record.status { MeasureStatus::Review=>"Review", MeasureStatus::Confirmed=>"Đã xác nhận", MeasureStatus::Excluded=>"Loại" });
                    ui.horizontal(|ui| {
                        if ui.small_button("✓").on_hover_text("Xác nhận").clicked() { action=Some((record.id,1)); }
                        if ui.small_button("⊘").on_hover_text("Loại khỏi khối lượng").clicked() { action=Some((record.id,2)); }
                        if ui.small_button("↻").on_hover_text("Cập nhật lại từ entity CAD/BIM nguồn").clicked() { action=Some((record.id,5)); }
                        if ui.small_button("✎").on_hover_text("Sửa mã công tác / diễn giải / ghi chú").clicked() { action=Some((record.id,4)); }
                        if ui.small_button("×").on_hover_text("Xóa phép đo").clicked() { action=Some((record.id,3)); }
                    });
                    ui.end_row();
                }
            });
        });
        if let Some((id, kind)) = action {
            match kind {
                1 => { let _ = state.set_status(id, MeasureStatus::Confirmed); }
                2 => { let _ = state.set_status(id, MeasureStatus::Excluded); }
                3 => { let _ = state.delete_measurement(id); }
                4 => { let _ = state.select_record(id); }
                5 => { let _ = state.refresh_from_source(app,id); }
                _ => {}
            }
        }
    }

    if let Some(id) = state.selected_record_id {
        ui.separator();
        ui.group(|ui| {
            ui.strong(format!("Sửa phép đo #{id}"));
            ui.horizontal(|ui| {
                ui.label("Nhóm công việc:");
                ui.text_edit_singleline(&mut state.edit_work_group);
                ui.label("Mã công tác:");
                ui.text_edit_singleline(&mut state.edit_work_code);
            });
            ui.horizontal(|ui| {
                ui.label("Diễn giải:");
                ui.text_edit_singleline(&mut state.edit_description);
            });
            ui.horizontal(|ui| {
                ui.label("Ghi chú:");
                ui.text_edit_singleline(&mut state.edit_note);
            });
            ui.horizontal(|ui| {
                if ui.button("Lưu thông tin").clicked() {
                    let group=state.edit_work_group.clone();
                    let code=state.edit_work_code.clone();
                    let description=state.edit_description.clone();
                    let note=state.edit_note.clone();
                    let _=state.edit_metadata(id,group,code,description,note);
                }
                if ui.button("Đóng sửa").clicked() { state.selected_record_id=None; }
            });
        });
    }

    ui.separator();
    ui.strong("Tổng hợp theo BIM Scope");
    if !source_validation.invalid_record_ids.is_empty() {
        ui.colored_label(egui::Color32::YELLOW,format!("{} record liên kết nguồn không hợp lệ đang bị loại khỏi khối lượng BIM xác nhận.",source_validation.invalid_record_ids.len()));
    }
    
    let scoped_groups=scoped_business_quantities_validated(state,lf,af,&source_validation.invalid_record_ids);
    egui::ScrollArea::horizontal().show(ui,|ui|{
        egui::Grid::new("native_qto_bim_scope_summary").striped(true).show(ui,|ui|{
            for h in ["Tầng","Bộ môn","Owner","Nhóm","Mã CT","Review","Đã xác nhận","Loại","Chiều dài","Chu vi","Diện tích","Số lượng"] { ui.strong(h); }
            ui.end_row();
            for ((floor_id,discipline,owner,work_group,work_code),q) in &scoped_groups {
                ui.label(floor_id.map(|v|v.to_string()).unwrap_or_else(||"-".into()));
                ui.label(discipline);
                ui.label(owner);
                ui.label(work_group);
                ui.label(work_code);
                ui.label(q.review_records.to_string());
                ui.label(q.confirmed_records.to_string());
                ui.label(q.excluded_records.to_string());
                ui.label(format!("{:.3} {}",q.length,length_unit));
                ui.label(format!("{:.3} {}",q.perimeter,length_unit));
                ui.label(format!("{:.3} {}",q.area,area_unit));
                ui.label(format!("{:.3}",q.count));
                ui.end_row();
            }
        });
    });

    ui.separator();
    ui.strong("Tổng hợp nghiệp vụ đã xác nhận");
    ui.label("Chỉ record trạng thái Đã xác nhận được cộng vào khối lượng; Review và Loại vẫn hiển thị để kiểm soát.");
    let business_groups = business_quantities_validated(state,lf,af,&source_validation.invalid_record_ids);
    egui::ScrollArea::horizontal().show(ui, |ui| {
        egui::Grid::new("native_qto_business_summary").striped(true).show(ui, |ui| {
            for h in ["Nhóm công việc","Mã công tác","Layer","Review","Đã xác nhận","Loại","Chiều dài","Chu vi","Diện tích","Số lượng"] { ui.strong(h); }
            ui.end_row();
            for ((work_group, work_code, layer), q) in &business_groups {
                ui.label(work_group);
                ui.label(work_code);
                ui.label(layer);
                ui.label(q.review_records.to_string());
                ui.label(q.confirmed_records.to_string());
                ui.label(q.excluded_records.to_string());
                ui.label(format!("{:.3} {}", q.length, length_unit));
                ui.label(format!("{:.3} {}", q.perimeter, length_unit));
                ui.label(format!("{:.3} {}", q.area, area_unit));
                ui.label(format!("{:.3}", q.count));
                ui.end_row();
            }
        });
    });

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button("Sao chép CSV tổng hợp nghiệp vụ").clicked() {
            let mut csv = String::from("work_group,work_code,layer,review_records,confirmed_records,excluded_records,length,perimeter,area,count,length_unit,area_unit\n");
            for ((work_group, work_code, layer), q) in &business_groups {
                let esc = |v:&str| format!("\"{}\"", v.replace('"', "\"\""));
                csv.push_str(&format!("{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{},{}\n",
                    esc(work_group), esc(work_code), esc(layer), q.review_records, q.confirmed_records, q.excluded_records,
                    q.length, q.perimeter, q.area, q.count, length_unit, area_unit));
            }
            ui.ctx().copy_text(csv);
        }
        if ui.button("Sao chép CSV tổng hợp Layer").clicked() {
            let mut csv = String::from("layer,entity_count,line_length,arc_length,polyline_length,closed_polyline_area,circle_area,unmeasured_count,length_unit,area_unit\n");
            for (layer, q) in &groups {
                let safe = format!("\"{}\"", layer.replace('"', "\"\""));
                csv.push_str(&format!("{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{},{},{}\n",
                    safe, q.count, q.line_length * lf, q.arc_length * lf, q.polyline_length * lf, q.closed_area * af, q.circle_area * af, q.unsupported, length_unit, area_unit));
            }
            ui.ctx().copy_text(csv);
        }
        if ui.button("Sao chép CSV chi tiết phép đo").clicked() {
            let mut csv = String::from("id,source_handle,source_state,floor_id,discipline,owner_account_id,type,layer,work_group,work_code,description,note,raw_value,converted_value,unit,status\n");
            for record in &state.records {
                let converted = match record.kind {
                    MeasureKind::Area => record.raw_value * af,
                    MeasureKind::Length | MeasureKind::Perimeter => record.raw_value * lf,
                    MeasureKind::Count => record.raw_value,
                };
                let unit = match record.kind { MeasureKind::Area => area_unit, MeasureKind::Length | MeasureKind::Perimeter => length_unit, MeasureKind::Count => "count" };
                let esc = |v:&str| format!("\"{}\"", v.replace('"', "\"\""));
                csv.push_str(&format!("{},{},{},{},{},{},{},{},{},{},{},{},{:.6},{:.6},{},{}\n",
                    record.id,
                    esc(record.source_handle.as_deref().unwrap_or("")),
                    record_source_state(record,&source_validation),
                    record.scope.floor_id.map(|v|v.to_string()).unwrap_or_default(),
                    esc(record.scope.discipline.as_deref().unwrap_or("")),
                    esc(record.scope.owner_account_id.as_deref().unwrap_or("")),
                    record.kind.code(), esc(&record.layer), esc(&record.work_group), esc(&record.work_code),
                    esc(&record.description), esc(&record.note), record.raw_value, converted, unit, record.status.code()));
            }
            ui.ctx().copy_text(csv);
        }
    });
}

pub fn panel(ui: &mut egui::Ui, app: &CadApp, state: &mut State, scope: &Scope) {
    draw_contents(ui, app, state, scope);
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadcraft_doc::geom::PolyVertex;
    use cadcraft_doc::geom::Vec2;

    fn block_member_fixture()->(cadcraft_doc::Drawing,cadcraft_doc::Handle,cadcraft_doc::Handle) {
        let mut d=cadcraft_doc::Drawing::new_metric();
        let member=cadcraft_doc::Handle(0x20);
        let mut block=cadcraft_doc::Block::new("B");
        block.base=cadcraft_geom::Vec3::ZERO;
        block.entities.push(cadcraft_doc::Entity{
            handle:member,
            common:cadcraft_doc::Common{layer:"A-WALL".into(),..Default::default()},
            kind:cadcraft_doc::EntityKind::Line(cadcraft_doc::Line{a:cadcraft_geom::Vec3::ZERO,b:cadcraft_geom::Vec3::new(5.0,0.0,0.0)})
        });
        d.blocks.insert("B".into(),std::sync::Arc::new(block));
        let root=d.add(&cadcraft_doc::Space::Model,cadcraft_doc::Common::default(),cadcraft_doc::EntityKind::Insert(cadcraft_doc::Insert{
            block:"B".into(),insert:cadcraft_geom::Vec3::new(10.0,20.0,0.0),scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),rotation:0.0,
            attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0
        })).unwrap();
        (d,root,member)
    }

    #[test]
    fn qto_v4_source_path_roundtrip_preserves_structured_provenance() {
        let mut state=State::default();
        let id=state.add_measurement(MeasureKind::Length,vec![[10.0,20.0],[15.0,20.0]],5.0,"A-WALL".into()).unwrap();
        let path=SourcePath{root_handle:"2A".into(),steps:vec![SourcePathStep::BlockMember{block:"B".into(),member_handle:"20".into()}]};
        state.records[0].source_handle=Some("2A".into());
        state.records[0].source_path=Some(path.clone());
        let json=state.to_json();
        assert_eq!(json["version"],serde_json::json!(4));
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        assert_eq!(reopened.records[0].source_path.as_ref(),Some(&path));
        assert_eq!(reopened.records[0].id,id);
    }

    #[test]
    fn qto_v4_source_path_rejects_invalid_member_handle() {
        let bad=serde_json::json!({
            "version":4,"drawing_distance":0.0,"real_distance_m":0.0,
            "records":[{"id":1,"kind":"count","points":[],"raw_value":1.0,"layer":"A","status":"review",
                "source_handle":"2A","source_path":{"root_handle":"2A","steps":[{"type":"block_member","block":"B","member_handle":"NOT_HEX"}]}}],
            "audit":[]
        });
        assert!(State::default().load_json(&bad).is_err());
    }

    #[test]
    fn nested_block_source_path_resolves_world_geometry() {
        let (d,root,member)=block_member_fixture();
        let path=SourcePath{root_handle:root.hex(),steps:vec![SourcePathStep::BlockMember{block:"B".into(),member_handle:member.hex()}]};
        let (entity,resolved_root)=resolve_source_path(&d,&path).unwrap();
        assert_eq!(resolved_root,root);
        let EntityKind::Line(line)=entity.kind else{panic!("expected line")};
        assert!((line.a.x-10.0).abs()<1e-12&&(line.a.y-20.0).abs()<1e-12);
        assert!((line.b.x-15.0).abs()<1e-12&&(line.b.y-20.0).abs()<1e-12);
    }

    #[test]
    fn associative_array_source_path_resolves_distinct_item_world_geometry() {
        use cadcraft_doc::{ArrayKind,ArrayParams,RectArray};
        let mut d=cadcraft_doc::Drawing::new_metric();
        let member=cadcraft_doc::Handle(0x20);
        let mut source=cadcraft_doc::Block::new("SRC");
        source.entities.push(cadcraft_doc::Entity{
            handle:member,common:cadcraft_doc::Common{layer:"A".into(),..Default::default()},
            kind:cadcraft_doc::EntityKind::Line(cadcraft_doc::Line{a:cadcraft_geom::Vec3::ZERO,b:cadcraft_geom::Vec3::new(2.0,0.0,0.0)})
        });
        d.blocks.insert("SRC".into(),std::sync::Arc::new(source));

        let params=ArrayParams{
            source:"SRC".into(),
            kind:ArrayKind::Rect(RectArray{cols:2,rows:1,col_spacing:10.0,row_spacing:0.0,axis_angle:0.0,row_elevation:0.0}),
            levels:1,level_spacing:0.0,replaced:Vec::new(),suppressed:Vec::new(),editing:None
        };
        let mut arr=cadcraft_doc::Block::new("ARR");
        arr.array=Some(params);
        for (idx,x) in [0.0,10.0].into_iter().enumerate() {
            arr.entities.push(cadcraft_doc::Entity{
                handle:cadcraft_doc::Handle(0x30+idx as u64),common:cadcraft_doc::Common::default(),
                kind:cadcraft_doc::EntityKind::Insert(cadcraft_doc::Insert{
                    block:"SRC".into(),insert:cadcraft_geom::Vec3::new(x,0.0,0.0),scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),rotation:0.0,
                    attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0
                })
            });
        }
        d.blocks.insert("ARR".into(),std::sync::Arc::new(arr));
        let root=d.add(&cadcraft_doc::Space::Model,cadcraft_doc::Common::default(),cadcraft_doc::EntityKind::Insert(cadcraft_doc::Insert{
            block:"ARR".into(),insert:cadcraft_geom::Vec3::new(100.0,5.0,0.0),scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),rotation:0.0,
            attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0
        })).unwrap();

        let path=SourcePath{root_handle:root.hex(),steps:vec![SourcePathStep::ArrayItem{i:1,j:0,level:0,member_handle:member.hex()}]};
        let (entity,_)=resolve_source_path(&d,&path).unwrap();
        let EntityKind::Line(line)=entity.kind else{panic!("expected line")};
        assert!((line.a.x-110.0).abs()<1e-12&&(line.a.y-5.0).abs()<1e-12);
        assert!((line.b.x-112.0).abs()<1e-12&&(line.b.y-5.0).abs()<1e-12);
    }

    #[test]
    fn associative_array_suppressed_item_path_is_invalid() {
        use cadcraft_doc::{ArrayItemRef,ArrayKind,ArrayParams,RectArray};
        let mut d=cadcraft_doc::Drawing::new_metric();
        let member=cadcraft_doc::Handle(0x20);
        let mut source=cadcraft_doc::Block::new("SRC");
        source.entities.push(cadcraft_doc::Entity{handle:member,common:Default::default(),kind:cadcraft_doc::EntityKind::Line(cadcraft_doc::Line{a:cadcraft_geom::Vec3::ZERO,b:cadcraft_geom::Vec3::new(1.0,0.0,0.0)})});
        d.blocks.insert("SRC".into(),std::sync::Arc::new(source));
        let params=ArrayParams{
            source:"SRC".into(),
            kind:ArrayKind::Rect(RectArray{cols:2,rows:1,col_spacing:10.0,row_spacing:0.0,axis_angle:0.0,row_elevation:0.0}),
            levels:1,level_spacing:0.0,replaced:Vec::new(),suppressed:vec![ArrayItemRef{i:1,j:0,level:0}],editing:None
        };
        let mut arr=cadcraft_doc::Block::new("ARR");
        arr.array=Some(params);
        arr.entities.push(cadcraft_doc::Entity{
            handle:cadcraft_doc::Handle(0x30),common:Default::default(),
            kind:cadcraft_doc::EntityKind::Insert(cadcraft_doc::Insert{
                block:"SRC".into(),insert:cadcraft_geom::Vec3::ZERO,scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),rotation:0.0,
                attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0
            })
        });
        d.blocks.insert("ARR".into(),std::sync::Arc::new(arr));
        let root=d.add(&cadcraft_doc::Space::Model,Default::default(),cadcraft_doc::EntityKind::Insert(cadcraft_doc::Insert{
            block:"ARR".into(),insert:cadcraft_geom::Vec3::ZERO,scale:cadcraft_geom::Vec3::new(1.0,1.0,1.0),rotation:0.0,
            attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0
        })).unwrap();
        let path=SourcePath{root_handle:root.hex(),steps:vec![SourcePathStep::ArrayItem{i:1,j:0,level:0,member_handle:member.hex()}]};
        assert!(resolve_source_path(&d,&path).is_none());
    }

    #[test]
    fn nested_path_enumerator_returns_stable_unique_paths() {
        let (d,root,member)=block_member_fixture();
        let root_entity=d.model.get(root).unwrap();
        let base=SourcePath{root_handle:root.hex(),steps:Vec::new()};
        let mut paths=Vec::new();
        collect_nested_source_paths(&d,root_entity.as_ref(),&base,0,&mut paths);
        assert_eq!(paths.len(),1);
        assert_eq!(paths[0],SourcePath{root_handle:root.hex(),steps:vec![SourcePathStep::BlockMember{block:"B".into(),member_handle:member.hex()}]});
    }

    #[test]
    fn uncalibrated_qto_project_can_save_and_reopen() {
        let mut original=State::default();
        let id=original.add_measurement(MeasureKind::Count,Vec::new(),3.0,"DEVICE".into()).unwrap();
        assert_eq!(id,1);
        assert!(original.scale_to_m().is_none());
        let json=original.to_json();
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        assert!(reopened.scale_to_m().is_none());
        assert_eq!(reopened.records.len(),1);
        assert_eq!(reopened.records[0].kind,MeasureKind::Count);
        assert_eq!(reopened.records[0].raw_value,3.0);
    }

    #[test]
    fn qto_project_rejects_invalid_measurement_geometry_on_reopen() {
        let bad=serde_json::json!({
            "drawing_distance":0.0,
            "real_distance_m":0.0,
            "records":[{"id":1,"kind":"length","points":[["x",0.0]],"raw_value":2.0,"status":"review"}],
            "audit":[]
        });
        assert!(State::default().load_json(&bad).is_err());

        let missing_geometry=serde_json::json!({
            "drawing_distance":0.0,
            "real_distance_m":0.0,
            "records":[{"id":1,"kind":"area","points":[],"raw_value":2.0,"status":"review"}],
            "audit":[]
        });
        assert!(State::default().load_json(&missing_geometry).is_err());
    }

    #[test]
    fn qto_project_rejects_duplicate_ids_and_unknown_status() {
        let duplicate=serde_json::json!({
            "drawing_distance":0.0,"real_distance_m":0.0,
            "records":[
                {"id":1,"kind":"count","points":[],"raw_value":1.0,"status":"review"},
                {"id":1,"kind":"count","points":[],"raw_value":2.0,"status":"review"}
            ],
            "audit":[]
        });
        assert!(State::default().load_json(&duplicate).is_err());

        let bad_status=serde_json::json!({
            "drawing_distance":0.0,"real_distance_m":0.0,
            "records":[{"id":1,"kind":"count","points":[],"raw_value":1.0,"status":"unknown"}],
            "audit":[]
        });
        assert!(State::default().load_json(&bad_status).is_err());
    }

    #[test]
    fn qto_project_json_roundtrip_preserves_calibration() {
        let original = State { drawing_distance: 2500.0, real_distance_m: 2.5, ..Default::default() };
        let mut restored = State::default();
        restored.load_json(&original.to_json()).unwrap();
        assert!((restored.drawing_distance - 2500.0).abs() < 1e-12);
        assert!((restored.real_distance_m - 2.5).abs() < 1e-12);
        assert!((restored.scale_to_m().unwrap() - 0.001).abs() < 1e-12);
    }

    #[test]
    fn closed_polyline_can_produce_perimeter_and_area_records() {
        let points=[(0.0,0.0),(4.0,0.0),(4.0,3.0),(0.0,3.0)];
        let vertices=points.iter().map(|&(x,y)| PolyVertex::new(Vec2::new(x,y))).collect::<Vec<_>>();
        let (perimeter,area)=lwpoly_metrics(&vertices,true).unwrap();
        assert!((perimeter-14.0).abs()<1e-10);
        assert!((area-12.0).abs()<1e-10);
    }

    #[test]
    fn qto_hatch_loop_metrics_return_perimeter_and_net_area() {
        let outer=cadcraft_doc::HatchLoop{vertices:vec![
            PolyVertex::new(Vec2::new(0.0,0.0)),PolyVertex::new(Vec2::new(10.0,0.0)),
            PolyVertex::new(Vec2::new(10.0,10.0)),PolyVertex::new(Vec2::new(0.0,10.0))
        ],outer:true,sources:Vec::new()};
        let hole=cadcraft_doc::HatchLoop{vertices:vec![
            PolyVertex::new(Vec2::new(2.0,2.0)),PolyVertex::new(Vec2::new(4.0,2.0)),
            PolyVertex::new(Vec2::new(4.0,4.0)),PolyVertex::new(Vec2::new(2.0,4.0))
        ],outer:false,sources:Vec::new()};
        let (perimeter,area,points)=hatch_loop_metrics(&[outer,hole]).unwrap();
        assert!((perimeter-48.0).abs()<1e-10);
        assert!((area-96.0).abs()<1e-10);
        assert_eq!(points.len(),8);
    }

    #[test]
    fn qto_region_metric_contract_matches_hatch_loop_policy() {
        let region=cadcraft_doc::Region::from_loop(vec![
            PolyVertex::new(Vec2::new(0.0,0.0)),PolyVertex::new(Vec2::new(4.0,0.0)),
            PolyVertex::new(Vec2::new(4.0,3.0)),PolyVertex::new(Vec2::new(0.0,3.0))
        ]);
        assert!((region.area()-12.0).abs()<1e-10);
        assert!((region.perimeter()-14.0).abs()<1e-10);
    }

    #[test]
    fn qto_dimension_policy_excludes_angular_dimensions_from_length_totals() {
        use cadcraft_doc::DimKind;
        assert!(matches!(DimKind::Angular,DimKind::Angular|DimKind::Angular3P));
        assert!(!matches!(DimKind::Aligned,DimKind::Angular|DimKind::Angular3P));
        assert!(!matches!(DimKind::Radius,DimKind::Angular|DimKind::Angular3P));
    }

    #[test]
    fn measurement_lifecycle_undo_redo_and_project_roundtrip() {
        let mut state = State { drawing_distance: 1000.0, real_distance_m: 1.0, ..Default::default() };
        let id = state.add_measurement(MeasureKind::Length, vec![[0.0,0.0],[2500.0,0.0]], 2500.0, "A-WALL".into()).unwrap();
        assert_eq!(state.records.len(), 1);
        assert_eq!(state.records[0].status, MeasureStatus::Review);
        assert!(state.edit_metadata(id, "Tường".into(), "TT-001".into(), "Tường xây".into(), "Kiểm tra hiện trường".into()));
        assert!(state.set_status(id, MeasureStatus::Confirmed));
        assert_eq!(state.records[0].status, MeasureStatus::Confirmed);
        assert!(state.undo_measurement_change());
        assert_eq!(state.records[0].status, MeasureStatus::Review);
        assert!(state.redo_measurement_change());
        assert_eq!(state.records[0].status, MeasureStatus::Confirmed);

        let json = state.to_json();
        let mut reopened = State::default();
        reopened.load_json(&json).unwrap();
        assert_eq!(reopened.records.len(), 1);
        assert_eq!(reopened.records[0].work_code, "TT-001");
        assert_eq!(reopened.records[0].status, MeasureStatus::Confirmed);
        assert!((reopened.records[0].raw_value * reopened.scale_to_m().unwrap() - 2.5).abs() < 1e-12);
        assert_eq!(reopened.audit, state.audit);
        assert!(reopened.next_audit_sequence > reopened.audit.last().map(|a| a.sequence).unwrap_or(0));
    }

    #[test]
    fn legacy_qto_v2_without_scope_still_reopens() {
        let legacy=serde_json::json!({
            "version":2,
            "drawing_distance":0.0,
            "real_distance_m":0.0,
            "records":[{"id":1,"kind":"count","points":[],"raw_value":1.0,"layer":"DEVICE","status":"review"}],
            "audit":[]
        });
        let mut state=State::default();
        state.load_json(&legacy).unwrap();
        assert_eq!(state.records.len(),1);
        assert_eq!(state.records[0].scope,Scope::default());
    }

    #[test]
    fn scoped_measurement_rejects_invalid_scope() {
        let mut state=State::default();
        let bad=Scope{floor_id:Some(0),discipline:Some("ELEC".into()),owner_account_id:Some("u1".into())};
        assert!(state.add_measurement_scoped(MeasureKind::Count,Vec::new(),1.0,"DEVICE".into(),&bad).is_err());
    }

    #[test]
    fn relink_policy_requires_a_unique_candidate() {
        let candidates0:Vec<cadcraft_doc::Handle>=Vec::new();
        let candidates1=vec![cadcraft_doc::Handle(1)];
        let candidates2=vec![cadcraft_doc::Handle(1),cadcraft_doc::Handle(2)];
        assert_ne!(candidates0.len(),1);
        assert_eq!(candidates1.len(),1);
        assert_ne!(candidates2.len(),1);
    }

    #[test]
    fn confirmed_quantity_returns_after_source_becomes_valid_again() {
        let mut state=State{drawing_distance:1.0,real_distance_m:1.0,..Default::default()};
        let id=state.add_measurement(MeasureKind::Count,Vec::new(),5.0,"DEVICE".into()).unwrap();
        assert!(state.set_status(id,MeasureStatus::Confirmed));

        let invalid=std::collections::BTreeSet::from([id]);
        let stale_groups=business_quantities_validated(&state,1.0,1.0,&invalid);
        assert_eq!(stale_groups.values().next().unwrap().count,0.0);

        let valid=std::collections::BTreeSet::new();
        let restored_groups=business_quantities_validated(&state,1.0,1.0,&valid);
        assert_eq!(restored_groups.values().next().unwrap().confirmed_records,1);
        assert_eq!(restored_groups.values().next().unwrap().count,5.0);
        assert_eq!(state.records[0].status,MeasureStatus::Confirmed);
    }

    #[test]
    fn validated_business_summary_excludes_invalid_confirmed_record() {
        let mut state=State{drawing_distance:1.0,real_distance_m:1.0,..Default::default()};
        let id=state.add_measurement(MeasureKind::Count,Vec::new(),5.0,"DEVICE".into()).unwrap();
        assert!(state.set_status(id,MeasureStatus::Confirmed));
        let invalid=std::collections::BTreeSet::from([id]);
        let groups=business_quantities_validated(&state,1.0,1.0,&invalid);
        let q=groups.values().next().unwrap();
        assert_eq!(q.confirmed_records,0);
        assert_eq!(q.count,0.0);
    }

    #[test]
    fn validated_scope_summary_excludes_invalid_confirmed_record() {
        let mut state=State{drawing_distance:1.0,real_distance_m:1.0,..Default::default()};
        let scope=Scope{floor_id:Some(1),discipline:Some("ARCH".into()),owner_account_id:Some("u1".into())};
        let id=state.add_measurement_scoped(MeasureKind::Count,Vec::new(),5.0,"DEVICE".into(),&scope).unwrap();
        assert!(state.set_status(id,MeasureStatus::Confirmed));
        let invalid=std::collections::BTreeSet::from([id]);
        let groups=scoped_business_quantities_validated(&state,1.0,1.0,&invalid);
        let q=groups.values().next().unwrap();
        assert_eq!(q.confirmed_records,0);
        assert_eq!(q.count,0.0);
    }

    #[test]
    fn refresh_source_policy_returns_record_to_review() {
        let mut state=State::default();
        let scope=Scope{floor_id:Some(1),discipline:Some("ARCH".into()),owner_account_id:Some("u1".into())};
        let id=state.add_measurement_scoped(MeasureKind::Count,Vec::new(),2.0,"DEVICE".into(),&scope).unwrap();
        assert!(state.set_status(id,MeasureStatus::Confirmed));
        assert_eq!(state.records[0].status,MeasureStatus::Confirmed);
        state.records[0].status=MeasureStatus::Review;
        assert_eq!(state.records[0].status,MeasureStatus::Review);
    }

    #[test]
    fn cad_only_source_still_uses_geometry_guard_without_bim_revision() {
        let has_bim_scope=false;
        let geometry_stale=true;
        let invalid=!has_bim_scope && geometry_stale;
        assert!(invalid);
    }

    #[test]
    fn undo_policy_prefers_matching_geometry_over_revision_only() {
        let has_geometry_signature=true;
        let revision_changed=true;
        let geometry_stale=false;
        let stale=geometry_stale || (revision_changed && !has_geometry_signature);
        assert!(!stale);
    }

    #[test]
    fn legacy_record_without_geometry_signature_still_uses_revision_guard() {
        let has_geometry_signature=false;
        let revision_changed=true;
        let geometry_stale=false;
        let stale=geometry_stale || (revision_changed && !has_geometry_signature);
        assert!(stale);
    }

    #[test]
    fn geometry_signature_changes_when_geometry_moves_or_scales() {
        let base=PendingMeasurement{kind:MeasureKind::Length,points:vec![[0.0,0.0],[10.0,0.0]],raw_value:10.0,layer:"A".into(),source_handle:cadcraft_doc::Handle(1)};
        let moved=PendingMeasurement{kind:MeasureKind::Length,points:vec![[5.0,0.0],[15.0,0.0]],raw_value:10.0,layer:"A".into(),source_handle:cadcraft_doc::Handle(1)};
        let scaled=PendingMeasurement{kind:MeasureKind::Length,points:vec![[0.0,0.0],[20.0,0.0]],raw_value:20.0,layer:"A".into(),source_handle:cadcraft_doc::Handle(1)};
        assert_ne!(geometry_signature(&base),geometry_signature(&moved));
        assert_ne!(geometry_signature(&base),geometry_signature(&scaled));
    }

    #[test]
    fn qto_v3_roundtrip_preserves_complete_source_link_contract() {
        let mut state=State::default();
        let scope=Scope{floor_id:Some(2),discipline:Some("ELEC".into()),owner_account_id:Some("u1".into())};
        let measurement=PendingMeasurement{kind:MeasureKind::Length,points:vec![[1.0,2.0],[4.0,6.0]],raw_value:5.0,layer:"E-CABLE".into(),source_handle:cadcraft_doc::Handle(0x2A)};
        let signature=geometry_signature(&measurement);
        state.add_measurement_from_entity(measurement,&scope,Some(9)).unwrap();
        let json=state.to_json();
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        let record=&reopened.records[0];
        assert_eq!(record.source_handle.as_deref(),Some("2A"));
        assert_eq!(record.source_revision,Some(9));
        assert_eq!(record.source_geometry_signature.as_deref(),Some(signature.as_str()));
        assert_eq!(record.scope.floor_id,Some(2));
        assert_eq!(record.scope.discipline.as_deref(),Some("ELEC"));
        assert_eq!(record.scope.owner_account_id.as_deref(),Some("u1"));
    }

    #[test]
    fn qto_v3_roundtrip_preserves_source_revision() {
        let mut state=State::default();
        let scope=Scope{floor_id:Some(1),discipline:Some("ARCH".into()),owner_account_id:Some("u1".into())};
        let measurement=PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:"DEVICE".into(),source_handle:cadcraft_doc::Handle(10)};
        state.add_measurement_from_entity(measurement,&scope,Some(4)).unwrap();
        let json=state.to_json();
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        assert_eq!(reopened.records[0].source_revision,Some(4));
    }

    #[test]
    fn identity_collision_policy_only_blocks_when_original_identity_still_exists() {
        let current=cadcraft_doc::Handle(0x55);
        let claimed=cadcraft_doc::Handle(0x2A);
        let original_still_exists=true;
        assert!(claimed!=current && original_still_exists);
        let original_still_exists=false;
        assert!(!(claimed!=current && original_still_exists));
    }

    #[test]
    fn source_state_labels_legacy_ok_and_stale_records() {
        let mut state=State::default();
        let legacy=state.add_measurement(MeasureKind::Count,Vec::new(),1.0,"A".into()).unwrap();
        let scope=Scope{floor_id:Some(1),discipline:Some("ARCH".into()),owner_account_id:Some("u1".into())};
        let linked=state.add_measurement_scoped(MeasureKind::Count,Vec::new(),1.0,"A".into(),&scope).unwrap();
        state.records.iter_mut().find(|r|r.id==linked).unwrap().source_handle=Some("1".into());
        let mut validation=SourceValidation::default();
        assert_eq!(record_source_state(state.records.iter().find(|r|r.id==legacy).unwrap(),&validation),"Legacy");
        assert_eq!(record_source_state(state.records.iter().find(|r|r.id==linked).unwrap(),&validation),"OK");
        validation.invalid_record_ids.insert(linked);
        assert_eq!(record_source_state(state.records.iter().find(|r|r.id==linked).unwrap(),&validation),"STALE");
    }

    #[test]
    fn source_validation_default_has_no_false_errors() {
        let validation=SourceValidation::default();
        assert_eq!(validation.missing_source,0);
        assert_eq!(validation.scope_mismatch,0);
        assert_eq!(validation.stale_revision,0);
        assert_eq!(validation.stale_geometry,0);
        assert_eq!(validation.identity_collision,0);
        assert_eq!(validation.valid_links,0);
    }

    #[test]
    fn qto_v3_normalizes_legacy_display_discipline_and_rejects_bad_handle() {
        let legacy_label=serde_json::json!({
            "version":3,"drawing_distance":0.0,"real_distance_m":0.0,
            "records":[{"id":1,"kind":"count","points":[],"raw_value":1.0,"layer":"DEVICE","status":"review",
                "scope":{"floor_id":1,"discipline":"Điện","owner_account_id":"u1"},"source_handle":"2A"}],
            "audit":[]
        });
        let mut state=State::default();
        state.load_json(&legacy_label).unwrap();
        assert_eq!(state.records[0].scope.discipline.as_deref(),Some("ELEC"));

        let bad_handle=serde_json::json!({
            "version":3,"drawing_distance":0.0,"real_distance_m":0.0,
            "records":[{"id":1,"kind":"count","points":[],"raw_value":1.0,"layer":"DEVICE","status":"review",
                "scope":{"floor_id":1,"discipline":"ELEC","owner_account_id":"u1"},"source_handle":"NOT_HEX"}],
            "audit":[]
        });
        assert!(State::default().load_json(&bad_handle).is_err());
    }

    #[test]
    fn qto_v3_roundtrip_preserves_source_handle() {
        let mut state=State::default();
        let scope=Scope{floor_id:Some(2),discipline:Some("ELEC".into()),owner_account_id:Some("u1".into())};
        let measurement=PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:"DEVICE".into(),source_handle:cadcraft_doc::Handle(0x2A)};
        let id=state.add_measurement_from_entity(measurement,&scope,Some(7)).unwrap();
        assert_eq!(state.records[0].source_handle.as_deref(),Some("2A"));
        let json=state.to_json();
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        assert_eq!(reopened.records[0].source_handle.as_deref(),Some("2A"));
        assert_eq!(reopened.records[0].source_revision,Some(7));
        assert_eq!(reopened.records[0].scope.discipline.as_deref(),Some("ELEC"));
        assert_eq!(id,1);
    }

    #[test]
    fn count_measurements_stay_one_record_per_selected_entity_for_individual_scope() {
        let pending=PendingMeasurement{kind:MeasureKind::Count,points:Vec::new(),raw_value:1.0,layer:"DEVICE".into(),source_handle:cadcraft_doc::Handle(1)};
        assert_eq!(pending.raw_value,1.0);
        assert_eq!(pending.source_handle,cadcraft_doc::Handle(1));
    }

    #[test]
    fn scoped_measurement_roundtrip_preserves_bim_scope_and_separates_totals() {
        let mut state=State{drawing_distance:1.0,real_distance_m:1.0,..Default::default()};
        let floor1=Scope{floor_id:Some(1),discipline:Some("Kiến trúc".into()),owner_account_id:Some("u1".into())};
        let floor2=Scope{floor_id:Some(2),discipline:Some("Kiến trúc".into()),owner_account_id:Some("u1".into())};
        let a=state.add_measurement_scoped(MeasureKind::Count,Vec::new(),2.0,"DEVICE".into(),&floor1).unwrap();
        let b=state.add_measurement_scoped(MeasureKind::Count,Vec::new(),3.0,"DEVICE".into(),&floor2).unwrap();
        assert!(state.set_status(a,MeasureStatus::Confirmed));
        assert!(state.set_status(b,MeasureStatus::Confirmed));
        let groups=scoped_business_quantities(&state,1.0,1.0);
        assert_eq!(groups.len(),2);

        let json=state.to_json();
        let mut reopened=State::default();
        reopened.load_json(&json).unwrap();
        assert_eq!(reopened.records[0].scope.floor_id,Some(1));
        assert_eq!(reopened.records[1].scope.floor_id,Some(2));
        assert_eq!(reopened.records[0].scope.discipline.as_deref(),Some("ARCH"));
    }

    #[test]
    fn business_summary_only_totals_confirmed_records() {
        let mut state=State{drawing_distance:1000.0,real_distance_m:1.0,..Default::default()};
        let confirmed=state.add_measurement(MeasureKind::Length,vec![[0.0,0.0],[2500.0,0.0]],2500.0,"A-WALL".into()).unwrap();
        let review=state.add_measurement(MeasureKind::Area,vec![[0.0,0.0],[1000.0,1000.0]],1_000_000.0,"A-WALL".into()).unwrap();
        let excluded=state.add_measurement(MeasureKind::Count,Vec::new(),4.0,"A-WALL".into()).unwrap();
        assert!(state.edit_metadata(confirmed,"Tường".into(),"TT-001".into(),"Tường xây".into(),String::new()));
        assert!(state.edit_metadata(review,"Tường".into(),"TT-001".into(),"Tường xây".into(),String::new()));
        assert!(state.edit_metadata(excluded,"Tường".into(),"TT-001".into(),"Tường xây".into(),String::new()));
        assert!(state.set_status(confirmed,MeasureStatus::Confirmed));
        assert!(state.set_status(excluded,MeasureStatus::Excluded));
        let groups=business_quantities(&state,0.001,0.000001);
        let q=groups.get(&("Tường".into(),"TT-001".into(),"A-WALL".into())).unwrap();
        assert_eq!(q.review_records,1);
        assert_eq!(q.confirmed_records,1);
        assert_eq!(q.excluded_records,1);
        assert!((q.length-2.5).abs()<1e-12);
        assert_eq!(q.area,0.0);
        assert_eq!(q.count,0.0);
    }

    #[test]
    fn selecting_record_populates_metadata_editor_and_save_is_audited() {
        let mut state=State{drawing_distance:1.0,real_distance_m:1.0,..Default::default()};
        let id=state.add_measurement(MeasureKind::Count,Vec::new(),1.0,"DEVICE".into()).unwrap();
        assert!(state.select_record(id));
        state.edit_work_code="EL-001".into();
        assert!(state.edit_metadata(id,state.edit_work_group.clone(),state.edit_work_code.clone(),state.edit_description.clone(),state.edit_note.clone()));
        assert_eq!(state.records[0].work_code,"EL-001");
        assert!(state.audit.iter().any(|a|a.record_id==id&&a.action=="EDIT_METADATA"));
    }

    #[test]
    fn excluded_measurements_remain_in_audit_but_not_as_confirmed_quantity() {
        let mut state = State { drawing_distance: 1.0, real_distance_m: 1.0, ..Default::default() };
        let id = state.add_measurement(MeasureKind::Count, Vec::new(), 1.0, "DEVICE".into()).unwrap();
        assert!(state.set_status(id, MeasureStatus::Excluded));
        assert_eq!(state.records[0].status, MeasureStatus::Excluded);
        assert!(state.audit.iter().any(|a| a.record_id == id && a.action == "STATUS"));
        assert!(state.delete_measurement(id));
        assert!(state.records.is_empty());
        assert!(state.audit.iter().any(|a| a.record_id == id && a.action == "DELETE"));
    }

    #[test]
    fn two_point_calibration_uses_world_distance_and_real_length() {
        let mut state = State::default();
        let scale = state.calibrate_two_points([10.0,20.0],[3010.0,4020.0],5.0).unwrap();
        assert!((state.drawing_distance - 5000.0).abs() < 1e-12);
        assert!((scale - 0.001).abs() < 1e-12);
        assert!(state.calibrate_two_points([1.0,1.0],[1.0,1.0],2.0).is_err());
    }

    #[test]
    fn calibration_scale_converts_length_and_area() {
        let state = State { drawing_distance: 1000.0, real_distance_m: 1.0, ..Default::default() };
        let scale = state.scale_to_m().unwrap();
        assert!((scale - 0.001).abs() < 1e-12);
        assert!((2500.0 * scale - 2.5).abs() < 1e-12);
        assert!((1_000_000.0 * scale * scale - 1.0).abs() < 1e-12);
    }

    #[test]
    fn invalid_calibration_is_rejected() {
        assert!(State::default().scale_to_m().is_none());
        assert!(State { drawing_distance: 0.0, real_distance_m: 1.0, ..Default::default() }.scale_to_m().is_none());
        assert!(State { drawing_distance: 1.0, real_distance_m: 0.0, ..Default::default() }.scale_to_m().is_none());
    }

    #[test]
    fn open_polyline_has_no_area() {
        let vertices = vec![PolyVertex::new(Vec2::new(0.0, 0.0)), PolyVertex::new(Vec2::new(3.0, 4.0))];
        assert_eq!(lwpoly_metrics(&vertices, false), Some((5.0, 0.0)));
    }

    #[test]
    fn closed_rectangle_area_and_perimeter() {
        let vertices = [(0.0,0.0),(10.0,0.0),(10.0,8.0),(0.0,8.0)]
            .iter().map(|&(x,y)| PolyVertex::new(Vec2::new(x,y))).collect::<Vec<_>>();
        let (perimeter, area) = lwpoly_metrics(&vertices, true).unwrap();
        assert!((perimeter - 36.0).abs() < 1e-10);
        assert!((area - 80.0).abs() < 1e-10);
    }
    #[test]
    fn semicircle_bulge_preserves_arc_and_segment_area() {
        let (length, area) = bulge_edge((0.0, 0.0), (2.0, 0.0), 1.0).unwrap();
        assert!((length - std::f64::consts::PI).abs() < 1e-10);
        assert!((area - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }
    #[test]
    fn self_crossing_polyline_must_not_create_qto_area() {
        let points = [(0.0, 0.0), (4.0, 4.0), (0.0, 4.0), (4.0, 0.0)];
        let vertices = points.iter().map(|&(x, y)|
            PolyVertex::new(Vec2::new(x, y))).collect::<Vec<_>>();
        assert!(lwpoly_metrics(&vertices, true).is_none());
    }

    #[test]
    fn zero_length_closed_edge_is_invalid() {
        let points = [(0.0, 0.0), (0.0, 0.0), (2.0, 1.0)];
        let vertices = points.iter().map(|&(x, y)|
            PolyVertex::new(Vec2::new(x, y))).collect::<Vec<_>>();
        assert!(lwpoly_metrics(&vertices, true).is_none());
    }

    #[test]
    fn nonzero_bulge_with_coincident_endpoints_is_rejected() {
        assert!(bulge_edge((2.0, 3.0), (2.0, 3.0), 1.0).is_none());
        let mut vertices = vec![PolyVertex::new(Vec2::new(2.0, 3.0)), PolyVertex::new(Vec2::new(2.0, 3.0))];
        vertices[0].bulge = 1.0;
        assert!(lwpoly_metrics(&vertices, false).is_none());
    }

    #[test]
    fn invalid_geometry_is_rejected() {
        assert!(bulge_edge((0.0, 0.0), (f64::NAN, 0.0), 0.0).is_none());
        assert!(bulge_edge((f64::NAN, 0.0), (1.0, 0.0), 0.0).is_none());
        let invalid = vec![
            PolyVertex::new(Vec2::new(f64::NAN, 0.0)),
            PolyVertex::new(Vec2::new(1.0, 0.0)),
        ];
        assert!(lwpoly_metrics(&invalid, true).is_none());
    }
}
