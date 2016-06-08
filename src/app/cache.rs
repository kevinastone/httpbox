extern crate iron;
extern crate router;

use self::iron::{Request, Response, IronResult};
use self::iron::headers;
use self::iron::modifiers::Header;
use self::iron::status;
use self::router::Router;

pub fn cache(req: &mut Request) -> IronResult<Response> {

    if req.headers.get::<headers::IfModifiedSince>().is_some() ||
       req.headers.get::<headers::IfNoneMatch>().is_some() {
        Ok(Response::with(status::Status::NotModified))
    } else {
        Ok(Response::with(status::Status::Ok))
    }
}

pub fn set_cache(req: &mut Request) -> IronResult<Response> {
    let n = iexpect!(req.extensions.get::<Router>().unwrap().find("n"));
    let n = itry!(n.parse::<u32>(), status::BadRequest);

    Ok(Response::with((status::Status::Ok,
                       Header(headers::CacheControl(vec![headers::CacheDirective::MaxAge(n)])))))
}
