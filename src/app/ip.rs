extern crate iron;

use self::iron::{Request, Response, IronResult};
use self::iron::status;

pub fn ip(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, req.remote_addr.ip().to_string())))
}
