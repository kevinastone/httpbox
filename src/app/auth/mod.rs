mod header;

use crate::app::response::{empty_response, ok};
use gotham::state::{FromState, State};

use self::header::BasicRealm;
use gotham_derive::{StateData, StaticResponseExtender};
use http::header as http_header;
use hyper::{Body, HeaderMap, Response, StatusCode};
use hyperx::header::{Authorization, Basic, Bearer, Header, Raw};
use serde_derive::Deserialize;

pub const REALM: &str = "User Visible Realm";

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct BasicAuthParams {
    user: String,
    passwd: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct BearerParams {
    token: String,
}

pub fn basic(state: State) -> (State, Response<Body>) {
    let creds = BasicAuthParams::borrow_from(&state);

    let headers = HeaderMap::borrow_from(&state)
        .get_all(http_header::AUTHORIZATION)
        .iter()
        .filter_map(|hv| hv.to_str().ok())
        .filter_map(|raw| {
            Authorization::<Basic>::parse_header(&Raw::from(raw))
                .ok()
                .map(|auth| auth.0)
        })
        .find(|basic| {
            basic.username == creds.user
                && basic.password.iter().any(|p| p == &creds.passwd)
        });

    match headers {
        Some(_) => ok(state, String::from("Authenticated")),
        None => {
            let mut res = empty_response(&state, StatusCode::UNAUTHORIZED);
            {
                let headers = res.headers_mut();
                headers.insert(
                    http_header::WWW_AUTHENTICATE,
                    http_header::HeaderValue::from_str(
                        &BasicRealm(REALM.to_owned()).to_string(),
                    )
                    .unwrap(),
                );
            }
            (state, res)
        }
    }
}

pub fn bearer(state: State) -> (State, Response<Body>) {
    let creds = BearerParams::borrow_from(&state);

    let headers = HeaderMap::borrow_from(&state)
        .get_all(http_header::AUTHORIZATION)
        .iter()
        .filter_map(|hv| hv.to_str().ok())
        .filter_map(|raw| {
            Authorization::<Bearer>::parse_header(&Raw::from(raw))
                .ok()
                .map(|auth| auth.0)
        })
        .find(|header| header.token == creds.token);

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
    use super::super::router;
    use super::{BasicRealm, REALM};

    use gotham::test::TestServer;
    use http::header;
    use hyper::StatusCode;
    use hyperx::header::{Authorization, Basic, Bearer};

    #[test]
    fn test_basic_no_authorization() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/basic-auth/my-username/my-password")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            header::HeaderValue::from_str(
                &BasicRealm(REALM.to_owned()).to_string()
            )
            .unwrap()
        )
    }

    #[test]
    fn test_basic_authorized() {
        let test_server = TestServer::new(router()).unwrap();

        let auth = Authorization(Basic {
            username: "my-username".to_owned(),
            password: Some("my-password".to_owned()),
        });

        let response = test_server
            .client()
            .get("http://localhost:3000/basic-auth/my-username/my-password")
            .with_header(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&auth.to_string()).unwrap(),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_basic_unauthorized() {
        let test_server = TestServer::new(router()).unwrap();

        let auth = Authorization(Basic {
            username: "my-username".to_owned(),
            password: Some("not-my-password".to_owned()),
        });

        let response = test_server
            .client()
            .get("http://localhost:3000/basic-auth/my-username/my-password")
            .with_header(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&auth.to_string()).unwrap(),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            header::HeaderValue::from_str(
                &BasicRealm(REALM.to_owned()).to_string()
            )
            .unwrap()
        )
    }

    #[test]
    fn test_bearer_no_authorization() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/bearer-auth/my-token")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_bearer_authorized() {
        let test_server = TestServer::new(router()).unwrap();

        let auth = Authorization(Bearer {
            token: "my-token".to_owned(),
        });

        let response = test_server
            .client()
            .get("http://localhost:3000/bearer-auth/my-token")
            .with_header(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&auth.to_string()).unwrap(),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_bearer_unauthorized() {
        let test_server = TestServer::new(router()).unwrap();

        let auth = Authorization(Bearer {
            token: "not-my-token".to_owned(),
        });

        let response = test_server
            .client()
            .get("http://localhost:3000/bearer-auth/my-token")
            .with_header(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&auth.to_string()).unwrap(),
            )
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
