use crate::app::response::{internal_server_error, ok};
use crate::headers::{XForwardedFor, X_FORWARDED_FOR};
use gotham::state::{client_addr, FromState, State};
use hyper::{Body, HeaderMap, Response};
use std::net::IpAddr;

fn x_forward_for(state: &State) -> Option<IpAddr> {
    HeaderMap::borrow_from(&state)
        .get(X_FORWARDED_FOR)
        .and_then(|h| XForwardedFor::try_for(h).ok())
        .map(|h| h.ip_addr())
}

fn client_ip_addr(state: &State) -> Option<IpAddr> {
    client_addr(state).map(|a| a.ip())
}

pub fn ip(state: State) -> (State, Response<Body>) {
    let remote_ip = expect_or_error_response!(
        internal_server_error,
        state,
        x_forward_for(&state).or_else(|| client_ip_addr(&state))
    );
    ok(state, remote_ip.to_string())
}

#[cfg(test)]
mod test {
    use crate::app::app;
    use crate::headers::X_FORWARDED_FOR;

    use gotham::test::TestServer;
    use http::header;
    use http::StatusCode;

    #[test]
    fn test_ip() {
        let test_server = TestServer::new(app()).unwrap();
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
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/ip")
            .with_header(
                X_FORWARDED_FOR,
                header::HeaderValue::from_static("1.2.3.4"),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "1.2.3.4")
    }

    #[test]
    fn test_ip_ignore_bad_header() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/ip")
            .with_header(
                X_FORWARDED_FOR,
                header::HeaderValue::from_static("abc"),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "127.0.0.1")
    }
}
