use crate::app::response::{bad_request, empty_response, ok};
use cookie::Cookie;
use failure::Fallible;
use gotham::state::{FromState, State};
use http::header;
use hyper::{Body, HeaderMap, Response, StatusCode, Uri};
use url::form_urlencoded;

fn parse_cookie(header: &header::HeaderValue) -> Fallible<Cookie> {
    Ok(Cookie::parse(header.to_str()?)?)
}

pub fn cookies(state: State) -> (State, Response<Body>) {
    let cookies = HeaderMap::borrow_from(&state)
        .get_all(header::COOKIE)
        .iter()
        .map(parse_cookie);

    let body = try_or_error_response!(
        state,
        cookies
            .map(|r| r.map(|c| format!("{} = {}", c.name(), c.value())))
            .collect::<Fallible<Vec<_>>>()
    );

    ok(state, body.join("\n"))
}

pub fn set_cookies(state: State) -> (State, Response<Body>) {
    let response_cookies: Vec<_> = Uri::borrow_from(&state)
        .query()
        .map(|query| form_urlencoded::parse(query.as_bytes()))
        .map(|pairs| pairs.into_owned().collect())
        .unwrap_or_else(|| vec![])
        .iter()
        .map(|(ref k, ref v)| Cookie::new(k.to_owned(), v.to_owned()))
        .collect();

    let cookies: Fallible<Vec<_>> = response_cookies
        .iter()
        .map(|cookie| Ok(cookie.to_string().parse()?))
        .collect();

    match cookies {
        Err(_) => bad_request(state),
        Ok(encoded_cookies) => {
            let mut res = empty_response(&state, StatusCode::OK);
            let headers = res.headers_mut();
            for cookie in encoded_cookies {
                headers.insert(header::SET_COOKIE, cookie);
            }

            (state, res)
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::router;
    use super::header;

    use gotham::test::TestServer;
    use hyper::StatusCode;

    #[test]
    fn test_no_cookies() {
        let test_server = TestServer::new(router()).unwrap();
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
        let test_server = TestServer::new(router()).unwrap();
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
    fn test_set_cookies() {
        let test_server = TestServer::new(router()).unwrap();
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
