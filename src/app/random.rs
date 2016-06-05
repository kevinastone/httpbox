extern crate rand;
extern crate urlencoded;

use self::rand::{Rng, XorShiftRng};
use self::urlencoded::QueryMap;


pub const SEED_QUERY_PARAM: &'static str = "seed";

fn to_bytes(val: u32) -> [u32; 4] {
    [val, val, val, val]
}

pub struct RandomGenerator {
    rng: XorShiftRng,
}

impl RandomGenerator {
    pub fn new(seed: Option<u32>) -> Self {
        let seed = seed.unwrap_or_else(|| rand::thread_rng().gen::<u32>());
        RandomGenerator { rng: rand::SeedableRng::from_seed(to_bytes(seed)) }
    }
}

impl Rng for RandomGenerator {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }
}

pub fn seed(hashmap: Option<&QueryMap>) -> Option<u32> {
    hashmap.and_then(|hashmap| hashmap.get(SEED_QUERY_PARAM))
        .and_then(|vals| vals.first())
        .and_then(|val| val.parse::<u32>().ok())
}

#[cfg(test)]
mod tests {

    mod seed {

        use super::super::*;
        use super::super::urlencoded::QueryMap;

        #[test]
        fn parse_seed_missing() {
            let query = QueryMap::new();

            assert_eq!(seed(Some(&query)), None)
        }

        #[test]
        fn parse_seed_empty() {
            let mut query = QueryMap::new();
            query.insert(String::from(SEED_QUERY_PARAM), vec![]);

            assert_eq!(seed(Some(&query)), None)
        }

        #[test]
        fn parse_seed_invalid() {
            let mut query = QueryMap::new();
            query.insert(String::from(SEED_QUERY_PARAM), vec![String::from("abcd")]);

            assert_eq!(seed(Some(&query)), None)
        }

        #[test]
        fn parse_seed_valid() {
            let mut query = QueryMap::new();
            query.insert(String::from(SEED_QUERY_PARAM), vec![String::from("1234")]);

            assert_eq!(seed(Some(&query)), Some(1234))
        }

    }

    mod rng {
        use super::super::*;
        use super::super::rand::Rng;

        #[test]
        fn rng_no_seed() {

            RandomGenerator::new(None);
        }

        #[test]
        fn rng_seed_consistent() {

            let mut rng = RandomGenerator::new(Some(1234));
            assert_eq!(rng.next_u32(), 2537108u32);
            assert_eq!(rng.next_u32(), 1238u32);
            assert_eq!(rng.next_u32(), 2537104u32);
            assert_eq!(rng.next_u32(), 1234u32);
        }
    }
}
