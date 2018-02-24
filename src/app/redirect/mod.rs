extern crate gotham;
extern crate hyper;
extern crate mime;

mod uri;

use app::response::{bad_request, redirect_to};
use gotham::state::{FromState, State};
use hyper::{Response, Uri};
use self::uri::{absolute_url, join_url};
use std::cmp::min;
use url::Url;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct RedirectCountParams {
    n: u16,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct RedirectUrlParams {
    url: String,
}

pub fn to(mut state: State) -> (State, Response) {
    let query = RedirectUrlParams::take_from(&mut state);
    match Url::parse(&query.url[..]) {
        Ok(url) => redirect_to(state, url.to_string()),
        Err(_) => bad_request(state),
    }
}

pub fn relative(mut state: State) -> (State, Response) {
    let mut n = RedirectCountParams::take_from(&mut state).n;
    n = min(n - 1, 100);

    let url = if n <= 0 {
        String::from("/")
    } else {
        format!("/relative-redirect/{}", n)
    };

    redirect_to(state, url)
}

pub fn redirect(state: State) -> (State, Response) {
    relative(state)
}

pub fn absolute(mut state: State) -> (State, Response) {
    let mut n = RedirectCountParams::take_from(&mut state).n;
    n = min(n - 1, 100);

    let url = if n <= 0 {
        String::from("/")
    } else {
        format!("/absolute-redirect/{}", n)
    };

    let uri = Uri::borrow_from(&state).clone();
    let base = absolute_url(&state, uri);
    match base.and_then(|base| join_url(&url[..], &base)) {
        Some(url) => redirect_to(state, url.to_string()),
        None => bad_request(state),
    }
}

#[cfg(test)]
mod test {
    use super::super::router;

    use gotham::test::TestServer;
    use hyper::StatusCode;
    use hyper::header::Location;

    #[test]
    fn test_redirect_to() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/redirect-to?url=http://example.com")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Found);
        assert_eq!(
            response.headers().get::<Location>().unwrap(),
            &Location::new(String::from("http://example.com/"))
        )
    }

    #[test]
    fn test_redirect() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/redirect/5")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Found);
        assert_eq!(
            response.headers().get::<Location>().unwrap(),
            &Location::new(String::from("/relative-redirect/4"))
        )
    }

    #[test]
    fn test_redirect_last() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/redirect/1")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Found);
        assert_eq!(
            response.headers().get::<Location>().unwrap(),
            &Location::new(String::from("/"))
        )
    }

    #[test]
    fn test_relative_redirect() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/relative-redirect/5")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Found);
        assert_eq!(
            response.headers().get::<Location>().unwrap(),
            &Location::new(String::from("/relative-redirect/4"))
        )
    }

    #[test]
    fn test_relative_redirect_last() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/relative-redirect/1")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Found);
        assert_eq!(
            response.headers().get::<Location>().unwrap(),
            &Location::new(String::from("/"))
        )
    }

    #[test]
    fn test_absolute_redirect() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/absolute-redirect/5")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Found);
        assert_eq!(
            response.headers().get::<Location>().unwrap(),
            &Location::new(String::from(
                "http://localhost:3000/absolute-redirect/4"
            ))
        )
    }

    #[test]
    fn test_absolute_redirect_last() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/absolute-redirect/1")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Found);
        assert_eq!(
            response.headers().get::<Location>().unwrap(),
            &Location::new(String::from("http://localhost:3000/"))
        )
    }
}
