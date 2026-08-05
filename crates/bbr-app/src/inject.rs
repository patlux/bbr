//! Determinism injection points for the composition root.
//!
//! Phase 1 only needs stable *ids* (`--bbr-ids`): the profile id and the web
//! view id are used to key engine state, and E2E fixtures must be able to
//! rely on fixed ids. The `TimeSource` seam is deliberately **not** added
//! yet — there is no time-based logic in Phase 1 (debounces land in
//! Phase 5/7); adding the trait now would be speculative abstraction.

/// Stable identifier generator.
pub trait IdGen {
    /// Next `u64` in the sequence.
    fn next_u64(&mut self) -> u64;
}

/// Values derived from a monotone counter: `1, 2, 3, …`. Used by E2E so
/// ids in snapshots and events are deterministic.
#[derive(Debug, Default)]
pub struct SequentialIdGen(u64);

impl IdGen for SequentialIdGen {
    fn next_u64(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }
}

/// Values derived from the system's randomness.
#[derive(Debug)]
pub struct RandomIdGen {
    next: u64,
    counter: u64,
}

impl Default for RandomIdGen {
    fn default() -> Self {
        let mut bytes = [0u8; 8];
        let mut mix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9e37_79b9_7f4a_7c15);
        // Cheap splitmix64-style mix so two launches in the same nanosecond
        // don't collide on the first id.
        for b in bytes.iter_mut() {
            mix ^= mix >> 30;
            mix = mix.wrapping_mul(0xbf58_476d_1ce4_e5b9);
            mix ^= mix >> 27;
            mix = mix.wrapping_mul(0x94d0_49bb_1331_11eb);
            mix ^= mix >> 31;
            *b = mix as u8;
        }
        RandomIdGen {
            next: u64::from_le_bytes(bytes),
            counter: 0,
        }
    }
}

impl IdGen for RandomIdGen {
    fn next_u64(&mut self) -> u64 {
        self.counter = self.counter.wrapping_add(1);
        self.next
            .wrapping_add(self.counter.wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequential_ids_are_1_based_and_strictly_increasing() {
        let mut gen = SequentialIdGen::default();
        assert_eq!(gen.next_u64(), 1);
        assert_eq!(gen.next_u64(), 2);
        assert_eq!(gen.next_u64(), 3);
    }

    #[test]
    fn random_ids_do_not_collide_in_the_first_1000() {
        let mut gen = RandomIdGen::default();
        let first = gen.next_u64();
        let mut seen = std::collections::HashSet::new();
        seen.insert(first);
        for _ in 0..1000 {
            let id = gen.next_u64();
            assert!(seen.insert(id), "collision in random gen");
        }
    }
}
