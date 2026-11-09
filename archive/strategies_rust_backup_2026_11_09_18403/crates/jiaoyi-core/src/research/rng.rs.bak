//! SplitMix64 (Steele, Lea & Flood 2014; the reference code by S. Vigna):
//! a small, fully specified generator, so a logged seed reproduces every draw
//! on any machine and any crate version.

#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform integer in `0..n` (`n` > 0), without modulo bias: draws
    /// falling in the incomplete last block are rejected.
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "below(0)");
        let zone = u64::MAX - (u64::MAX % n) - 1;
        loop {
            let v = self.next_u64();
            if v <= zone {
                return v % n;
            }
        }
    }

    /// A fair coin.
    pub fn coin(&mut self) -> bool {
        self.next_u64() >> 63 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_reference_outputs() {
        // Computed independently in Python from the reference algorithm.
        let mut r = SplitMix64::new(0);
        assert_eq!(r.next_u64(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(r.next_u64(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(r.next_u64(), 0x06c4_5d18_8009_454f);
        let mut r = SplitMix64::new(20_261_024);
        assert_eq!(r.next_u64(), 0xdd1e_cd57_d727_dd20);
    }

    #[test]
    fn below_stays_in_range_and_covers_it() {
        let mut r = SplitMix64::new(7);
        let mut seen = [0u32; 6];
        for _ in 0..6000 {
            seen[r.below(6) as usize] += 1;
        }
        assert!(seen.iter().all(|&c| (800..1200).contains(&c)), "{seen:?}");
    }
}
