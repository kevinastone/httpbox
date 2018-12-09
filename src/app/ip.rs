use crate::app::response::{internal_server_error, ok};
use gotham::state::{client_addr, FromState, State};

use hyper::{Body, HeaderMap, Response};

pub const X_FORWARD_FOR: &str = "X-Forwarded-For";

fn client_ip_addr(state: &State) -> Option<String> {
    client_addr(&state).map(|a| a.ip().to_string())
}

pub fn ip(state: State) -> (State, Response<Body>) {
    let remote_ip = expect_or_error_response!(
        internal_server_error,
        state,
        HeaderMap::borrow_from(&state)
            .get(X_FORWARD_FOR)
            .and_then(|h| h.to_str().ok().map(String::from))
            .or_else(|| client_ip_addr(&state))
    );
    ok(state, remote_ip)
}

#[cfg(test)]
mod test {
    use super::super::router;
    use super::X_FORWARD_FOR;

    use gotham::test::TestServer;
    use http::header;
    use hyper::StatusCode;

    #[test]
    fn test_ip() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/ip")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "127.0.0.1")
    }

    #[test]
    fn test_ip_from_header() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/ip")
            .with_header(
                X_FORWARD_FOR,
                header::HeaderValue::from_static("1.2.3.4"),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "1.2.3.4")
    }
}
