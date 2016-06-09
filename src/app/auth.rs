extern crate iron;
extern crate router;

use self::iron::{Request, Response, IronResult};
use self::iron::headers;
use self::iron::status;
use self::router::Router;

pub fn basic(req: &mut Request) -> IronResult<Response> {

    let username = iexpect!(req.extensions.get::<Router>().unwrap().find("user"));
    let password = req.extensions.get::<Router>().unwrap().find("passwd").map(|s| s.to_owned());

    if req.headers
        .get::<headers::Authorization<headers::Basic>>()
        .iter()
        .filter(|header| header.username == username && header.password == password)
        .next()
        .is_some() {
        Ok(Response::with(status::Status::Ok))
    } else {
        Ok(Response::with(status::Status::Unauthorized))
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
        Ok(Response::with(status::Status::Ok))
    } else {
        Ok(Response::with(status::Status::Unauthorized))
    }
}

#[cfg(test)]
mod test {

    extern crate iron_test;

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

        assert_eq!(res.status.unwrap(), status::Unauthorized)
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

        assert_eq!(res.status.unwrap(), status::Unauthorized)
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
