use rand::{rngs::SmallRng, Rng as _, SeedableRng};

// Used for compatibility reasons
#[derive(Debug)]
pub(crate) struct Rng {
    rng: SmallRng,
}

impl Rng {
    pub(crate) fn random_u8_4(&mut self) -> [u8; 4] {
        self.rng.gen()
    }

    pub(crate) fn _random_u8_16(&mut self) -> [u8; 16] {
        self.rng.gen()
    }
}

impl Default for Rng {
    #[inline]
    fn default() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
        }
    }
}
