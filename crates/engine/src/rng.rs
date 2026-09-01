pub fn hash_text(value: &str) -> u32 {
    let mut hash: i32 = 0x811c9dc5_u32 as i32;
    for character in value.chars() {
        hash ^= character as i32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7feb_352d);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x846c_a68b_u32 as i32);
    (hash ^ (hash >> 16)) as u32
}

pub struct DeterministicRandom {
    state: u32,
}

impl DeterministicRandom {
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x6d2b_79f5 } else { seed },
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6d2b_79f5);
        let mut value = self.state as i32;
        value = imul(value ^ (value >> 15), value | 1);
        value ^= value.wrapping_add(imul(value ^ (value >> 7), value | 61));
        f64::from((value ^ (value >> 14)) as u32) / 4_294_967_296.0
    }

    pub fn integer(&mut self, max_exclusive: u32) -> u32 {
        (self.next() * f64::from(max_exclusive)).floor() as u32
    }

    pub fn pick<'a, T>(&mut self, values: &'a [T]) -> &'a T {
        &values[self.integer(values.len() as u32) as usize]
    }

    pub fn shuffle<T: Copy>(&mut self, values: &[T]) -> Vec<T> {
        let mut shuffled = values.to_vec();
        let mut index = shuffled.len();
        while index > 1 {
            index -= 1;
            let other = self.integer((index + 1) as u32) as usize;
            shuffled.swap(index, other);
        }
        shuffled
    }
}

fn imul(left: i32, right: i32) -> i32 {
    left.wrapping_mul(right)
}

#[cfg(test)]
mod tests {
    use super::{hash_text, DeterministicRandom};

    #[test]
    fn hash_is_stable() {
        assert_eq!(hash_text("string:level-004"), hash_text("string:level-004"));
        assert_ne!(
            hash_text("string:level-004"),
            hash_text("pocket-circuit\0string:level-004")
        );
    }

    #[test]
    fn random_stream_is_deterministic() {
        let mut first = DeterministicRandom::new(42);
        let mut second = DeterministicRandom::new(42);
        for _ in 0..16 {
            assert_eq!(first.next(), second.next());
        }
    }
}
