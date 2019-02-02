use crate::headers::{HeaderMapExt, Location};
use crate::http::{Body, Response, StatusCode, Uri};
use gotham::helpers::http::response::{create_empty_response, create_response};
use gotham::state::State;

pub fn empty_response(state: &State, status: StatusCode) -> Response {
    create_empty_response(state, status)
}

pub fn bad_request(state: State) -> (State, Response) {
    let res = empty_response(&state, StatusCode::BAD_REQUEST);
    (state, res)
}

pub fn internal_server_error(state: State) -> (State, Response) {
    let res = empty_response(&state, StatusCode::INTERNAL_SERVER_ERROR);
    (state, res)
}

pub fn html<B>(state: State, body: B) -> (State, Response)
where
    B: Into<Body>,
{
    let res =
        create_response(&state, StatusCode::OK, mime::TEXT_HTML, body.into());
    (state, res)
}

pub fn ok<B>(state: State, body: B) -> (State, Response)
where
    B: Into<Body>,
{
    let res =
        create_response(&state, StatusCode::OK, mime::TEXT_PLAIN, body.into());
    (state, res)
}

pub fn redirect_to(state: State, uri: Uri) -> (State, Response) {
    let mut res = empty_response(&state, StatusCode::FOUND);
    res.headers_mut().typed_insert(Location::from(uri));
    (state, res)
}
