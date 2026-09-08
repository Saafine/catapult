//! Tiny deterministic PRNG (xorshift64*). Seeded runs replay identically,
//! which matters for tests, replays, and later for any daily-challenge or
//! multiplayer feature where both sides must see the same world.

#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // Avoid the all-zero state.
        Rng(seed ^ 0x2545_F491_4F6C_DD1D | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }

    /// Inclusive integer range.
    pub fn range_i(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.unit() * (hi - lo + 1) as f32) as i32
    }
}
