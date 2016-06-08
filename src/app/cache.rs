extern crate iron;

use self::iron::{Request, Response, IronResult};
use self::iron::headers;
use self::iron::status;

pub fn cache(req: &mut Request) -> IronResult<Response> {

    if req.headers.get::<headers::IfModifiedSince>().is_some() ||
       req.headers.get::<headers::IfNoneMatch>().is_some() {
        Ok(Response::with(status::Status::NotModified))
    } else {
        Ok(Response::with(status::Status::Ok))
    }
}
