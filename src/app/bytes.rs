extern crate iron;
extern crate rand;
extern crate router;
extern crate urlencoded;

use self::iron::{Request, Response, IronResult};
use self::iron::Plugin;
use self::iron::status;
use self::router::Router;
use self::rand::{Rng, XorShiftRng};
use self::urlencoded::{UrlEncodedQuery, QueryMap};

pub const SEED_QUERY_PARAM: &'static str = "seed";

fn to_bytes(val: u32) -> [u32; 4] {
    [val, val, val, val]
}

pub fn seed(hashmap: Option<&QueryMap>) -> Option<u32> {
    hashmap
    .and_then(|hashmap| hashmap.get(SEED_QUERY_PARAM))
    .and_then(|vals| vals.first())
    .and_then(|val| val.parse::<u32>().ok())
}

pub fn rng(seed: Option<u32>) -> XorShiftRng {
    let seed = seed.unwrap_or_else(|| rand::thread_rng().gen::<u32>());
    rand::SeedableRng::from_seed(to_bytes(seed))
}

pub fn bytes(req: &mut Request) -> IronResult<Response> {

    let count = itry!(
        req.extensions.get::<Router>().unwrap().find("n").unwrap_or("1024").parse::<u32>(),
        status::BadRequest
    );

    let seed_param = seed(req.get_ref::<UrlEncodedQuery>().ok());

    let mut rng = rng(seed_param);
    let bytes = (0..count).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>();

    Ok(Response::with((status::Ok, bytes)))
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
        use super::super::rand::{Rng};

        #[test]
        fn rng_no_seed() {

            rng(None);
        }        

        #[test]
        fn rng_seed_consistent() {

            let mut rng = rng(Some(1234));
            assert_eq!(rng.next_u32(), 2537108u32);
            assert_eq!(rng.next_u32(), 1238u32);
            assert_eq!(rng.next_u32(), 2537104u32);
            assert_eq!(rng.next_u32(), 1234u32);
        }        
    }
}
