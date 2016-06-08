extern crate iron;
extern crate router;

use self::iron::{Request, Response, IronResult};
use self::iron::headers;
use self::iron::status;
use self::router::Router;

pub fn basic(req: &mut Request) -> IronResult<Response> {

    let username = iexpect!(req.extensions.get::<Router>().unwrap().find("user"));
    let password = req.extensions.get::<Router>().unwrap().find("passwd").map(|s| s.to_owned());

    match req.headers
        .get::<headers::Authorization<headers::Basic>>()
        .iter()
        .filter(|header| header.username == username && header.password == password)
        .next() {
        Some(_) => Ok(Response::with(status::Status::Ok)),
        None => Ok(Response::with(status::Status::Unauthorized)),
    }
}

pub fn bearer(req: &mut Request) -> IronResult<Response> {

    let token = iexpect!(req.extensions.get::<Router>().unwrap().find("token"));

    match req.headers
        .get::<headers::Authorization<headers::Bearer>>()
        .iter()
        .filter(|header| header.token == token)
        .next() {
        Some(_) => Ok(Response::with(status::Status::Ok)),
        None => Ok(Response::with(status::Status::Unauthorized)),
    }
}
