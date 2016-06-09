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

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::super::app;
    use iron::Headers;
    use self::iron_test::{request, response};

    #[test]
    fn test_bytes() {

        let app = app();

        let res = request::get("http://localhost:3000/bytes/4?seed=1234",
                               Headers::new(),
                               &app)
            .unwrap();

        let result_body = response::extract_body_to_bytes(res);
        assert_eq!(result_body, [148, 214, 144, 210])
    }

    #[test]
    fn test_stream_bytes() {

        let app = app();

        let res = request::get("http://localhost:3000/stream-bytes/4?seed=1234",
                               Headers::new(),
                               &app)
            .unwrap();

        let result_body = response::extract_body_to_bytes(res);
        assert_eq!(result_body, [148, 214, 144, 210])
    }
}
