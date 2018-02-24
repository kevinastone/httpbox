extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate serde;

use app::response::bad_request;
use gotham::http::response::create_response;
use gotham::state::{FromState, State};

use hyper::{Response, StatusCode};

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct StatusCodeParams {
    code: u16,
}

pub fn status_code(mut state: State) -> (State, Response) {
    let params = StatusCodeParams::take_from(&mut state);

    match StatusCode::try_from(params.code) {
        Ok(status) => {
            let res = create_response(
                &state,
                status,
                Some((vec![], mime::TEXT_PLAIN)),
            );
            (state, res)
        }
        Err(_) => bad_request(state),
    }
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

        assert_eq!(response.status(), StatusCode::TooManyRequests);
    }
}
