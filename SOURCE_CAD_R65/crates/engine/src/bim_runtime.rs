use std::sync::Arc;

use cadcraft_doc::{Common, Drawing, Entity, EntityKind, EntityStore, Handle};

use crate::Session;

const APP:&str="TBT360_BIM";

fn payload_index(common:&Common)->Option<usize> {
    common.xdata.windows(2).position(|w|w[0].code==1001 && w[0].value.eq_ignore_ascii_case(APP) && w[1].code==1000).map(|i|i+1)
}

fn field(payload:&str,key:&str)->Option<String> {
    payload.split(';').find_map(|part|{
        let (k,v)=part.split_once('=')?;
        k.eq_ignore_ascii_case(key).then(||v.to_string())
    })
}

fn revision(common:&Common)->Option<u64> {
    let i=payload_index(common)?;
    field(&common.xdata.get(i)?.value,"revision")?.parse().ok()
}

fn identity(common:&Common)->Option<Handle> {
    let i=payload_index(common)?;
    Handle::parse_hex(&field(&common.xdata.get(i)?.value,"entity")?)
}

fn set_identity_revision(common:&mut Common,handle:Handle,revision:u64) {
    let Some(i)=payload_index(common) else { return };
    let Some(tag)=common.xdata.get_mut(i) else { return };
    let mut parts=tag.value.split(';').map(str::to_string).collect::<Vec<_>>();
    let mut has_entity=false;
    let mut has_revision=false;
    for part in &mut parts {
        if part.starts_with("entity=") {
            *part=format!("entity={}",handle.hex());
            has_entity=true;
        } else if part.starts_with("revision=") {
            *part=format!("revision={}",revision.max(1));
            has_revision=true;
        }
    }
    if !has_entity { parts.push(format!("entity={}",handle.hex())); }
    if !has_revision { parts.push(format!("revision={}",revision.max(1))); }
    tag.value=parts.join(";");
}

fn without_bim(entity:&Entity)->Entity {
    let mut e=entity.clone();
    let mut out=Vec::with_capacity(e.common.xdata.len());
    let mut skip=false;
    for tag in &e.common.xdata {
        if tag.code==1001 {
            skip=tag.value.eq_ignore_ascii_case(APP);
            if skip { continue; }
        }
        if skip { continue; }
        out.push(tag.clone());
    }
    e.common.xdata=out;
    e
}

fn reconcile_store(before:&EntityStore,after:&mut EntityStore)->usize {
    let handles=after.handles();
    let mut changed=0;
    for handle in handles {
        let Some(current)=after.get(handle).map(|e|(**e).clone()) else { continue };
        if payload_index(&current.common).is_none() { continue; }

        let previous=before.get(handle).map(|e|(**e).clone());
        let before_rev=previous.as_ref().and_then(|e|revision(&e.common));
        let current_rev=revision(&current.common).unwrap_or(1).max(1);
        let non_bim_changed=previous.as_ref().is_some_and(|e|without_bim(e)!=without_bim(&current));
        let target_rev=match before_rev {
            Some(r) if non_bim_changed && current_rev<=r => r.saturating_add(1),
            _ => current_rev,
        };
        let needs_identity=identity(&current.common)!=Some(handle);
        let needs_revision=target_rev!=current_rev;
        if needs_identity || needs_revision {
            after.modify(handle,|e|set_identity_revision(&mut e.common,handle,target_rev));
            changed+=1;
        }
    }
    changed
}

fn changed_blocks(before:&Drawing,after:&Drawing)->std::collections::BTreeSet<String> {
    let mut changed=std::collections::BTreeSet::new();
    for (name,block) in &after.blocks {
        if before.block(name).is_some_and(|old|**old!=**block) { changed.insert(name.to_ascii_uppercase()); }
    }
    changed
}

fn bump_insert_refs_in_store(before:&EntityStore,after:&mut EntityStore,changed:&std::collections::BTreeSet<String>)->usize {
    let handles=after.handles();
    let mut n=0;
    for h in handles {
        let should=after.get(h).is_some_and(|e|matches!(&e.kind,EntityKind::Insert(i) if changed.contains(&i.block.to_ascii_uppercase())));
        if !should { continue; }
        let Some(old_rev)=before.get(h).and_then(|e|revision(&e.common)) else { continue };
        let Some(cur_rev)=after.get(h).and_then(|e|revision(&e.common)) else { continue };
        if cur_rev>old_rev { continue; }
        after.modify(h,|e|set_identity_revision(&mut e.common,h,old_rev.saturating_add(1)));
        n+=1;
    }
    n
}

fn bump_changed_block_references(before:&Drawing,after:&mut Drawing,changed:&std::collections::BTreeSet<String>)->usize {
    if changed.is_empty() { return 0; }
    let mut n=bump_insert_refs_in_store(&before.model,&mut after.model,changed);

    for layout in &mut after.layouts {
        let empty=EntityStore::new();
        let old=before.layouts.iter().find(|x|x.name.eq_ignore_ascii_case(&layout.name)).map(|x|&x.entities).unwrap_or(&empty);
        n+=bump_insert_refs_in_store(old,&mut layout.entities,changed);
    }

    let names=after.blocks.keys().cloned().collect::<Vec<_>>();
    for name in names {
        let empty=EntityStore::new();
        let old=before.block(&name).map(|b|&b.entities).unwrap_or(&empty);
        if let Some(block)=after.blocks.get_mut(&name) {
            n+=bump_insert_refs_in_store(old,&mut Arc::make_mut(block).entities,changed);
        }
    }
    n
}

fn reconcile_drawing(before:&Drawing,after:&mut Drawing)->usize {
    let block_changes=changed_blocks(before,after);
    let mut changed=reconcile_store(&before.model,&mut after.model);

    for layout in &mut after.layouts {
        if let Some(old)=before.layouts.iter().find(|x|x.name.eq_ignore_ascii_case(&layout.name)) {
            changed+=reconcile_store(&old.entities,&mut layout.entities);
        } else {
            changed+=reconcile_store(&EntityStore::new(),&mut layout.entities);
        }
    }

    let names=after.blocks.keys().cloned().collect::<Vec<_>>();
    for name in names {
        let old=before.block(&name).map(|b|b.entities.clone()).unwrap_or_else(EntityStore::new);
        if let Some(block)=after.blocks.get_mut(&name) {
            let block=Arc::make_mut(block);
            changed+=reconcile_store(&old,&mut block.entities);
        }
    }

    changed+=bump_changed_block_references(before,after,&block_changes);
    changed
}

pub(crate) fn normalize_drawing(d:&mut Drawing)->usize {
    let empty=Drawing::new_imperial();
    reconcile_drawing(&empty,d)
}

/// Runtime safety net after every command, after constraints/associativity have settled.
///
/// Rules:
/// - an entity that already carries TBT360_BIM always has entity=<real handle>;
/// - an in-place entity change bumps revision when the command itself did not already do so;
/// - imported/copied/renumbered entities keep their scope and revision but get the real handle;
/// - no BIM scope is invented for an entity that did not already have TBT360_BIM.
pub(crate) fn after_command(s:&mut Session,before:Option<&Arc<Drawing>>,uid:Option<u64>) {
    let Some(before)=before else { return };
    if uid.is_some_and(|u|s.state().is_ok_and(|st|st.uid!=u)) { return; }
    let Ok(after)=s.doc_mut() else { return };
    let _=reconcile_drawing(before,after);
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadcraft_doc::{EntityKind, Line, RawTag, Space};
    use cadcraft_geom::Vec3;

    fn common(handle:Handle,revision:u64)->Common {
        let mut c=Common::default();
        c.xdata=vec![
            RawTag{code:1001,value:APP.into()},
            RawTag{code:1000,value:format!("floor=2;discipline=ELEC;owner=u1;revision={revision};entity={}",handle.hex())},
        ];
        c
    }

    fn drawing(handle:Handle,revision:u64)->Drawing {
        let mut d=Drawing::new_metric();
        d.add(&Space::Model,common(handle,revision),EntityKind::Line(Line{a:Vec3::new(0.0,0.0,0.0),b:Vec3::new(10.0,0.0,0.0)})).unwrap();
        d
    }

    #[test]
    fn runtime_bumps_changed_entity_only_once() {
        let h=Handle(1);
        let before=drawing(h,7);
        let mut after=before.clone();
        after.modify_entity(h,|e|if let EntityKind::Line(l)=&mut e.kind { l.b.x=12.0; }).unwrap();
        assert_eq!(reconcile_drawing(&before,&mut after),1);
        assert_eq!(revision(&after.entity(h).unwrap().common),Some(8));
        assert_eq!(reconcile_drawing(&before,&mut after),0);
        assert_eq!(revision(&after.entity(h).unwrap().common),Some(8));
    }

    #[test]
    fn runtime_does_not_double_bump_command_revision() {
        let h=Handle(1);
        let before=drawing(h,7);
        let mut after=before.clone();
        after.modify_entity(h,|e|{
            if let EntityKind::Line(l)=&mut e.kind { l.b.x=12.0; }
            set_identity_revision(&mut e.common,h,8);
        }).unwrap();
        reconcile_drawing(&before,&mut after);
        assert_eq!(revision(&after.entity(h).unwrap().common),Some(8));
    }

    #[test]
    fn runtime_repairs_renumbered_identity_without_resetting_revision() {
        let old=Handle(1);
        let new=Handle(0x55);
        let before=Drawing::new_metric();
        let mut after=drawing(old,9);
        let e=after.remove_entity(old).unwrap();
        let mut e=Arc::unwrap_or_clone(e);
        e.handle=new;
        after.model.push(e);
        reconcile_drawing(&before,&mut after);
        let c=&after.entity(new).unwrap().common;
        assert_eq!(identity(c),Some(new));
        assert_eq!(revision(c),Some(9));
    }

    #[test]
    fn runtime_ignores_non_bim_entities() {
        let mut before=Drawing::new_metric();
        let h=before.add(&Space::Model,Common::default(),EntityKind::Line(Line{a:Vec3::ZERO,b:Vec3::new(1.0,0.0,0.0)})).unwrap();
        let mut after=before.clone();
        after.modify_entity(h,|e|if let EntityKind::Line(l)=&mut e.kind { l.b.x=2.0; }).unwrap();
        assert_eq!(reconcile_drawing(&before,&mut after),0);
        assert!(payload_index(&after.entity(h).unwrap().common).is_none());
    }

    #[test]
    fn runtime_does_not_double_bump_insert_when_command_already_bumped_it() {
        let mut before=Drawing::new_metric();
        let mut block=cadcraft_doc::Block::new("B");
        block.entities.push(Entity{handle:Handle(0x10),common:Common::default(),kind:EntityKind::Line(Line{a:Vec3::ZERO,b:Vec3::new(1.0,0.0,0.0)})});
        before.blocks.insert("B".into(),Arc::new(block));
        let h=before.add(&Space::Model,common(Handle(1),4),EntityKind::Insert(cadcraft_doc::Insert{
            block:"B".into(),insert:Vec3::ZERO,scale:Vec3::new(1.0,1.0,1.0),rotation:0.0,attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0,
        })).unwrap();
        let mut after=before.clone();
        if let Some(b)=after.blocks.get_mut("B") {
            Arc::make_mut(b).entities.modify(Handle(0x10),|e|if let EntityKind::Line(l)=&mut e.kind { l.b.x=2.0; });
        }
        after.modify_entity(h,|e|set_identity_revision(&mut e.common,h,5)).unwrap();
        reconcile_drawing(&before,&mut after);
        assert_eq!(revision(&after.entity(h).unwrap().common),Some(5));
    }

    #[test]
    fn runtime_bumps_insert_when_referenced_block_definition_changes() {
        let mut before=Drawing::new_metric();
        let mut block=cadcraft_doc::Block::new("B");
        block.entities.push(Entity{handle:Handle(0x10),common:Common::default(),kind:EntityKind::Line(Line{a:Vec3::ZERO,b:Vec3::new(1.0,0.0,0.0)})});
        before.blocks.insert("B".into(),Arc::new(block));
        let h=before.add(&Space::Model,common(Handle(1),4),EntityKind::Insert(cadcraft_doc::Insert{
            block:"B".into(),insert:Vec3::ZERO,scale:Vec3::new(1.0,1.0,1.0),rotation:0.0,attribs:Vec::new(),cols:1,rows:1,col_spacing:0.0,row_spacing:0.0,
        })).unwrap();
        let mut after=before.clone();
        if let Some(b)=after.blocks.get_mut("B") {
            Arc::make_mut(b).entities.modify(Handle(0x10),|e|if let EntityKind::Line(l)=&mut e.kind { l.b.x=2.0; });
        }
        reconcile_drawing(&before,&mut after);
        assert_eq!(revision(&after.entity(h).unwrap().common),Some(5));
    }

    #[test]
    fn runtime_applies_same_rule_to_model_layout_and_block_entities() {
        let h1=Handle(1);
        let h2=Handle(2);
        let h3=Handle(3);
        let mut before=Drawing::new_metric();
        before.add(&Space::Model,common(h1,1),EntityKind::Line(Line{a:Vec3::ZERO,b:Vec3::new(1.0,0.0,0.0)})).unwrap();
        before.add(&Space::Paper("Layout1".into()),common(h2,1),EntityKind::Line(Line{a:Vec3::ZERO,b:Vec3::new(1.0,0.0,0.0)})).unwrap();
        let mut block=cadcraft_doc::Block::new("B");
        block.entities.push(Entity{handle:h3,common:common(h3,1),kind:EntityKind::Line(Line{a:Vec3::ZERO,b:Vec3::new(1.0,0.0,0.0)})});
        before.blocks.insert("B".into(),Arc::new(block));

        let mut after=before.clone();
        after.modify_entity(h1,|e|e.common.layer="A".into()).unwrap();
        after.modify_entity(h2,|e|e.common.layer="A".into()).unwrap();
        if let Some(b)=after.blocks.get_mut("B") {
            Arc::make_mut(b).entities.modify(h3,|e|e.common.layer="A".into());
        }
        reconcile_drawing(&before,&mut after);
        assert_eq!(revision(&after.entity(h1).unwrap().common),Some(2));
        assert_eq!(revision(&after.entity(h2).unwrap().common),Some(2));
        assert_eq!(revision(&after.block("B").unwrap().entities.get(h3).unwrap().common),Some(2));
    }
}
