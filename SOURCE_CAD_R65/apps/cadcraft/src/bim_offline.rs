//! Offline-first BIM synchronization state machine.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Connectivity {
    #[default]
    Offline,
    Connecting,
    Online,
    Degraded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QueueState {
    #[default]
    Local,
    Pending,
    Synced,
    Conflict,
    Rejected,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyncCheckpoint {
    pub remote_project_revision: u64,
    pub last_pulled_revision: u64,
    pub local_sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfflineQueueItem {
    pub local_sequence: u64,
    pub change_set_id: String,
    pub state: QueueState,
    pub retry_count: u32,
    pub last_error: Option<String>,
}

#[derive(Default)]
pub struct OfflineSyncState {
    pub connectivity: Connectivity,
    pub checkpoint: SyncCheckpoint,
    pub queue: Vec<OfflineQueueItem>,
}

impl OfflineSyncState {
    pub fn enqueue(&mut self, change_set_id: String) -> u64 {
        self.checkpoint.local_sequence += 1;
        let seq = self.checkpoint.local_sequence;
        self.queue.push(OfflineQueueItem { local_sequence: seq, change_set_id, state: QueueState::Local, retry_count: 0, last_error: None });
        seq
    }

    pub fn begin_reconnect(&mut self) {
        self.connectivity = Connectivity::Connecting;
        for item in &mut self.queue {
            if item.state == QueueState::Local { item.state = QueueState::Pending; }
        }
    }

    pub fn mark_online(&mut self, remote_revision: u64) {
        self.connectivity = Connectivity::Online;
        self.checkpoint.remote_project_revision = remote_revision;
    }

    pub fn acknowledge_synced(&mut self, local_sequence: u64, remote_revision: u64) {
        if let Some(item) = self.queue.iter_mut().find(|i| i.local_sequence == local_sequence) {
            item.state = QueueState::Synced;
            item.last_error = None;
        }
        self.checkpoint.remote_project_revision = self.checkpoint.remote_project_revision.max(remote_revision);
    }

    pub fn retry_delay_seconds(&self, local_sequence: u64) -> Option<u64> {
        let item = self.queue.iter().find(|i| i.local_sequence == local_sequence)?;
        Some((1u64 << item.retry_count.min(6)).min(60))
    }

    pub fn mark_retry_error(&mut self, local_sequence: u64, message: String) {
        if let Some(item) = self.queue.iter_mut().find(|i| i.local_sequence == local_sequence) {
            item.retry_count = item.retry_count.saturating_add(1);
            item.state = QueueState::Pending;
            item.last_error = Some(message);
        }
        self.connectivity = Connectivity::Degraded;
    }

    pub fn mark_conflict(&mut self, local_sequence: u64, message: String) {
        if let Some(item) = self.queue.iter_mut().find(|i| i.local_sequence == local_sequence) {
            item.state = QueueState::Conflict;
            item.last_error = Some(message);
        }
    }

    pub fn pending_count(&self) -> usize {
        self.queue.iter().filter(|i| !matches!(i.state, QueueState::Synced | QueueState::Rejected)).count()
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_backoff_is_bounded_and_acknowledgement_advances_checkpoint() {
        let mut s=OfflineSyncState::default();
        let seq=s.enqueue("c1".into());
        s.mark_retry_error(seq,"network".into());
        assert_eq!(s.retry_delay_seconds(seq),Some(2));
        for _ in 0..10 { s.mark_retry_error(seq,"network".into()); }
        assert_eq!(s.retry_delay_seconds(seq),Some(60));
        s.acknowledge_synced(seq,12);
        assert_eq!(s.checkpoint.remote_project_revision,12);
        assert_eq!(s.pending_count(),0);
    }

    #[test]
    fn offline_changes_move_to_pending_on_reconnect() {
        let mut s = OfflineSyncState::default();
        s.enqueue("c1".into());
        s.enqueue("c2".into());
        assert_eq!(s.pending_count(), 2);
        s.begin_reconnect();
        assert!(s.queue.iter().all(|i| i.state == QueueState::Pending));
        s.mark_online(8);
        assert_eq!(s.connectivity, Connectivity::Online);
        assert_eq!(s.checkpoint.remote_project_revision, 8);
    }
}
