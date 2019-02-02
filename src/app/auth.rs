use crate::app::response::{empty_response, ok};
use crate::headers::authorization::{Basic, Bearer};
use crate::headers::WWWAuthenticate;
use crate::headers::{Authorization, HeaderMapExt};
use crate::http::{HeaderMap, Response, StatusCode};
use gotham::state::{FromState, State};
use gotham_derive::{StateData, StaticResponseExtender};
use serde_derive::Deserialize;

pub(crate) const REALM: &str = "User Visible Realm";

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct BasicAuthParams {
    user: String,
    passwd: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct BearerParams {
    token: String,
}

pub fn basic(state: State) -> (State, Response) {
    let creds = BasicAuthParams::borrow_from(&state);

    let headers = HeaderMap::borrow_from(&state)
        .typed_get::<Authorization<Basic>>()
        .map(|header| header.0)
        .filter(|basic| {
            basic.username() == creds.user && basic.password() == creds.passwd
        });

    match headers {
        Some(_) => ok(state, String::from("Authenticated")),
        None => {
            let mut res = empty_response(&state, StatusCode::UNAUTHORIZED);
            res.headers_mut()
                .typed_insert(WWWAuthenticate::basic_realm(REALM));
            (state, res)
        }
    }
}

pub fn bearer(state: State) -> (State, Response) {
    let creds = BearerParams::borrow_from(&state);

    let headers = HeaderMap::borrow_from(&state)
        .typed_get::<Authorization<Bearer>>()
        .map(|header| header.0)
        .filter(|bearer| bearer.token() == creds.token);

    match headers {
        Some(_) => ok(state, String::from("Authenticated")),
        None => {
            let res = empty_response(&state, StatusCode::UNAUTHORIZED);
            (state, res)
        }
    }
}

#[cfg(test)]
mod test {
    use super::REALM;
    use crate::app::app;
    use crate::headers::{Authorization, HeaderMapExt, WWWAuthenticate};
    use crate::test::request::TestRequestTypedHeader;
    use gotham::test::TestServer;
    use http::StatusCode;

    #[test]
    fn test_basic_no_authorization() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/basic-auth/my-username/my-password")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers().typed_get::<WWWAuthenticate>().unwrap(),
            WWWAuthenticate::basic_realm(REALM),
        )
    }

    #[test]
    fn test_basic_authorized() {
        let test_server = TestServer::new(app()).unwrap();

        let auth = Authorization::basic("my-username", "my-password");

        let response = test_server
            .client()
            .get("http://localhost:3000/basic-auth/my-username/my-password")
            .with_typed_header(auth)
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_basic_unauthorized() {
        let test_server = TestServer::new(app()).unwrap();

        let auth = Authorization::basic("my-username", "not-my-password");

        let response = test_server
            .client()
            .get("http://localhost:3000/basic-auth/my-username/my-password")
            .with_typed_header(auth)
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers().typed_get::<WWWAuthenticate>().unwrap(),
            WWWAuthenticate::basic_realm(REALM),
        )
    }

    #[test]
    fn test_bearer_no_authorization() {
        let test_server = TestServer::new(app()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/bearer-auth/my-token")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_bearer_authorized() {
        let test_server = TestServer::new(app()).unwrap();

        let auth = Authorization::bearer("my-token").unwrap();

        let response = test_server
            .client()
            .get("http://localhost:3000/bearer-auth/my-token")
            .with_typed_header(auth)
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_bearer_unauthorized() {
        let test_server = TestServer::new(app()).unwrap();

        let auth = Authorization::bearer("not-my-token").unwrap();

        let response = test_server
            .client()
            .get("http://localhost:3000/bearer-auth/my-token")
            .with_typed_header(auth)
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
