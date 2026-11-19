use std::hash::{BuildHasherDefault, Hash, Hasher};

#[derive(Default, Clone, Copy)]
pub struct Fx(u64);

const SEED: u64 = 0x517c_c1b7_2722_0a95;

impl Hasher for Fx {
    fn write(&mut self, bytes: &[u8]) {
        let mut hash = self.0;
        for chunk in bytes.chunks(8) {
            let mut word = 0u64;
            for (shift, byte) in chunk.iter().enumerate() {
                word |= u64::from(*byte) << (shift * 8);
            }
            hash = (hash.rotate_left(5) ^ word).wrapping_mul(SEED);
        }
        self.0 = hash;
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = (self.0.rotate_left(5) ^ value).wrapping_mul(SEED);
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

pub(crate) type Build = BuildHasherDefault<Fx>;

pub(crate) fn hash(value: impl Hash) -> u64 {
    let mut hasher = Fx::default();
    value.hash(&mut hasher);
    hasher.finish()
}
