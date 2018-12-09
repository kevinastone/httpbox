use crate::app::response::ok;
use gotham::state::{FromState, State};

use hyper::{Body, HeaderMap, Response};

pub fn user_agent(state: State) -> (State, Response<Body>) {
    let user_agent = expect_or_error_response!(
        state,
        HeaderMap::borrow_from(&state)
            .get(http::header::USER_AGENT)
            .and_then(|hv| hv.to_str().ok())
            .map(String::from)
    );
    ok(state, user_agent)
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn test_user_agent() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/user-agent")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_user_agent_custom() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/user-agent")
            .with_header(
                http::header::USER_AGENT,
                http::header::HeaderValue::from_static("HTTPBoxBot/1.0"),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "HTTPBoxBot/1.0");
    }
}
