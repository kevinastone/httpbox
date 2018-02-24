extern crate gotham;
extern crate hyper;
extern crate mime;

use app::response::{internal_server_error, ok};
use gotham::state::{client_addr, FromState, State};

use hyper::{header, Headers, Response};

pub const X_FORWARD_FOR: &'static str = "X-Forwarded-For";

fn client_ip_addr(state: &State) -> Option<String> {
    client_addr(&state).map(|a| a.ip().to_string())
}

pub fn ip(state: State) -> (State, Response) {
    match {
        let headers = Headers::borrow_from(&state);
        headers
            .get_raw(X_FORWARD_FOR)
            .map(|h| header::parsing::from_one_raw_str(h).ok())
            .unwrap_or_else(|| client_ip_addr(&state))
    } {
        Some(remote_ip) => ok(state, remote_ip.into_bytes()),
        None => internal_server_error(state),
    }
}

#[cfg(test)]
mod test {
    use super::X_FORWARD_FOR;
    use super::super::router;

    use gotham::test::TestServer;
    use hyper::StatusCode;

    header! { (XForwardFor, X_FORWARD_FOR) => [String] }

    #[test]
    fn test_ip() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/ip")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "127.0.0.1")
    }

    #[test]
    fn test_ip_from_header() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/ip")
            .with_header(XForwardFor(String::from("1.2.3.4")))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "1.2.3.4")
    }
}
