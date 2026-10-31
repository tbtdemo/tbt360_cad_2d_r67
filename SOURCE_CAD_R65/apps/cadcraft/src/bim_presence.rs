//! Realtime collaboration presence and lease state without binding to a network stack.

#[derive(Clone, Debug, PartialEq)]
pub struct PresenceState {
    pub account_id: String,
    pub device_id: String,
    pub floor_id: String,
    pub discipline: String,
    pub selected_handle: Option<String>,
    pub cursor_world: Option<[f64;3]>,
    pub last_seen_ms: u64,
}

impl PresenceState {
    pub fn active_at(&self, now_ms:u64, timeout_ms:u64)->bool {
        now_ms.saturating_sub(self.last_seen_ms) <= timeout_ms
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lease {
    pub object_id:String,
    pub account_id:String,
    pub device_id:String,
    pub acquired_revision:u64,
    pub expires_ms:u64,
}

impl Lease {
    pub fn expired_at(&self,now_ms:u64)->bool{now_ms>=self.expires_ms}
    pub fn can_renew(&self,account_id:&str,device_id:&str)->bool{
        self.account_id==account_id && self.device_id==device_id
    }
}

#[derive(Default)]
pub struct PresenceRegistry {
    pub users: Vec<PresenceState>,
    pub leases: Vec<Lease>,
}

impl PresenceRegistry {
    pub fn active_users(&self,now_ms:u64,timeout_ms:u64)->Vec<&PresenceState>{
        self.users.iter().filter(|p|p.active_at(now_ms,timeout_ms)).collect()
    }

    pub fn renew(&mut self,object_id:&str,account_id:&str,device_id:&str,new_expires_ms:u64)->bool{
        let Some(lease)=self.leases.iter_mut().find(|l|l.object_id==object_id) else{return false};
        if !lease.can_renew(account_id,device_id){return false;}
        lease.expires_ms=new_expires_ms;
        true
    }

    pub fn release(&mut self,object_id:&str,account_id:&str,device_id:&str)->bool{
        let before=self.leases.len();
        self.leases.retain(|l|!(l.object_id==object_id && l.account_id==account_id && l.device_id==device_id));
        self.leases.len()!=before
    }

    pub fn heartbeat(&mut self, p:PresenceState){
        if let Some(existing)=self.users.iter_mut().find(|x|x.account_id==p.account_id&&x.device_id==p.device_id){*existing=p;}else{self.users.push(p);}
    }
    pub fn prune(&mut self,now_ms:u64,timeout_ms:u64){
        self.users.retain(|p|p.active_at(now_ms,timeout_ms));
        self.leases.retain(|l|!l.expired_at(now_ms));
    }
    pub fn acquire(&mut self, lease:Lease, now_ms:u64)->bool{
        self.prune(now_ms,30_000);
        if self.leases.iter().any(|l|l.object_id==lease.object_id && !l.expired_at(now_ms)){return false;}
        self.leases.push(lease); true
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn only_lease_owner_can_renew_or_release() {
        let mut r=PresenceRegistry::default();
        assert!(r.acquire(Lease{object_id:"A".into(),account_id:"u1".into(),device_id:"d1".into(),acquired_revision:1,expires_ms:200},100));
        assert!(!r.renew("A","u2","d2",300));
        assert!(r.renew("A","u1","d1",300));
        assert!(!r.release("A","u2","d2"));
        assert!(r.release("A","u1","d1"));
    }

    #[test]
    fn presence_expires_and_abandoned_lease_can_be_reacquired(){
        let mut r=PresenceRegistry::default();
        r.heartbeat(PresenceState{account_id:"u1".into(),device_id:"d1".into(),floor_id:"F1".into(),discipline:"ARCH".into(),selected_handle:None,cursor_world:None,last_seen_ms:100});
        assert!(r.acquire(Lease{object_id:"A".into(),account_id:"u1".into(),device_id:"d1".into(),acquired_revision:1,expires_ms:200},100));
        assert!(!r.acquire(Lease{object_id:"A".into(),account_id:"u2".into(),device_id:"d2".into(),acquired_revision:1,expires_ms:300},150));
        r.prune(250,100);
        assert!(r.acquire(Lease{object_id:"A".into(),account_id:"u2".into(),device_id:"d2".into(),acquired_revision:1,expires_ms:400},250));
    }
}
