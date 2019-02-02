use crate::headers::{HeaderMapExt, Location};
use gotham::helpers::http::response::{create_empty_response, create_response};
use gotham::state::State;
use hyper::{Body, Response, StatusCode, Uri};

pub fn empty_response(state: &State, status: StatusCode) -> Response<Body> {
    create_empty_response(state, status)
}

pub fn bad_request(state: State) -> (State, Response<Body>) {
    let res = empty_response(&state, StatusCode::BAD_REQUEST);
    (state, res)
}

pub fn internal_server_error(state: State) -> (State, Response<Body>) {
    let res = empty_response(&state, StatusCode::INTERNAL_SERVER_ERROR);
    (state, res)
}

pub fn html<B>(state: State, body: B) -> (State, Response<Body>)
where
    B: Into<Body>,
{
    let res =
        create_response(&state, StatusCode::OK, mime::TEXT_HTML, body.into());
    (state, res)
}

pub fn ok<B>(state: State, body: B) -> (State, Response<Body>)
where
    B: Into<Body>,
{
    let res =
        create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, body.into());
    (state, res)
}

pub fn redirect_to(state: State, uri: Uri) -> (State, Response<Body>) {
    let mut res = empty_response(&state, StatusCode::FOUND);
    res.headers_mut().typed_insert(Location::from(uri));
    (state, res)
}
