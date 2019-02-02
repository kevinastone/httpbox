use crate::app::response::{empty_response, ok};
use crate::headers::{Cookie, HeaderMapExt, SetCookie};
use crate::http::{HeaderMap, Response, StatusCode, Uri};
use cookie::Cookie as HTTPCookie;
use gotham::state::{FromState, State};
use itertools::{Either, Itertools};
use url::form_urlencoded;

pub fn cookies(state: State) -> (State, Response) {
    let cookies = HeaderMap::borrow_from(&state).typed_get::<Cookie>();

    let body = cookies
        .iter()
        .flat_map(|cookie| cookie.iter())
        .format_with("\n", |cookie, f| {
            f(&format_args!("{} = {}", cookie.name(), cookie.value()))
        })
        .to_string();

    ok(state, body)
}

pub fn set_cookies(state: State) -> (State, Response) {
    let response_cookies = Uri::borrow_from(&state)
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
        .map(|(k, v)| SetCookie(HTTPCookie::new(k, v)));

    let mut res = empty_response(&state, StatusCode::OK);
    let headers = res.headers_mut();
    for cookie in response_cookies {
        headers.typed_insert(cookie);
    }

    (state, res)
}

#[cfg(test)]
mod test {
    use crate::app::app;
    use crate::headers::{Cookie, SetCookie};
    use crate::test::request::TestRequestTypedHeader;
    use cookie::Cookie as HTTPCookie;
    use gotham::test::TestServer;
    use headers::HeaderMapExt;
    use http::StatusCode;

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
            .with_typed_header(Cookie(vec![HTTPCookie::new("test", "value")]))
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
            .with_typed_header(Cookie(vec![
                HTTPCookie::new("first", "value"),
                HTTPCookie::new("second", "another"),
            ]))
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
            response.headers().typed_get::<SetCookie>().unwrap(),
            SetCookie(HTTPCookie::new("test", "value"))
        )
    }
}
