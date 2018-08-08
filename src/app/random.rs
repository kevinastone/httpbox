extern crate byteorder;
extern crate rand;

use byteorder::{LittleEndian, WriteBytesExt};
use rand::prelude::*;
use std::iter::repeat;


fn to_bytes(val: u32) -> [u8; 32] {

    let mut slice = vec![];
    slice.write_u32::<LittleEndian>(val).unwrap();
    let slice = &slice[..];
    let mut array = [0; 32];
    array.copy_from_slice(
        &repeat(slice).take(8).collect::<Vec<&[u8]>>().concat()
    );
    array
}

pub fn rng(seed: Option<u32>) -> StdRng {
    let seed = seed.unwrap_or_else(|| thread_rng().gen::<u32>());
    StdRng::from_seed(to_bytes(seed))
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
        assert_eq!(rng.next_u32(), 956056973u32);
        assert_eq!(rng.next_u32(), 667675964u32);
        assert_eq!(rng.next_u32(), 1063033695u32);
        assert_eq!(rng.next_u32(), 1062349892u32);
    }
}
