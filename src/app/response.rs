extern crate gotham;
extern crate hyper;
extern crate mime;

use gotham::http::response::create_response;
use gotham::state::State;
use hyper::{header, Response, StatusCode};

pub fn bad_request(state: State) -> (State, Response) {
    let res = create_response(
        &state,
        StatusCode::BadRequest,
        Some((vec![], mime::TEXT_PLAIN)),
    );
    (state, res)
}

pub fn internal_server_error(state: State) -> (State, Response) {
    let res = create_response(
        &state,
        StatusCode::InternalServerError,
        Some((vec![], mime::TEXT_PLAIN)),
    );
    (state, res)
}

pub fn ok<B>(state: State, body: B) -> (State, Response)
where
    B: Into<Vec<u8>>,
{
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((body.into(), mime::TEXT_PLAIN)),
    );
    (state, res)
}

pub fn redirect_to(state: State, url: String) -> (State, Response) {
    let mut res = create_response(
        &state,
        StatusCode::Found,
        Some((vec![], mime::TEXT_PLAIN)),
    );
    {
        let headers = res.headers_mut();
        headers.set(header::Location::new(url));
    }
    (state, res)
}
