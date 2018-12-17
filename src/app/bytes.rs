use crate::app::random::rng;
use crate::headers::{ContentLength, HeaderMapExt};
use gotham::helpers::http::response::create_response;
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use hyper::{Body, Response, StatusCode};
use rand::Rng;
use serde_derive::Deserialize;

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

fn get_bytes(state: &State) -> Vec<u8> {
    let count = BytesPathParams::borrow_from(&state).n;
    let seed = BytesQueryParams::borrow_from(&state).seed;

    let mut rng = rng(seed);
    (0..count).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>()
}

pub fn bytes(state: State) -> (State, Response<Body>) {
    let data = get_bytes(&state);
    let res = create_response(
        &state,
        StatusCode::OK,
        mime::APPLICATION_OCTET_STREAM,
        data,
    );
    (state, res)
}

pub fn stream_bytes(state: State) -> (State, Response<Body>) {
    let data = get_bytes(&state);

    let content_length = data.len() as u64;

    let mut res = create_response(
        &state,
        StatusCode::OK,
        mime::APPLICATION_OCTET_STREAM,
        Body::from(data),
    );

    res.headers_mut()
        .typed_insert(ContentLength(content_length));
    (state, res)
}

#[cfg(test)]
mod test {
    use crate::app::app;
    use gotham::test::TestServer;
    use http::StatusCode;

    #[test]
    fn test_bytes() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/bytes/4?seed=1234")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_body().unwrap();
        assert_eq!(result_body, [149, 120, 12, 223])
    }

    #[test]
    fn test_stream_bytes() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/stream-bytes/4?seed=1234")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_body().unwrap();
        assert_eq!(result_body, [149, 120, 12, 223])
    }

    #[test]
    fn test_stream_bytes_with_chunk_size() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/stream-bytes/4?seed=1234&chunk_size=2")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_body().unwrap();
        assert_eq!(result_body, [149, 120, 12, 223])
    }
}
