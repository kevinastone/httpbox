use crate::app::response::{empty_response, ok};
use crate::headers::{Cookie, HeaderMapExt};
use cookie::Cookie as HTTPCookie;
use failure::Fallible;
use gotham::state::{FromState, State};
use http::header;
use hyper::{Body, HeaderMap, Response, StatusCode, Uri};
use url::form_urlencoded;

pub fn cookies(state: State) -> (State, Response<Body>) {
    let cookies = HeaderMap::borrow_from(&state).typed_get::<Cookie>();

    let body = cookies
        .iter()
        .flat_map(|cookie| cookie.iter().map(|(n, v)| format!("{} = {}", n, v)))
        .collect::<Vec<_>>();

    ok(state, body.join("\n"))
}

pub fn set_cookies(state: State) -> (State, Response<Body>) {
    let response_cookies: Vec<_> = Uri::borrow_from(&state)
        .query()
        .map(|query| form_urlencoded::parse(query.as_bytes()))
        .map(|pairs| pairs.into_owned().collect())
        .unwrap_or_else(|| vec![])
        .iter()
        .map(|(ref k, ref v)| HTTPCookie::new(k.to_owned(), v.to_owned()))
        .collect();

    let cookies: Fallible<Vec<_>> = response_cookies
        .iter()
        .map(|cookie| Ok(cookie.to_string().parse()?))
        .collect();

    let mut res = empty_response(&state, StatusCode::OK);
    let headers = res.headers_mut();
    for cookie in try_or_error_response!(state, cookies) {
        headers.insert(header::SET_COOKIE, cookie);
    }

    (state, res)
}

#[cfg(test)]
mod test {
    use super::header;
    use crate::app::app;

    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn test_no_cookies() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cookies")
            .perform()
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "")
    }

    #[test]
    fn test_cookies() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cookies")
            .with_header(
                header::COOKIE,
                header::HeaderValue::from_static("test=value"),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "test = value")
    }

    #[test]
    fn test_multiple_cookies() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cookies")
            .with_header(
                header::COOKIE,
                header::HeaderValue::from_static("first=value; second=another"),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "first = value\nsecond = another")
    }

    #[test]
    fn test_set_cookies() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cookies/set?test=value")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::SET_COOKIE).unwrap(),
            "test=value"
        )
    }
}
