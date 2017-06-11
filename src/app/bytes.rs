extern crate iron;
extern crate modifier;
extern crate rand;
extern crate router;
extern crate urlencoded;

use self::iron::{Request, Response, IronResult};
use self::iron::Plugin;
use self::iron::headers::ContentLength;
use self::iron::response::WriteBody;
use self::iron::status;
use self::modifier::Modifier;
use self::rand::Rng;
use self::router::Router;
use self::urlencoded::UrlEncodedQuery;
use self::super::random::RandomGenerator;
use super::util::parse_query_value;
use std::io::{self, Write};


pub const CHUNK_SIZE_QUERY_PARAM: &'static str = "chunk_size";
pub const SEED_QUERY_PARAM: &'static str = "seed";


pub struct ChunkedByteResponse {
    data: Vec<u8>,
    chunk_size: usize,
}

impl ChunkedByteResponse {
    pub fn new(data: Vec<u8>, chunk_size: usize) -> Self {
        ChunkedByteResponse {
            data: data,
            chunk_size: chunk_size,
        }
    }
}

impl WriteBody for ChunkedByteResponse {
    fn write_body(&mut self, res: &mut Write) -> io::Result<()> {

        for chunk in self.data.chunks(self.chunk_size) {
            res.write(chunk)?;
            res.flush()?;
        }

        Ok(())
    }
}

impl Modifier<Response> for ChunkedByteResponse {
    fn modify(self, res: &mut Response) {
        res.headers.set(ContentLength(self.data.len() as u64));
        res.body = Some(Box::new(self));
    }
}

fn get_bytes(req: &mut Request) -> IronResult<Vec<u8>> {

    let count =
        itry!(req.extensions.get::<Router>().unwrap().find("n").unwrap_or("1024").parse::<u32>(),
              status::BadRequest);

    let seed_param = parse_query_value(req.get_ref::<UrlEncodedQuery>().ok(), SEED_QUERY_PARAM);

    let mut rng = RandomGenerator::new(seed_param);

    Ok((0..count).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>())
}


pub fn bytes(req: &mut Request) -> IronResult<Response> {

    Ok(Response::with((status::Ok, get_bytes(req)?)))
}

pub fn stream_bytes(req: &mut Request) -> IronResult<Response> {

    let bytes = get_bytes(req)?;
    let chunk_size = parse_query_value(req.get_ref::<UrlEncodedQuery>().ok(),
                                       CHUNK_SIZE_QUERY_PARAM)
        .unwrap_or(1);

    let reader = ChunkedByteResponse::new(bytes, chunk_size);
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

    #[test]
    fn test_stream_bytes_with_chunk_size() {

        let app = app();

        let res = request::get("http://localhost:3000/stream-bytes/4?seed=1234&chunk_size=2",
                               Headers::new(),
                               &app)
            .unwrap();

        let result_body = response::extract_body_to_bytes(res);
        assert_eq!(result_body, [148, 214, 144, 210])
    }
}
