use rand::make_rng;
use rand::prelude::*;

fn to_bytes(val: u32) -> <SmallRng as SeedableRng>::Seed {
    let slice = val.to_le_bytes();

    let mut seed = <SmallRng as SeedableRng>::Seed::default();
    for chunk in seed.as_mut().chunks_mut(4) {
        chunk.copy_from_slice(&slice);
    }
    seed
}

pub fn rng(seed: Option<u32>) -> SmallRng {
    match seed {
        Some(seed) => SmallRng::from_seed(to_bytes(seed)),
        None => make_rng(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn rng_no_seed() {
        rng(None);
    }

    #[test]
    fn rng_seed_consistent() {
        let mut rng = rng(Some(1234));
        assert_eq!(rng.next_u32(), 3523216598u32);
        assert_eq!(rng.next_u32(), 1761608916u32);
        assert_eq!(rng.next_u32(), 19744u32);
        assert_eq!(rng.next_u32(), 2636570400u32);
    }
}
