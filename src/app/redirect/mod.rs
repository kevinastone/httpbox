mod uri;

use self::uri::absolute_url;
use crate::app::response::redirect_to;
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use hyper::{Body, Response, Uri};
use serde_derive::Deserialize;
use std::cmp::min;

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct RedirectCountParams {
    n: u16,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct RedirectUrlParams {
    url: String,
}

pub fn to(state: State) -> (State, Response<Body>) {
    let query = RedirectUrlParams::borrow_from(&state);
    let uri = try_or_error_response!(state, query.url.parse::<Uri>());
    redirect_to(state, uri)
}

pub fn relative(state: State) -> (State, Response<Body>) {
    let mut n = RedirectCountParams::borrow_from(&state).n;
    n = min(n - 1, 100);

    let url = if n > 0 {
        format!("/relative-redirect/{}", n)
    } else {
        String::from("/")
    };

    let uri = try_or_error_response!(state, url.parse::<Uri>());
    redirect_to(state, uri)
}

pub fn redirect(state: State) -> (State, Response<Body>) {
    relative(state)
}

pub fn absolute(mut state: State) -> (State, Response<Body>) {
    let mut n = RedirectCountParams::borrow_from(&state).n;
    n = min(n - 1, 100);

    let url = if n > 0 {
        format!("/absolute-redirect/{}", n)
    } else {
        String::from("/")
    };

    let request_uri = Uri::take_from(&mut state);
    let response_uri = try_or_error_response!(
        state,
        absolute_url(&state, request_uri)
            .and_then(|base| Ok(base.join(&url)?))
            .and_then(|url| Ok(url.to_string().parse::<Uri>()?))
    );
    redirect_to(state, response_uri)
}

#[cfg(test)]
mod test {
    use crate::app::app;
    use crate::headers::{HeaderMapExt, Location};
    use gotham::test::TestServer;
    use http::{StatusCode, Uri};

    #[test]
    fn test_redirect_to() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/redirect-to?url=http://example.com")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("http://example.com/")
        )
    }

    #[test]
    fn test_redirect() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/redirect/5")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("/relative-redirect/4")
        )
    }

    #[test]
    fn test_redirect_last() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/redirect/1")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("/")
        )
    }

    #[test]
    fn test_relative_redirect() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/relative-redirect/5")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("/relative-redirect/4")
        )
    }

    #[test]
    fn test_relative_redirect_last() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/relative-redirect/1")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("/")
        )
    }

    #[test]
    fn test_absolute_redirect() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/absolute-redirect/5")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("http://localhost:3000/absolute-redirect/4")
        )
    }

    #[test]
    fn test_absolute_redirect_last() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/absolute-redirect/1")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
        assert_eq!(
            response.headers().typed_get::<Location>().unwrap().uri(),
            &Uri::from_static("http://localhost:3000/")
        )
    }
}
