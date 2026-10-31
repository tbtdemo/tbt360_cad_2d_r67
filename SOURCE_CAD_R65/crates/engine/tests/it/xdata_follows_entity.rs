//! Other applications' xdata is part of its object, as in AutoCAD: COPY, MIRROR, ARRAY and paste
//! give the copies the same xdata, so it is saved on them too; the pieces of an exploded object
//! don't keep it.

use cadcraft_engine::Session;
use cadcraft_engine::cmd::file::{IoHooks,set_io};
use cadcraft_engine::doc::{Drawing, EntityKind, Handle, RawTag};
use serde_json::json;

/// A LINE with a hyperlink (`PE_URL`) and another application's data, and an LWPOLYLINE with
/// the same, read from DXF.
const DXF: &str = "0\nSECTION\n2\nENTITIES\n\
0\nLINE\n5\n2A\n8\n0\n10\n0\n20\n0\n30\n0\n11\n10\n21\n0\n31\n0\n\
1001\nPE_URL\n1000\nhttps://example.com/door-7\n1002\n{\n1000\nDoor 7\n1002\n}\n\
1001\nACME_TAGS\n1000\nD-07\n1070\n3\n1040\n2.5\n\
0\nLWPOLYLINE\n5\n2B\n8\n0\n90\n3\n70\n0\n10\n0\n20\n5\n10\n5\n20\n5\n10\n5\n20\n10\n\
1001\nACME_TAGS\n1000\nP-01\n\
0\nENDSEC\n0\nEOF\n";

fn tags(xdata: &[RawTag]) -> Vec<(i32, String)> {
    xdata.iter().map(|t| (t.code, t.value.trim().to_string())).collect()
}

fn session() -> (Session, Handle, Handle) {
    let d = cadcraft_io::read_dxf(DXF.as_bytes()).unwrap();
    let mut s = Session::empty();
    s.open_drawing(d, "xdata", None);
    let doc = s.doc().unwrap();
    let find = |line: bool| doc.model.iter().find(|e| matches!(e.kind, EntityKind::Line(_)) == line).map(|e| e.handle).unwrap();
    let (l, p) = (find(true), find(false));
    (s, l, p)
}

fn xdata_of(d: &Drawing, h: Handle) -> Vec<(i32, String)> {
    tags(&d.entity(h).unwrap().common.xdata)
}

fn new_handles(s: &Session, before: &[Handle]) -> Vec<Handle> {
    s.doc().unwrap().model.iter().map(|e| e.handle).filter(|h| !before.contains(h)).collect()
}

fn model_handles(s: &Session) -> Vec<Handle> {
    s.doc().unwrap().model.iter().map(|e| e.handle).collect()
}

fn add_bim(s:&mut Session,h:Handle,floor:u64,discipline:&str,owner:&str,revision:u64) {
    s.doc_mut().unwrap().modify_entity(h,|e|e.common.xdata.extend([
        RawTag{code:1001,value:"TBT360_BIM".into()},
        RawTag{code:1000,value:format!("floor={floor};discipline={discipline};owner={owner};revision={revision};entity={}",h.hex())},
    ])).unwrap();
}

fn bim_payload(s:&Session,h:Handle)->String {
    let e=s.doc().unwrap().entity(h).unwrap();
    let at=e.common.xdata.iter().position(|t|t.code==1001&&t.value=="TBT360_BIM").unwrap();
    e.common.xdata.get(at+1).unwrap().value.clone()
}

#[test]
fn wblock_insert_real_file_preserves_bim_member_identity_and_explode_scope() {
    set_io(IoHooks{
        read:|bytes,name|cadcraft_io::read(bytes,name).map_err(|e|e.to_string()),
        write:|d,name|cadcraft_io::write(d,name).map_err(|e|e.to_string()),
        plot:None,
    });

    let (mut source,line_handle,_)=session();
    add_bim(&mut source,line_handle,2,"ELEC","u1",7);
    let dir=std::env::temp_dir().join(format!("cadcraft-bim-wblock-{}",std::process::id()));
    let _=std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path=dir.join("part.dxf");

    source.execute("wblock",&json!({"path":path.to_string_lossy(),"handles":[line_handle.hex()],"base":[0,0]})).unwrap();
    let written=cadcraft_io::read(&std::fs::read(&path).unwrap(),&path.to_string_lossy()).unwrap();
    let written_member=written.model.first().unwrap();
    let at=written_member.common.xdata.iter().position(|t|t.code==1001&&t.value=="TBT360_BIM").unwrap();
    let payload=&written_member.common.xdata[at+1].value;
    assert!(payload.contains("floor=2;discipline=ELEC;owner=u1;revision=1"));
    assert!(payload.contains(&format!("entity={}",written_member.handle.hex())));

    let mut target=Session::empty();
    target.open_drawing(Drawing::new_metric(),"target",None);
    let r=target.execute("insert",&json!({"file":path.to_string_lossy(),"at":[0,0]})).unwrap();
    let insert_handle=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let insert_entity=target.doc().unwrap().entity(insert_handle).unwrap();
    let EntityKind::Insert(ins)=&insert_entity.kind else{panic!("expected insert")};
    let block=target.doc().unwrap().block(&ins.block).unwrap();
    let member=block.entities.first().unwrap();
    let at=member.common.xdata.iter().position(|t|t.code==1001&&t.value=="TBT360_BIM").unwrap();
    let payload=&member.common.xdata[at+1].value;
    assert!(payload.contains("floor=2;discipline=ELEC;owner=u1;revision=1"));
    assert!(payload.contains(&format!("entity={}",member.handle.hex())));

    let before=model_handles(&target);
    target.execute("explode",&json!({"handles":[insert_handle.hex()]})).unwrap();
    let pieces=new_handles(&target,&before);
    assert_eq!(pieces.len(),1);
    let piece=pieces[0];
    let payload=bim_payload(&target,piece);
    assert!(payload.contains("floor=2;discipline=ELEC;owner=u1;revision=1"));
    assert!(payload.contains(&format!("entity={}",piece.hex())));
}

#[test]
fn reading_puts_xdata_on_the_entity() {
    let (s, l, p) = session();
    let d = s.doc().unwrap();
    let x = xdata_of(d, l);
    assert_eq!(x.first(), Some(&(1001, "PE_URL".to_string())));
    assert!(x.contains(&(1001, "ACME_TAGS".to_string())) && x.contains(&(1000, "D-07".to_string())));
    assert_eq!(xdata_of(d, p), vec![(1001, "ACME_TAGS".to_string()), (1000, "P-01".to_string())]);
    assert!(d.foreign.entities.values().all(|x| x.xdata.is_empty()), "entity xdata is not kept twice");
}

#[test]
fn copies_keep_the_xdata() {
    let (mut s, l, _) = session();
    let want = xdata_of(s.doc().unwrap(), l);

    let before = model_handles(&s);
    s.execute("copy", &json!({ "handles": [l.hex()], "delta": [0, 20] })).unwrap();
    let copied = new_handles(&s, &before);
    assert_eq!(copied.len(), 1);
    assert_eq!(xdata_of(s.doc().unwrap(), copied[0]), want, "COPY");

    let before = model_handles(&s);
    s.execute("mirror", &json!({ "handles": [l.hex()], "p1": [0, -1], "p2": [10, -1] })).unwrap();
    let mirrored = new_handles(&s, &before);
    assert_eq!(mirrored.len(), 1);
    assert_eq!(xdata_of(s.doc().unwrap(), mirrored[0]), want, "MIRROR");

    let before = model_handles(&s);
    s.execute("arrayrect", &json!({ "handles": [l.hex()], "rows": 1, "cols": 3, "colSpacing": 20, "associative": false })).unwrap();
    let arrayed = new_handles(&s, &before);
    assert_eq!(arrayed.len(), 2);
    for h in arrayed {
        assert_eq!(xdata_of(s.doc().unwrap(), h), want, "ARRAYRECT");
    }

    s.execute("copyclip", &json!({ "handles": [l.hex()] })).unwrap();
    let before = model_handles(&s);
    s.execute("pasteclip", &json!({ "at": [100, 100] })).unwrap();
    let pasted = new_handles(&s, &before);
    assert_eq!(pasted.len(), 1);
    assert_eq!(xdata_of(s.doc().unwrap(), pasted[0]), want, "PASTECLIP");

    // Undo takes the copy away; the original keeps its xdata.
    s.execute("u", &json!({})).unwrap();
    assert!(s.doc().unwrap().entity(pasted[0]).is_none());
    assert_eq!(xdata_of(s.doc().unwrap(), l), want);
}

#[test]
fn tbt360_bim_copies_get_independent_identity() {
    let (mut s,l,_)=session();
    s.doc_mut().unwrap().modify_entity(l,|e|e.common.xdata.extend([
        RawTag{code:1001,value:"TBT360_BIM".into()},
        RawTag{code:1000,value:format!("floor=2;discipline=ELEC;owner=u1;revision=7;entity={}",l.hex())},
    ])).unwrap();

    let payload=|s:&Session,h:Handle|{
        let e=s.doc().unwrap().entity(h).unwrap();
        let at=e.common.xdata.iter().position(|t|t.code==1001&&t.value=="TBT360_BIM").unwrap();
        e.common.xdata.get(at+1).unwrap().value.clone()
    };
    let assert_copy=|s:&Session,h:Handle|{
        let p=payload(s,h);
        assert!(p.contains("floor=2"));
        assert!(p.contains("discipline=ELEC"));
        assert!(p.contains("owner=u1"));
        assert!(p.contains("revision=1"));
        assert!(p.contains(&format!("entity={}",h.hex())));
    };

    let before=model_handles(&s);
    s.execute("copy",&json!({"handles":[l.hex()],"delta":[0,20]})).unwrap();
    let copied=new_handles(&s,&before);
    assert_eq!(copied.len(),1);
    assert_copy(&s,copied[0]);

    let before=model_handles(&s);
    s.execute("mirror",&json!({"handles":[l.hex()],"p1":[0,-1],"p2":[10,-1]})).unwrap();
    let mirrored=new_handles(&s,&before);
    assert_eq!(mirrored.len(),1);
    assert_copy(&s,mirrored[0]);

    let before=model_handles(&s);
    s.execute("arrayrect",&json!({"handles":[l.hex()],"rows":1,"cols":3,"colSpacing":20,"associative":false})).unwrap();
    let arrayed=new_handles(&s,&before);
    assert_eq!(arrayed.len(),2);
    for h in arrayed { assert_copy(&s,h); }

    let original=payload(&s,l);
    assert!(original.contains("revision=7"));
    assert!(original.contains(&format!("entity={}",l.hex())));
}

#[test]
fn a_copy_saves_its_xdata() {
    let (mut s, l, _) = session();
    s.execute("copy", &json!({ "handles": [l.hex()], "delta": [0, 20] })).unwrap();
    let text = cadcraft_io::write_dxf(s.doc().unwrap());
    let groups: Vec<&str> = text.lines().map(str::trim).collect();
    let count = |code: &str, value: &str| groups.chunks(2).filter(|g| *g == [code, value]).count();
    assert_eq!(count("1001", "PE_URL"), 2, "on the line and on its copy");
    assert_eq!(count("1000", "https://example.com/door-7"), 2);
    assert_eq!((count("2", "PE_URL"), count("2", "ACME_TAGS")), (1, 1), "the applications stay registered");
    let back = cadcraft_io::read_dxf(text.as_bytes()).unwrap();
    let lines: Vec<_> = back.model.iter().filter(|e| matches!(e.kind, EntityKind::Line(_))).collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(tags(&lines[0].common.xdata), tags(&lines[1].common.xdata));
}

#[test]
fn break_pieces_get_unique_bim_identity() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,2,"ELEC","u1",5);
    let r=s.execute("break",&json!({"handle":h.hex(),"p1":[3,0],"p2":[7,0]})).unwrap();
    let hs=r["handles"].as_array().unwrap().iter().filter_map(|v|v.as_str().and_then(Handle::parse_hex)).collect::<Vec<_>>();
    assert_eq!(hs.len(),2);
    assert!(s.doc().unwrap().entity(h).is_none());
    for nh in hs {
        let p=bim_payload(&s,nh);
        assert!(p.contains("floor=2"));
        assert!(p.contains("discipline=ELEC"));
        assert!(p.contains("revision=1"));
        assert!(p.contains(&format!("entity={}",nh.hex())));
    }
}

#[test]
fn join_same_scope_creates_one_new_bim_identity() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[5,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",2);
    add_bim(&mut s,b,1,"ARCH","u1",3);
    let r=s.execute("join",&json!({"handles":[a.hex(),b.hex()]})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("floor=1"));
    assert!(p.contains("discipline=ARCH"));
    assert!(p.contains("owner=u1"));
    assert!(p.contains("revision=1"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn join_different_scopes_is_left_unscoped() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[5,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",1);
    add_bim(&mut s,b,2,"STRUCT","u2",1);
    let r=s.execute("join",&json!({"handles":[a.hex(),b.hex()]})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let e=s.doc().unwrap().entity(h).unwrap();
    assert!(!e.common.xdata.iter().any(|t|t.code==1001&&t.value=="TBT360_BIM"));
}

#[test]
fn pedit_vertex_move_keeps_identity_and_bumps_revision() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,2,"ELEC","u1",4);
    s.execute("pedit",&json!({"handle":h.hex(),"option":"vertex","index":1,"action":"move","to":[10,2]})).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("revision=5"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn pedit_join_same_scope_keeps_base_identity_and_bumps_revision() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[5,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",2);
    add_bim(&mut s,b,1,"ARCH","u1",4);
    let r=s.execute("pedit",&json!({"handle":a.hex(),"option":"join","handles2":[b.hex()]})).unwrap();
    assert_eq!(r["handle"].as_str(),Some(a.hex().as_str()));
    assert!(s.doc().unwrap().entity(b).is_none());
    let p=bim_payload(&s,a);
    assert!(p.contains("revision=3"));
    assert!(p.contains(&format!("entity={}",a.hex())));
}

#[test]
fn pedit_join_different_scopes_strips_bim_from_result() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[5,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",2);
    add_bim(&mut s,b,2,"STRUCT","u2",4);
    s.execute("pedit",&json!({"handle":a.hex(),"option":"join","handles2":[b.hex()]})).unwrap();
    let e=s.doc().unwrap().entity(a).unwrap();
    assert!(!e.common.xdata.iter().any(|t|t.code==1001&&t.value=="TBT360_BIM"));
}

#[test]
fn grip_stretch_keeps_identity_and_bumps_revision() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,2,"ELEC","u1",2);
    s.execute("grip.stretch",&json!({"handle":h.hex(),"index":1,"to":[12,0]})).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("revision=3"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn grip_stretch_copy_gets_new_identity() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,2,"ELEC","u1",2);
    let r=s.execute("grip.stretch",&json!({"grips":[[h.hex(),1]],"delta":[2,0],"copy":true})).unwrap();
    let nh=Handle::parse_hex(r["handles"][0].as_str().unwrap()).unwrap();
    let p=bim_payload(&s,nh);
    assert!(p.contains("revision=1"));
    assert!(p.contains(&format!("entity={}",nh.hex())));
    assert!(bim_payload(&s,h).contains("revision=2"));
}

#[test]
fn move_and_undo_restore_bim_revision_and_geometry() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,2,"ELEC","u1",7);
    s.execute("move",&json!({"handles":[h.hex()],"delta":[5,0]})).unwrap();
    assert!(bim_payload(&s,h).contains("revision=8"));
    s.execute("undo",&json!({})).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("revision=7"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn delete_and_undo_restore_same_bim_entity_identity() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,3,"STRUCT","u2",6);
    let before=bim_payload(&s,h);
    s.execute("erase",&json!({"handles":[h.hex()]})).unwrap();
    assert!(s.doc().unwrap().entity(h).is_none());
    s.execute("undo",&json!({})).unwrap();
    assert_eq!(bim_payload(&s,h),before);
}

#[test]
fn stretch_lengthen_and_align_bump_bim_revision_in_place() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,2,"ELEC","u1",1);

    s.execute("stretch",&json!({"window":[[9,-1],[11,1]],"delta":[2,0]})).unwrap();
    assert!(bim_payload(&s,h).contains("revision=2"));

    s.execute("lengthen",&json!({"handle":h.hex(),"delta":2.0})).unwrap();
    assert!(bim_payload(&s,h).contains("revision=3"));

    s.execute("align",&json!({"handles":[h.hex()],"s1":[0,0],"d1":[5,0]})).unwrap();
    assert!(bim_payload(&s,h).contains("revision=4"));
    assert!(bim_payload(&s,h).contains(&format!("entity={}",h.hex())));
}

#[test]
fn pasteclip_assigns_new_bim_identity() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,2,"ELEC","u1",7);
    s.execute("copyclip",&json!({"handles":[h.hex()]})).unwrap();
    let r=s.execute("pasteclip",&json!({"at":[20,0]})).unwrap();
    let nh=Handle::parse_hex(r["handles"][0].as_str().unwrap()).unwrap();
    let p=bim_payload(&s,nh);
    assert!(p.contains("floor=2"));
    assert!(p.contains("discipline=ELEC"));
    assert!(p.contains("revision=1"));
    assert!(p.contains(&format!("entity={}",nh.hex())));
    assert!(bim_payload(&s,h).contains("revision=7"));
}

#[test]
fn pasteblock_uses_shared_scope_only() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[0,2],[5,2]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",2);
    add_bim(&mut s,b,1,"ARCH","u1",3);
    s.execute("copyclip",&json!({"handles":[a.hex(),b.hex()]})).unwrap();
    let r=s.execute("pasteblock",&json!({"at":[20,0]})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("floor=1"));
    assert!(p.contains("discipline=ARCH"));
    assert!(p.contains("owner=u1"));
    assert!(p.contains("revision=1"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn associative_arraypath_refresh_bumps_array_revision() {
    let mut s=Session::new();
    let obj=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[1,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let path=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[20,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,obj,2,"ELEC","u1",1);
    let r=s.execute("arraypath",&json!({"handles":[obj.hex()],"path":path.hex(),"count":3,"associative":true})).unwrap();
    let arr=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let before=bim_payload(&s,arr);
    assert!(before.contains("revision=1"));
    s.execute("move",&json!({"handles":[path.hex()],"delta":[0,5]})).unwrap();
    s.execute("arraypathrefresh",&json!({"handle":arr.hex()})).unwrap();
    let after=bim_payload(&s,arr);
    assert!(after.contains("revision=2"));
    assert!(after.contains(&format!("entity={}",arr.hex())));
}

#[test]
fn block_insert_inherits_shared_bim_scope() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[0,2],[5,2]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,4,"STRUCT","u2",2);
    add_bim(&mut s,b,4,"STRUCT","u2",3);
    s.execute("block",&json!({"name":"BIM_SHARED","base":[0,0],"handles":[a.hex(),b.hex()],"keep":"retain"})).unwrap();
    let r=s.execute("insert",&json!({"name":"BIM_SHARED","at":[20,0]})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("floor=4"));
    assert!(p.contains("discipline=STRUCT"));
    assert!(p.contains("owner=u2"));
    assert!(p.contains("revision=1"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn block_insert_with_mixed_scopes_is_unscoped() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[0,2],[5,2]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",1);
    add_bim(&mut s,b,2,"ELEC","u2",1);
    s.execute("block",&json!({"name":"BIM_MIXED","base":[0,0],"handles":[a.hex(),b.hex()],"keep":"retain"})).unwrap();
    let r=s.execute("insert",&json!({"name":"BIM_MIXED","at":[20,0]})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let e=s.doc().unwrap().entity(h).unwrap();
    assert!(!e.common.xdata.iter().any(|t|t.code==1001&&t.value=="TBT360_BIM"));
}

#[test]
fn reverse_and_flatten_bump_bim_revision_in_place() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0,2],[10,0,2]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,2,"ELEC","u1",1);
    s.execute("reverse",&json!({"handles":[h.hex()]})).unwrap();
    assert!(bim_payload(&s,h).contains("revision=2"));
    s.execute("flatten",&json!({"handles":[h.hex()]})).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("revision=3"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn blend_same_scope_gets_new_bim_identity() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[10,5],[10,10]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,2,"ELEC","u1",1);
    add_bim(&mut s,b,2,"ELEC","u1",2);
    let r=s.execute("blend",&json!({"h1":a.hex(),"p1":[5,0],"h2":b.hex(),"p2":[10,5]})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("floor=2"));
    assert!(p.contains("discipline=ELEC"));
    assert!(p.contains("owner=u1"));
    assert!(p.contains("revision=1"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn blend_mixed_scopes_is_unscoped() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[10,5],[10,10]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",1);
    add_bim(&mut s,b,2,"STRUCT","u2",1);
    let r=s.execute("blend",&json!({"h1":a.hex(),"p1":[5,0],"h2":b.hex(),"p2":[10,5]})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let e=s.doc().unwrap().entity(h).unwrap();
    assert!(!e.common.xdata.iter().any(|t|t.code==1001&&t.value=="TBT360_BIM"));
}

#[test]
fn overkill_keeps_distinct_bim_identities_separate() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",1);
    add_bim(&mut s,b,1,"ARCH","u1",1);
    s.execute("overkill",&json!({"handles":[a.hex(),b.hex()]})).unwrap();
    assert!(s.doc().unwrap().entity(a).is_some());
    assert!(s.doc().unwrap().entity(b).is_some());
}

#[test]
fn splinedit_keeps_identity_and_bumps_revision() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("spline",&json!({"fit":[[0,0],[5,2],[10,0]]})).unwrap()["handle"].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,3,"ARCH","u1",2);
    s.execute("splinedit",&json!({"handle":h.hex(),"option":"reverse"})).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("revision=3"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn properties_set_keeps_identity_and_bumps_revision() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,2,"ELEC","u1",4);
    s.execute("properties.set",&json!({"handles":[h.hex()],"length":12.0,"color":"3"})).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("revision=5"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn layer_move_and_merge_bump_bim_revision() {
    let mut s=Session::new();
    s.execute("layer.new",&json!({"name":"A"})).unwrap();
    s.execute("layer.new",&json!({"name":"B"})).unwrap();
    let h=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,1,"ARCH","u1",1);
    s.execute("laymch",&json!({"handles":[h.hex()],"layer":"A"})).unwrap();
    assert!(bim_payload(&s,h).contains("revision=2"));
    s.execute("laymrg",&json!({"from":["A"],"to":"B"})).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("revision=3"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn textedit_keeps_identity_and_undo_restores_revision() {
    let mut s=Session::new();
    let h=Handle::parse_hex(s.execute("text",&json!({"at":[0,0],"text":"A","height":2.5})).unwrap()["handle"].as_str().unwrap()).unwrap();
    add_bim(&mut s,h,1,"ARCH","u1",7);
    s.execute("textedit",&json!({"handle":h.hex(),"text":"B"})).unwrap();
    assert!(bim_payload(&s,h).contains("revision=8"));
    s.execute("undo",&json!({})).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("revision=7"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn array_grip_edit_keeps_array_identity_and_bumps_revision() {
    let mut s=Session::new();
    let src=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[1,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,src,2,"ELEC","u1",1);
    let r=s.execute("arrayrect",&json!({"handles":[src.hex()],"rows":1,"cols":2,"rowSpacing":5.0,"colSpacing":10.0,"associative":true})).unwrap();
    let arr=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    assert!(bim_payload(&s,arr).contains("revision=1"));
    s.execute("arraygripset",&json!({"handle":arr.hex(),"parameter":"cols","value":3})).unwrap();
    let p=bim_payload(&s,arr);
    assert!(p.contains("revision=2"));
    assert!(p.contains(&format!("entity={}",arr.hex())));
}

#[test]
fn common_scope_rule_centermark_inherits_single_source() {
    let mut s=Session::new();
    let src=Handle::parse_hex(s.execute("circle",&json!({"center":[0,0],"radius":5.0})).unwrap()["handle"].as_str().unwrap()).unwrap();
    add_bim(&mut s,src,2,"ELEC","u1",3);
    let r=s.execute("centermark",&json!({"handle":src.hex()})).unwrap();
    let hs=r["handles"].as_array().unwrap();
    assert_eq!(hs.len(),2);
    for v in hs {
        let h=Handle::parse_hex(v.as_str().unwrap()).unwrap();
        let p=bim_payload(&s,h);
        assert!(p.contains("floor=2"));
        assert!(p.contains("discipline=ELEC"));
        assert!(p.contains("owner=u1"));
        assert!(p.contains("revision=1"));
        assert!(p.contains(&format!("entity={}",h.hex())));
    }
}

#[test]
fn common_scope_rule_centerline_requires_matching_sources() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[0,4],[10,4]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",1);
    add_bim(&mut s,b,1,"ARCH","u1",2);
    let r=s.execute("centerline",&json!({"h1":a.hex(),"h2":b.hex()})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("floor=1"));
    assert!(p.contains("discipline=ARCH"));
    assert!(p.contains("owner=u1"));
    assert!(p.contains("revision=1"));
}

#[test]
fn common_scope_rule_divide_inherits_source() {
    let mut s=Session::new();
    let src=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[12,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,src,3,"STRUCT","u2",4);
    let r=s.execute("divide",&json!({"handle":src.hex(),"segments":3})).unwrap();
    let hs=r["handles"].as_array().unwrap();
    assert_eq!(hs.len(),2);
    for v in hs {
        let h=Handle::parse_hex(v.as_str().unwrap()).unwrap();
        let p=bim_payload(&s,h);
        assert!(p.contains("floor=3"));
        assert!(p.contains("discipline=STRUCT"));
        assert!(p.contains("owner=u2"));
    }
}

#[test]
fn common_scope_rule_measure_inherits_source() {
    let mut s=Session::new();
    let src=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[12,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,src,4,"ELEC","u3",5);
    let r=s.execute("measure",&json!({"handle":src.hex(),"length":4.0})).unwrap();
    let hs=r["handles"].as_array().unwrap();
    assert_eq!(hs.len(),2);
    for v in hs {
        let h=Handle::parse_hex(v.as_str().unwrap()).unwrap();
        assert!(bim_payload(&s,h).contains("floor=4"));
    }
}

#[test]
fn common_scope_rule_wipeout_inherits_polyline_source() {
    let mut s=Session::new();
    let src=Handle::parse_hex(s.execute("pline",&json!({"points":[[0,0],[10,0],[10,5],[0,5]],"closed":true})).unwrap()["handle"].as_str().unwrap()).unwrap();
    add_bim(&mut s,src,2,"ARCH","u1",6);
    let r=s.execute("wipeout",&json!({"handle":src.hex(),"erase":false})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("floor=2"));
    assert!(p.contains("discipline=ARCH"));
    assert!(p.contains("owner=u1"));
    assert!(p.contains("revision=1"));
}

#[test]
fn shared_derived_scope_qdim_inherits_matching_sources() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[5,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,2,"ARCH","u1",1);
    add_bim(&mut s,b,2,"ARCH","u1",2);
    let r=s.execute("qdim",&json!({"handles":[a.hex(),b.hex()],"at":[0,4]})).unwrap();
    let hs=r["handles"].as_array().unwrap();
    assert!(!hs.is_empty());
    for v in hs {
        let h=Handle::parse_hex(v.as_str().unwrap()).unwrap();
        let p=bim_payload(&s,h);
        assert!(p.contains("floor=2"));
        assert!(p.contains("discipline=ARCH"));
        assert!(p.contains("owner=u1"));
        assert!(p.contains("revision=1"));
    }
}

#[test]
fn shared_derived_scope_qdim_mixed_sources_stays_unscoped() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[5,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[5,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",1);
    add_bim(&mut s,b,2,"ELEC","u2",1);
    let r=s.execute("qdim",&json!({"handles":[a.hex(),b.hex()],"at":[0,4]})).unwrap();
    for v in r["handles"].as_array().unwrap() {
        let h=Handle::parse_hex(v.as_str().unwrap()).unwrap();
        let e=s.doc().unwrap().entity(h).unwrap();
        assert!(!e.common.xdata.iter().any(|t|t.code==1001&&t.value=="TBT360_BIM"));
    }
}

#[test]
fn shared_derived_scope_leader_inherits_explicit_source() {
    let mut s=Session::new();
    let src=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,src,3,"ELEC","u1",5);
    let r=s.execute("leader",&json!({"points":[[0,0],[4,2]],"source":src.hex()})).unwrap();
    let h=Handle::parse_hex(r["handles"][0].as_str().unwrap()).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("floor=3"));
    assert!(p.contains("discipline=ELEC"));
    assert!(p.contains("owner=u1"));
}

#[test]
fn shared_derived_scope_mleader_inherits_explicit_sources() {
    let mut s=Session::new();
    let src=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,src,4,"STRUCT","u2",2);
    let r=s.execute("mleader",&json!({"points":[[0,0],[4,2]],"text":"A","sources":[src.hex()]})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let p=bim_payload(&s,h);
    assert!(p.contains("floor=4"));
    assert!(p.contains("discipline=STRUCT"));
    assert!(p.contains("owner=u2"));
}

#[test]
fn shared_derived_scope_tangent_circle_mixed_sources_stays_unscoped() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[20,-5],[20,5]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",1);
    add_bim(&mut s,b,2,"ELEC","u2",1);
    let r=s.execute("circle.ttr",&json!({"h1":a.hex(),"p1":[5,0],"h2":b.hex(),"p2":[20,0],"radius":5.0})).unwrap();
    let h=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let e=s.doc().unwrap().entity(h).unwrap();
    assert!(!e.common.xdata.iter().any(|t|t.code==1001&&t.value=="TBT360_BIM"));
}

#[test]
fn txt2mtxt_matching_sources_keep_shared_bim_scope() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("text",&json!({"at":[0,2],"text":"A","height":1.0})).unwrap()["handle"].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("text",&json!({"at":[0,0],"text":"B","height":1.0})).unwrap()["handle"].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,2,"ARCH","u1",3);
    add_bim(&mut s,b,2,"ARCH","u1",7);
    let r=s.execute("txt2mtxt",&json!({"handles":[a.hex(),b.hex()],"combine":true,"sort":"selection"})).unwrap();
    let h=Handle::parse_hex(r["handles"][0].as_str().unwrap()).unwrap();
    assert_eq!(h,a);
    let p=bim_payload(&s,h);
    assert!(p.contains("floor=2"));
    assert!(p.contains("discipline=ARCH"));
    assert!(p.contains("owner=u1"));
    assert!(p.contains(&format!("entity={}",h.hex())));
}

#[test]
fn txt2mtxt_mixed_sources_strip_bim_scope() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("text",&json!({"at":[0,2],"text":"A","height":1.0})).unwrap()["handle"].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("text",&json!({"at":[0,0],"text":"B","height":1.0})).unwrap()["handle"].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",1);
    add_bim(&mut s,b,2,"ELEC","u2",1);
    let r=s.execute("txt2mtxt",&json!({"handles":[a.hex(),b.hex()],"combine":true,"sort":"selection"})).unwrap();
    let h=Handle::parse_hex(r["handles"][0].as_str().unwrap()).unwrap();
    let e=s.doc().unwrap().entity(h).unwrap();
    assert!(!e.common.xdata.iter().any(|t|t.code==1001&&t.value=="TBT360_BIM"));
}

#[test]
fn fillet_same_bim_scope_bumps_sources_and_scopes_new_arc() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[10,0],[10,10]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,2,"ELEC","u1",3);
    add_bim(&mut s,b,2,"ELEC","u1",5);
    let r=s.execute("fillet",&json!({"h1":a.hex(),"p1":[5,0],"h2":b.hex(),"p2":[10,5],"radius":1.0,"trim":true})).unwrap();
    let arc=Handle::parse_hex(r["arc"].as_str().unwrap()).unwrap();
    assert!(bim_payload(&s,a).contains("revision=4"));
    assert!(bim_payload(&s,b).contains("revision=6"));
    let p=bim_payload(&s,arc);
    assert!(p.contains("floor=2"));
    assert!(p.contains("discipline=ELEC"));
    assert!(p.contains("owner=u1"));
    assert!(p.contains("revision=1"));
    assert!(p.contains(&format!("entity={}",arc.hex())));
}

#[test]
fn chamfer_different_bim_scopes_does_not_assign_join_scope() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[10,0],[10,10]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,1,"ARCH","u1",1);
    add_bim(&mut s,b,2,"STRUCT","u2",1);
    let r=s.execute("chamfer",&json!({"h1":a.hex(),"p1":[5,0],"h2":b.hex(),"p2":[10,5],"d1":1.0,"d2":1.0,"trim":true})).unwrap();
    let join=Handle::parse_hex(r["line"].as_str().unwrap()).unwrap();
    let e=s.doc().unwrap().entity(join).unwrap();
    assert!(!e.common.xdata.iter().any(|t|t.code==1001&&t.value=="TBT360_BIM"));
    assert!(bim_payload(&s,a).contains("revision=2"));
    assert!(bim_payload(&s,b).contains("revision=2"));
}

#[test]
fn fillet_no_trim_does_not_bump_source_revisions() {
    let mut s=Session::new();
    let a=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let b=Handle::parse_hex(s.execute("line",&json!({"points":[[10,0],[10,10]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,a,2,"ELEC","u1",3);
    add_bim(&mut s,b,2,"ELEC","u1",5);
    let _=s.execute("fillet",&json!({"h1":a.hex(),"p1":[5,0],"h2":b.hex(),"p2":[10,5],"radius":1.0,"trim":false})).unwrap();
    assert!(bim_payload(&s,a).contains("revision=3"));
    assert!(bim_payload(&s,b).contains("revision=5"));
}

#[test]
fn trim_pieces_get_new_bim_identities() {
    let mut s=Session::new();
    let target=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let e1=Handle::parse_hex(s.execute("line",&json!({"points":[[3,-5],[3,5]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let e2=Handle::parse_hex(s.execute("line",&json!({"points":[[7,-5],[7,5]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,target,2,"ELEC","u1",7);
    let r=s.execute("trim",&json!({"handle":target.hex(),"pick":[5,0],"edges":[e1.hex(),e2.hex()]})).unwrap();
    let hs=r["handles"].as_array().unwrap().iter().filter_map(|v|v.as_str().and_then(Handle::parse_hex)).collect::<Vec<_>>();
    assert_eq!(hs.len(),2);
    assert!(s.doc().unwrap().entity(target).is_none());
    for h in hs {
        let p=bim_payload(&s,h);
        assert!(p.contains("floor=2"));
        assert!(p.contains("discipline=ELEC"));
        assert!(p.contains("owner=u1"));
        assert!(p.contains("revision=1"));
        assert!(p.contains(&format!("entity={}",h.hex())));
    }
}

#[test]
fn extend_keeps_bim_identity_and_bumps_revision() {
    let mut s=Session::new();
    let target=Handle::parse_hex(s.execute("line",&json!({"points":[[0,0],[10,0]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    let edge=Handle::parse_hex(s.execute("line",&json!({"points":[[20,-5],[20,5]]})).unwrap()["handles"][0].as_str().unwrap()).unwrap();
    add_bim(&mut s,target,3,"STRUCT","u2",4);
    s.execute("extend",&json!({"handle":target.hex(),"pick":[10,0],"edges":[edge.hex()]})).unwrap();
    let p=bim_payload(&s,target);
    assert!(p.contains("floor=3"));
    assert!(p.contains("discipline=STRUCT"));
    assert!(p.contains("owner=u2"));
    assert!(p.contains("revision=5"));
    assert!(p.contains(&format!("entity={}",target.hex())));
}

#[test]
fn offset_gets_new_tbt360_bim_identity() {
    let (mut s,l,_)=session();
    s.doc_mut().unwrap().modify_entity(l,|e|e.common.xdata.extend([
        RawTag{code:1001,value:"TBT360_BIM".into()},
        RawTag{code:1000,value:format!("floor=2;discipline=ELEC;owner=u1;revision=7;entity={}",l.hex())},
    ])).unwrap();
    let r=s.execute("offset",&json!({"handle":l.hex(),"distance":2.0,"side":[0.0,5.0]})).unwrap();
    let nh=Handle::parse_hex(r["handle"].as_str().unwrap()).unwrap();
    let p=xdata_of(s.doc().unwrap(),nh);
    let payload=p.iter().find(|(code,value)|*code==1000&&value.contains("floor=2")).unwrap().1.clone();
    assert!(payload.contains("discipline=ELEC"));
    assert!(payload.contains("owner=u1"));
    assert!(payload.contains("revision=1"));
    assert!(payload.contains(&format!("entity={}",nh.hex())));
    let original=xdata_of(s.doc().unwrap(),l);
    assert!(original.iter().any(|(_,v)|v.contains("revision=7")));
}

#[test]
fn exploded_bim_pieces_keep_scope_with_unique_identity_only() {
    let (mut s,_,p)=session();
    s.doc_mut().unwrap().modify_entity(p,|e|e.common.xdata.extend([
        RawTag{code:1001,value:"TBT360_BIM".into()},
        RawTag{code:1000,value:format!("floor=3;discipline=STRUCT;owner=u2;revision=4;entity={}",p.hex())},
    ])).unwrap();
    let before=model_handles(&s);
    s.execute("explode",&json!({"handles":[p.hex()]})).unwrap();
    let pieces=new_handles(&s,&before);
    assert_eq!(pieces.len(),2);
    for h in pieces {
        let tags=xdata_of(s.doc().unwrap(),h);
        assert!(!tags.iter().any(|(_,v)|v=="ACME_TAGS"||v=="P-01"),"foreign xdata stays removed on own EXPLODE pieces");
        let payload=tags.iter().find(|(code,value)|*code==1000&&value.contains("floor=3")).unwrap().1.clone();
        assert!(payload.contains("discipline=STRUCT"));
        assert!(payload.contains("owner=u2"));
        assert!(payload.contains("revision=1"));
        assert!(payload.contains(&format!("entity={}",h.hex())));
    }
}

#[test]
fn exploded_pieces_do_not_keep_the_xdata() {
    let (mut s, _, p) = session();
    let before = model_handles(&s);
    s.execute("explode", &json!({ "handles": [p.hex()] })).unwrap();
    let pieces = new_handles(&s, &before);
    assert_eq!(pieces.len(), 2);
    for h in pieces {
        assert!(s.doc().unwrap().entity(h).unwrap().common.xdata.is_empty());
    }
}
