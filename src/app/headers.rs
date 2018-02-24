extern crate gotham;
extern crate hyper;
extern crate mime;

use app::response::ok;
use gotham::http::response::create_response;
use gotham::state::{FromState, State};
use hyper::{Headers, Response, StatusCode, Uri};
use url::form_urlencoded;

pub fn headers(state: State) -> (State, Response) {
    let headers = Headers::borrow_from(&state)
        .iter()
        .map(|h| format!("{}", h).trim().to_owned())
        .collect::<Vec<String>>()
        .join("\n");

    ok(state, headers.to_string())
}

pub fn response_headers(state: State) -> (State, Response) {
    let response_headers: Vec<(String, String)> = {
        Uri::borrow_from(&state)
            .query()
            .map(|query| form_urlencoded::parse(query.as_bytes()))
            .map(|pairs| pairs.into_owned().collect())
            .unwrap_or_else(|| vec![])
    };

    let mut res = create_response(
        &state,
        StatusCode::Ok,
        Some((vec![], mime::TEXT_PLAIN)),
    );

    {
        let headers = res.headers_mut();
        for (key, value) in response_headers {
            headers.set_raw(key, value);
        }
    }

    (state, res)
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use hyper::StatusCode;

    header! { (XRequestID, "X-Request-ID") => [String] }

    #[test]
    fn test_headers() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/headers")
            .with_header(XRequestID(String::from("1234")))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_utf8_body().unwrap();
        assert!(result_body.contains("X-Request-ID: 1234"));
        assert_eq!(result_body, "Host: localhost:3000\nX-Request-ID: 1234")
    }

    #[test]
    fn test_response_headers() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/response-headers?X-Request-ID=1234")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
        assert_eq!(
            response.headers().get::<XRequestID>().unwrap(),
            &XRequestID(String::from("1234"))
        )
    }
}
