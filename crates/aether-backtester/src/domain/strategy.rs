/// Parameters for the built-in template strategy (deterministic from job seed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrategyParams {
    pub fast_period: u32,
    pub slow_period: u32,
}

impl StrategyParams {
    pub fn from_seed(seed: u64) -> Self {
        let fast = 2 + (seed % 5) as u32;
        let slow = fast + 3 + ((seed >> 8) % 7) as u32;
        Self {
            fast_period: fast.max(2),
            slow_period: slow.max(fast + 1),
        }
    }
}
