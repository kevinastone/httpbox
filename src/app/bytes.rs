use crate::app::random::rng;
use crate::headers::{ContentLength, HeaderMapExt};
use crate::http::{body_from_stream, Response, StatusCode};
use futures::prelude::*;
use gotham::helpers::http::response::create_response;
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use rand::Rng;
use serde_derive::Deserialize;
use std::iter::ExactSizeIterator;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct BytesPathParams {
    n: u32,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct BytesQueryParams {
    seed: Option<u32>,
    chunk_size: Option<usize>,
}

fn iter_bytes(state: &State) -> impl ExactSizeIterator<Item = u8> {
    let count = BytesPathParams::borrow_from(&state).n;
    let seed = BytesQueryParams::borrow_from(&state).seed;

    let mut rng = rng(seed);
    (0..count).map(move |_| rng.gen::<u8>())
}

pub fn bytes(state: State) -> (State, Response) {
    let data = iter_bytes(&state).collect::<Vec<u8>>();
    let res = create_response(
        &state,
        StatusCode::OK,
        mime::APPLICATION_OCTET_STREAM,
        data,
    );
    (state, res)
}

pub fn stream_bytes(state: State) -> (State, Response) {
    let chunk_size = BytesQueryParams::borrow_from(&state).chunk_size;
    let data = iter_bytes(&state);
    let content_length = data.len() as u64;

    let mut res = create_response(
        &state,
        StatusCode::OK,
        mime::APPLICATION_OCTET_STREAM,
        body_from_stream(stream::iter(data).chunks(chunk_size.unwrap_or(1))),
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
        assert_eq!(result_body, [236, 97, 38, 144])
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
        assert_eq!(result_body, [236, 97, 38, 144])
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
        assert_eq!(result_body, [236, 97, 38, 144])
    }
}
