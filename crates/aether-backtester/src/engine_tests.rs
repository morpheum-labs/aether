#[cfg(test)]
mod tests {
    use aether_common::traits::DeterministicEngine;
    use aether_common::types::{DataCommitment, ExecutionTier, JobSpec};

    use crate::VectorBacktestEngine;

    fn sample_job(id_byte: u8, seed: u64) -> JobSpec {
        JobSpec {
            id: [id_byte; 32],
            tier: ExecutionTier::Public,
            data_commitment: DataCommitment {
                merkle_root: [0u8; 32],
                version_cid: String::new(),
            },
            cargo_lock_hash: None,
            seed,
            initial_capital: "10000".into(),
            fee_bps: 10,
            slippage_bps: 5,
        }
    }

    fn monotone_up(n: usize) -> Vec<(String, String, String, String, String)> {
        (0..n)
            .map(|i| {
                let x = format!("{}", 100 + i);
                (x.clone(), x.clone(), x.clone(), x.clone(), "1".into())
            })
            .collect()
    }

    #[test]
    fn deterministic_repeatable() {
        let eng = VectorBacktestEngine;
        let spec = sample_job(9, 99);
        let bars = monotone_up(50);
        let a = eng.run_deterministic(&spec, &bars).unwrap();
        let b = eng.run_deterministic(&spec, &bars).unwrap();
        assert_eq!(a.result_digest, b.result_digest);
        assert_eq!(a.equity_curve, b.equity_curve);
    }
}
