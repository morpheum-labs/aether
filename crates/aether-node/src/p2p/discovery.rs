use std::collections::VecDeque;

use aether_common::types::JobSpec;

/// Placeholder for L1 event polling / gossip subscription.
pub trait JobSource {
    fn next_job(&mut self) -> Option<JobSpec>;
}

/// In-memory job queue for tests and local demos (Phase 0 — no network yet).
#[derive(Debug, Clone)]
pub struct VecJobSource {
    jobs: VecDeque<JobSpec>,
}

impl VecJobSource {
    #[must_use]
    pub fn new(jobs: impl IntoIterator<Item = JobSpec>) -> Self {
        Self {
            jobs: jobs.into_iter().collect(),
        }
    }
}

impl JobSource for VecJobSource {
    fn next_job(&mut self) -> Option<JobSpec> {
        self.jobs.pop_front()
    }
}
