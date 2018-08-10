extern crate byteorder;
extern crate rand;

use byteorder::{LittleEndian, WriteBytesExt};
use rand::prelude::*;
use rand::prng::chacha::ChaChaRng;

const SEED_WORDS: usize = 8;

fn to_bytes(val: u32) -> [u8; SEED_WORDS * 4] {
    let mut slice = vec![];
    slice.write_u32::<LittleEndian>(val).unwrap();
    let slice = &slice[..];

    let mut array = [0; SEED_WORDS * 4];
    for (begin, end) in (0..SEED_WORDS).zip(1..=SEED_WORDS) {
        array[begin * 4..end * 4].copy_from_slice(slice)
    }
    array
}

pub fn rng(seed: Option<u32>) -> ChaChaRng {
    match seed {
        Some(seed) => ChaChaRng::from_seed(to_bytes(seed)),
        None => ChaChaRng::from_rng(thread_rng()).unwrap(),
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
        assert_eq!(rng.next_u32(), 2202577813u32);
        assert_eq!(rng.next_u32(), 260684152u32);
        assert_eq!(rng.next_u32(), 3056137228u32);
        assert_eq!(rng.next_u32(), 1845999327u32);
    }
}
