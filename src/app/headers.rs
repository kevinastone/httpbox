use crate::app::response::{empty_response, ok};
use failure::Fallible;
use gotham::state::{FromState, State};
use http::header::HeaderName;
use hyper::{Body, HeaderMap, Response, StatusCode, Uri};
use itertools::{process_results, Either, Itertools};
use url::form_urlencoded;

pub fn headers(state: State) -> (State, Response<Body>) {
    let request_headers = HeaderMap::borrow_from(&state)
        .iter()
        .map(|(n, v)| v.to_str().map(|v| (n, v)));

    let body = try_or_error_response!(
        state,
        process_results(request_headers, |iter| iter
            .format_with("\n", |(n, v), f| f(&format_args!(
                "{}: {}",
                n,
                v.trim()
            )))
            .to_string())
    );

    ok(state, body)
}

pub fn response_headers(state: State) -> (State, Response<Body>) {
    let response_headers = {
        Uri::borrow_from(&state)
            .query()
            .map_or_else(
                || Either::Right(vec![]),
                |query| {
                    Either::Left(
                        form_urlencoded::parse(query.as_bytes()).into_owned(),
                    )
                },
            )
            .into_iter()
    };

    let output_headers = try_or_error_response!(
        state,
        response_headers
            .map(|(name, value)| Ok((
                name.parse::<HeaderName>()?,
                value.parse()?
            )))
            .collect::<Fallible<Vec<_>>>()
    );

    let mut res = empty_response(&state, StatusCode::OK);
    let headers = res.headers_mut();
    for (key, value) in output_headers {
        headers.insert(key, value);
    }

    (state, res)
}

#[cfg(test)]
mod test {
    use crate::app::app;
    use gotham::test::TestServer;
    use http::{header, StatusCode};

    #[test]
    fn test_headers() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/headers")
            .with_header(
                "X-Request-ID",
                header::HeaderValue::from_static("1234"),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert!(result_body.contains("x-request-id: 1234"));
        assert_eq!(result_body, "x-request-id: 1234\nhost: localhost:3000")
    }

    #[test]
    fn test_response_headers() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/response-headers?X-Request-ID=1234")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("X-Request-ID").unwrap(), "1234")
    }
}
