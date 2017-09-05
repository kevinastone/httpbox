extern crate iron;
extern crate router;

use self::iron::{Request, Response, IronResult};
use self::iron::error::HttpError;
use self::iron::headers;
use self::iron::modifiers::Header;
use self::iron::status;
use self::router::Router;
use std::fmt;
use std::str::FromStr;

const BASIC_REALM_PREAMBLE: &'static str = "Basic realm=";
pub const REALM: &'static str = "User Visible Realm";

#[derive(Clone, Debug, PartialEq)]
struct BasicRealm(pub String);

impl BasicRealm {
    fn fmt_realm(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(BASIC_REALM_PREAMBLE)?;
        f.write_str("\"")?;
        f.write_str(&self.0)?;
        f.write_str("\"")?;
        Ok(())
    }
}

impl FromStr for BasicRealm {
    type Err = HttpError;

    fn from_str(s: &str) -> Result<BasicRealm, HttpError> {
        if s.starts_with(BASIC_REALM_PREAMBLE) {
            Ok(BasicRealm(s[BASIC_REALM_PREAMBLE.len() + 1..s.len() - 1].to_owned()))
        } else {
            Err(HttpError::Header)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct WWWAuthenticate(BasicRealm);

impl headers::Header for WWWAuthenticate {
    fn header_name() -> &'static str {
        "WWW-Authenticate"
    }

    fn parse_header(raw: &[Vec<u8>]) -> Result<Self, HttpError> {
        headers::parsing::from_one_raw_str(raw).map(WWWAuthenticate)
    }
}

impl headers::HeaderFormat for WWWAuthenticate {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_realm(f)
    }
}

pub fn basic(req: &mut Request) -> IronResult<Response> {

    let username = iexpect!(req.extensions.get::<Router>().unwrap().find("user"));
    let password = req.extensions
        .get::<Router>()
        .unwrap()
        .find("passwd")
        .map(|s| s.to_owned());

    if req.headers
           .get::<headers::Authorization<headers::Basic>>()
           .iter()
           .filter(|header| header.username == username && header.password == password)
           .next()
           .is_some() {
        Ok(Response::with((status::Status::Ok, "Authenticated")))
    } else {
        Ok(Response::with((status::Status::Unauthorized,
                           Header(WWWAuthenticate(BasicRealm(REALM.to_owned()))))))
    }
}

pub fn bearer(req: &mut Request) -> IronResult<Response> {

    let token = iexpect!(req.extensions.get::<Router>().unwrap().find("token"));

    if req.headers
           .get::<headers::Authorization<headers::Bearer>>()
           .iter()
           .filter(|header| header.token == token)
           .next()
           .is_some() {
        Ok(Response::with((status::Status::Ok, "Authenticated")))
    } else {
        Ok(Response::with(status::Status::Unauthorized))
    }
}

#[cfg(test)]
mod test {

    extern crate iron_test;

    use super::REALM;
    use super::super::app;
    use super::iron::headers;
    use iron::{Headers, status};
    use self::iron_test::request;

    #[test]
    fn test_basic_no_authorization() {

        let app = app();

        let res = request::get("http://localhost:3000/basic-auth/my-username/my-password",
                               Headers::new(),
                               &app)
                .unwrap();

        assert_eq!(res.status.unwrap(), status::Unauthorized);
        assert_eq!(res.headers
                       .get_raw("WWW-Authenticate")
                       .map(|v| &v[0])
                       .and_then(|v| String::from_utf8(v.clone()).ok())
                       .unwrap(),
                   format!("Basic realm=\"{}\"", REALM))
    }

    #[test]
    fn test_basic_authorized() {

        let app = app();
        let mut headers = Headers::new();
        headers.set(headers::Authorization(headers::Basic {
                                               username: "my-username".to_owned(),
                                               password: Some("my-password".to_owned()),
                                           }));

        let res = request::get("http://localhost:3000/basic-auth/my-username/my-password",
                               headers,
                               &app)
                .unwrap();

        assert_eq!(res.status.unwrap(), status::Ok)
    }

    #[test]
    fn test_basic_unauthorized() {

        let app = app();
        let mut headers = Headers::new();
        headers.set(headers::Authorization(headers::Basic {
                                               username: "my-username".to_owned(),
                                               password: Some("not-my-password".to_owned()),
                                           }));

        let res = request::get("http://localhost:3000/basic-auth/my-username/my-password",
                               headers,
                               &app)
                .unwrap();

        assert_eq!(res.status.unwrap(), status::Unauthorized);
        assert_eq!(res.headers
                       .get_raw("WWW-Authenticate")
                       .map(|v| &v[0])
                       .and_then(|v| String::from_utf8(v.clone()).ok())
                       .unwrap(),
                   format!("Basic realm=\"{}\"", REALM))
    }

    #[test]
    fn test_bearer_no_authorization() {

        let app = app();

        let res = request::get("http://localhost:3000/bearer-auth/my-token",
                               Headers::new(),
                               &app)
                .unwrap();

        assert_eq!(res.status.unwrap(), status::Unauthorized)
    }

    #[test]
    fn test_bearer_authorized() {

        let app = app();
        let mut headers = Headers::new();
        headers.set(headers::Authorization(headers::Bearer { token: "my-token".to_owned() }));

        let res = request::get("http://localhost:3000/bearer-auth/my-token", headers, &app)
            .unwrap();

        assert_eq!(res.status.unwrap(), status::Ok)
    }

    #[test]
    fn test_bearer_unauthorized() {

        let app = app();
        let mut headers = Headers::new();
        headers.set(headers::Authorization(headers::Bearer { token: "not-my-token".to_owned() }));

        let res = request::get("http://localhost:3000/bearer-auth/my-token", headers, &app)
            .unwrap();

        assert_eq!(res.status.unwrap(), status::Unauthorized)
    }
}
