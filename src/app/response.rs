use gotham::helpers::http::response::create_response;
use gotham::state::State;
use http::header;
use hyper::{Body, Response, StatusCode};

pub fn empty_response(state: &State, status: StatusCode) -> Response<Body> {
    create_response(state, status, mime::TEXT_PLAIN, vec![])
}

pub fn bad_request(state: State) -> (State, Response<Body>) {
    let res = empty_response(&state, StatusCode::BAD_REQUEST);
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

pub fn internal_server_error(state: State) -> (State, Response<Body>) {
    let res = empty_response(&state, StatusCode::INTERNAL_SERVER_ERROR);
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

pub fn redirect_to(state: State, url: &str) -> (State, Response<Body>) {
    let mut res = empty_response(&state, StatusCode::FOUND);
    {
        let headers = res.headers_mut();
        headers.insert(
            header::LOCATION,
            header::HeaderValue::from_str(url).unwrap(),
        );
    }
    (state, res)
}
