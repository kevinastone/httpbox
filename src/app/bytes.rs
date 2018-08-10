extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate rand;

use app::random::rng;
use gotham::http::response::{create_response, set_headers};
use gotham::state::{FromState, State};
use hyper::{Body, Response, StatusCode};
use rand::Rng;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct BytesPathParams {
    n: u32,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct BytesQueryParams {
    seed: Option<u32>,
    #[allow(dead_code)]
    chunk_size: Option<usize>,
}

fn get_bytes(mut state: &State) -> Vec<u8> {
    let count = BytesPathParams::borrow_from(&mut state).n;
    let seed = BytesQueryParams::borrow_from(&mut state).seed;

    let mut rng = rng(seed);
    (0..count).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>()
}

pub fn bytes(mut state: State) -> (State, Response) {
    let data = get_bytes(&mut state);
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((data, mime::APPLICATION_OCTET_STREAM)),
    );
    (state, res)
}

pub fn stream_bytes(mut state: State) -> (State, Response) {
    let data = get_bytes(&mut state);

    let content_length = data.len() as u64;

    let mut res = Response::new();
    res.set_status(StatusCode::Ok);
    set_headers(
        &mut state,
        &mut res,
        Some(mime::APPLICATION_OCTET_STREAM),
        Some(content_length),
    );
    res.set_body(Body::from(data));
    (state, res)
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn test_bytes() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/bytes/4?seed=1234")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_body().unwrap();
        assert_eq!(result_body, [149, 120, 12, 223])
    }

    #[test]
    fn test_stream_bytes() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/stream-bytes/4?seed=1234")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_body().unwrap();
        assert_eq!(result_body, [149, 120, 12, 223])
    }

    #[test]
    fn test_stream_bytes_with_chunk_size() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/stream-bytes/4?seed=1234&chunk_size=2")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_body().unwrap();
        assert_eq!(result_body, [149, 120, 12, 223])
    }
}
