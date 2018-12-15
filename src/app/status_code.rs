use crate::app::response::empty_response;
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use hyper::{Body, Response, StatusCode};
use serde_derive::Deserialize;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct StatusCodeParams {
    code: u16,
}

pub fn status_code(state: State) -> (State, Response<Body>) {
    let params = StatusCodeParams::borrow_from(&state);

    let res = empty_response(
        &state,
        try_or_error_response!(state, StatusCode::from_u16(params.code)),
    );
    (state, res)
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn test_status_code() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/status/429")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_bad_status_code() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/status/999")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
