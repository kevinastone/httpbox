use crate::app::response::{bad_request, empty_response, ok};
use failure::Fallible;
use gotham::state::{FromState, State};
use http::header::HeaderName;
use hyper::{Body, HeaderMap, Response, StatusCode, Uri};
use url::form_urlencoded;

pub fn headers(state: State) -> (State, Response<Body>) {
    let request_headers = HeaderMap::borrow_from(&state)
        .iter()
        .map(|(n, v)| Ok((n, v.to_str()?)));

    match request_headers
        .map(|r| r.map(|(n, v)| format!("{}: {}", n, v).trim().to_owned()))
        .collect::<Fallible<Vec<_>>>()
    {
        Err(_) => bad_request(state),
        Ok(body) => ok(state, body.join("\n")),
    }
}

pub fn response_headers(state: State) -> (State, Response<Body>) {
    let response_headers: Vec<_> = {
        Uri::borrow_from(&state)
            .query()
            .map(|query| form_urlencoded::parse(query.as_bytes()))
            .map(|pairs| pairs.into_owned().collect())
            .unwrap_or_else(|| vec![])
    };

    let headers: Fallible<Vec<_>> = response_headers
        .iter()
        .map(|(name, value)| Ok((name.parse::<HeaderName>()?, value.parse()?)))
        .collect();

    match headers {
        Err(_) => bad_request(state),
        Ok(hdrs) => {
            let mut res = empty_response(&state, StatusCode::OK);
            let headers = res.headers_mut();
            for (key, value) in hdrs {
                headers.insert(key, value);
            }

            (state, res)
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use http::header;
    use hyper::StatusCode;

    #[test]
    fn test_headers() {
        let test_server = TestServer::new(router()).unwrap();
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
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/response-headers?X-Request-ID=1234")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("X-Request-ID").unwrap(), "1234")
    }
}
