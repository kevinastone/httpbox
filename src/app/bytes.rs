extern crate iron;
extern crate rand;
extern crate router;
extern crate urlencoded;

use self::iron::{Request, Response, IronResult};
use self::iron::Plugin;
use self::iron::status;
use self::rand::Rng;
use self::router::Router;
use self::urlencoded::UrlEncodedQuery;
use self::super::stream::StreamResponse;
use self::super::random::{RandomGenerator, seed};


pub fn bytes(req: &mut Request) -> IronResult<Response> {

    let count =
        itry!(req.extensions.get::<Router>().unwrap().find("n").unwrap_or("1024").parse::<u32>(),
              status::BadRequest);

    let seed_param = seed(req.get_ref::<UrlEncodedQuery>().ok());

    let mut rng = RandomGenerator::new(seed_param);
    let bytes = (0..count).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>();

    Ok(Response::with((status::Ok, bytes)))
}

pub fn stream_bytes(req: &mut Request) -> IronResult<Response> {

    let count =
        itry!(req.extensions.get::<Router>().unwrap().find("n").unwrap_or("1024").parse::<u32>(),
              status::BadRequest);

    let seed_param = seed(req.get_ref::<UrlEncodedQuery>().ok());

    let mut rng = RandomGenerator::new(seed_param);

    let bytes = (0..count).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>();

    let reader = StreamResponse::new(bytes);
    Ok(Response::with((status::Ok, reader)))
}
