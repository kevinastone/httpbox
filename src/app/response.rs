extern crate gotham;
extern crate hyper;
extern crate mime;

use gotham::http::response::create_response;
use gotham::state::State;
use hyper::{header, Response, StatusCode};

pub fn empty_response(state: &State, status: StatusCode) -> Response {
    create_response(state, status, Some((vec![], mime::TEXT_PLAIN)))
}

pub fn bad_request(state: State) -> (State, Response) {
    let res = empty_response(&state, StatusCode::BadRequest);
    (state, res)
}

pub fn internal_server_error(state: State) -> (State, Response) {
    let res = empty_response(&state, StatusCode::InternalServerError);
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
    let mut res = empty_response(&state, StatusCode::Found);
    {
        let headers = res.headers_mut();
        headers.set(header::Location::new(url));
    }
    (state, res)
}
