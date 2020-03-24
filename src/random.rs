use byteorder::{LittleEndian, WriteBytesExt};
use rand::prelude::*;
use rand::rngs::SmallRng as Rng;

const SEED_WORDS: usize = 4;

fn to_bytes(val: u32) -> <Rng as SeedableRng>::Seed {
    let mut slice = vec![];
    slice.write_u32::<LittleEndian>(val).unwrap();
    let slice = &slice[..];

    let mut array = [0; SEED_WORDS * 4];
    for (begin, end) in (0..SEED_WORDS).zip(1..=SEED_WORDS) {
        array[begin * 4..end * 4].copy_from_slice(slice)
    }
    array
}

pub fn rng(seed: Option<u32>) -> Rng {
    match seed {
        Some(seed) => Rng::from_seed(to_bytes(seed)),
        None => Rng::from_rng(thread_rng()).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_no_seed() {
        rng(None);
    }

    #[test]
    fn rng_seed_consistent() {
        let mut rng = rng(Some(1234));
        assert_eq!(rng.next_u32(), 2468986604u32);
        assert_eq!(rng.next_u32(), 1283941473u32);
        assert_eq!(rng.next_u32(), 3396522534u32);
        assert_eq!(rng.next_u32(), 1785331600u32);
    }
}
