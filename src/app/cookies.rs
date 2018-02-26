extern crate cookie;
extern crate gotham;
extern crate hyper;
extern crate lazy_static;
extern crate mime;

use app::response::{empty_response, ok};
use cookie::Cookie;
use gotham::state::{FromState, State};
use hyper::{header, Headers, Response, StatusCode, Uri};
use url::form_urlencoded;

lazy_static! {
    static ref EMPTY_COOKIES: header::Cookie = header::Cookie::new();
}

pub fn cookies(state: State) -> (State, Response) {
    let cookies = Headers::borrow_from(&state)
        .get::<header::Cookie>()
        .unwrap_or(&EMPTY_COOKIES)
        .iter()
        .map(|(k, v)| format!("{} = {}", k, v))
        .collect::<Vec<String>>()
        .join("\n");

    ok(state, cookies.into_bytes())
}

pub fn set_cookies(state: State) -> (State, Response) {
    let cookies: Vec<(String)> = Uri::borrow_from(&state)
        .query()
        .map(|query| form_urlencoded::parse(query.as_bytes()))
        .map(|pairs| pairs.into_owned().collect())
        .unwrap_or_else(|| vec![])
        .iter()
        .map(|&(ref k, ref v)| Cookie::new(k.to_owned(), v.to_owned()))
        .map(|c| c.to_string())
        .collect();

    let mut res = empty_response(&state, StatusCode::Ok);
    {
        let headers = res.headers_mut();
        headers.set(header::SetCookie(cookies))
    }
    (state, res)
}

#[cfg(test)]
mod test {
    use super::header;
    use super::super::router;

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
        assert_eq!(response.status(), StatusCode::Ok);
        let result_body = response.read_utf8_body().unwrap();
        assert_eq!(result_body, "")
    }

    #[test]
    fn test_cookies() {
        let mut cookie = header::Cookie::new();
        cookie.set("test", "value");

        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/cookies")
            .with_header(cookie)
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
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

        assert_eq!(response.status(), StatusCode::Ok);
        assert_eq!(
            response.headers().get::<header::SetCookie>().unwrap(),
            &header::SetCookie(vec![String::from("test=value")])
        )
    }
}
