extern crate gotham;
extern crate hyper;
extern crate mime;

mod header;

use crate::app::response::{empty_response, ok};
use gotham::state::{FromState, State};

use self::header::{BasicRealm, WWWAuthenticate};
use hyper::header::{Authorization, Basic, Bearer};
use hyper::{Headers, Response, StatusCode};

pub const REALM: &'static str = "User Visible Realm";

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct BasicAuthParams {
    user: String,
    passwd: String,
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct BearerParams {
    token: String,
}

pub fn basic(mut state: State) -> (State, Response) {
    let creds = BasicAuthParams::take_from(&mut state);

    let headers = Headers::take_from(&mut state);
    match headers
        .get::<Authorization<Basic>>()
        .iter()
        .filter(|header| {
            header.username == creds.user
                && header
                    .password
                    .iter()
                    .filter(|p| *p == &creds.passwd)
                    .next()
                    .is_some()
        })
        .next()
    {
        Some(_) => ok(state, String::from("Authenticated").into_bytes()),
        None => {
            let mut res = empty_response(&state, StatusCode::Unauthorized);
            {
                let headers = res.headers_mut();
                headers.set(WWWAuthenticate(BasicRealm(REALM.to_owned())))
            }
            (state, res)
        }
    }
}

pub fn bearer(mut state: State) -> (State, Response) {
    let creds = BearerParams::take_from(&mut state);

    match Headers::take_from(&mut state)
        .get::<Authorization<Bearer>>()
        .iter()
        .filter(|header| header.token == creds.token)
        .next()
    {
        Some(_) => ok(state, String::from("Authenticated").into_bytes()),
        None => {
            let res = empty_response(&state, StatusCode::Unauthorized);
            (state, res)
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::router;
    use super::{BasicRealm, WWWAuthenticate, REALM};

    use gotham::test::TestServer;
    use hyper::header::{Authorization, Basic, Bearer};
    use hyper::StatusCode;

    #[test]
    fn test_basic_no_authorization() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/basic-auth/my-username/my-password")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Unauthorized);
        assert_eq!(
            response.headers().get::<WWWAuthenticate>().unwrap(),
            &WWWAuthenticate(BasicRealm(REALM.to_owned()))
        )
    }

    #[test]
    fn test_basic_authorized() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/basic-auth/my-username/my-password")
            .with_header(Authorization(Basic {
                username: "my-username".to_owned(),
                password: Some("my-password".to_owned()),
            }))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
    }

    #[test]
    fn test_basic_unauthorized() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/basic-auth/my-username/my-password")
            .with_header(Authorization(Basic {
                username: "my-username".to_owned(),
                password: Some("not-my-password".to_owned()),
            }))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Unauthorized);
        assert_eq!(
            response.headers().get::<WWWAuthenticate>().unwrap(),
            &WWWAuthenticate(BasicRealm(REALM.to_owned()))
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

        assert_eq!(response.status(), StatusCode::Unauthorized);
    }

    #[test]
    fn test_bearer_authorized() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/bearer-auth/my-token")
            .with_header(Authorization(Bearer {
                token: "my-token".to_owned(),
            }))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);
    }

    #[test]
    fn test_bearer_unauthorized() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost:3000/bearer-auth/my-token")
            .with_header(Authorization(Bearer {
                token: "not-my-token".to_owned(),
            }))
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Unauthorized);
    }
}
