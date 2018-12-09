use crate::app::response::{bad_request, empty_response, ok};
use gotham::state::{FromState, State};
use http::header;
use hyper::{Body, HeaderMap, Response, StatusCode, Uri};
use std::str::FromStr;
use url::form_urlencoded;

pub fn headers(state: State) -> (State, Response<Body>) {
    let headers = HeaderMap::borrow_from(&state)
        .iter()
        .map(|(n, v)| {
            format!("{}: {}", n, v.to_str().unwrap()).trim().to_owned()
        })
        .collect::<Vec<String>>()
        .join("\n");

    ok(state, headers.to_string())
}

pub fn response_headers(state: State) -> (State, Response<Body>) {
    let response_headers: Vec<(String, String)> = {
        Uri::borrow_from(&state)
            .query()
            .map(|query| form_urlencoded::parse(query.as_bytes()))
            .map(|pairs| pairs.into_owned().collect())
            .unwrap_or_else(|| vec![])
    };

    let headers: Result<Vec<_>, String> = response_headers
        .iter()
        .map(|(key, value)| {
            let name =
                header::HeaderName::from_str(key).map_err(|e| e.to_string())?;
            let value = header::HeaderValue::from_str(value)
                .map_err(|e| e.to_string())?;

            Ok((name, value))
        })
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
