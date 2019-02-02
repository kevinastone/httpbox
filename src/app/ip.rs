use crate::app::response::{internal_server_error, ok};
use crate::headers::{HeaderMapExt, XForwardedFor};
use gotham::state::{client_addr, FromState, State};
use hyper::{Body, HeaderMap, Response};
use std::net::IpAddr;

fn x_forward_for(state: &State) -> Option<IpAddr> {
    HeaderMap::borrow_from(&state)
        .typed_get::<XForwardedFor>()
        .map(|h| h.ip_addr())
}

fn client_ip_addr(state: &State) -> Option<IpAddr> {
    client_addr(state).map(|a| a.ip())
}

pub fn ip(state: State) -> (State, Response<Body>) {
    let remote_ip = eexpect!(
        internal_server_error,
        state,
        x_forward_for(&state).or_else(|| client_ip_addr(&state))
    );
    ok(state, remote_ip.to_string())
}

#[cfg(test)]
mod test {
    use crate::app::app;
    use crate::headers::{Header, XForwardedFor};
    use crate::test::request::TestRequestTypedHeader;
    use gotham::test::TestServer;
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
            .with_typed_header(XForwardedFor("1.2.3.4".parse().unwrap()))
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
            .with_header(XForwardedFor::name(), "abc".parse().unwrap())
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "127.0.0.1")
    }
}
