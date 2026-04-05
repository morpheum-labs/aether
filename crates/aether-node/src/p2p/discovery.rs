use aether_common::types::JobSpec;

/// Placeholder for L1 event polling / gossip subscription.
pub trait JobSource {
    fn next_job(&mut self) -> Option<JobSpec>;
}
