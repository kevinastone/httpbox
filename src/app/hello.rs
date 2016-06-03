extern crate iron;

use self::iron::{Request, Response, IronResult};
use self::iron::status;

pub fn hello(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, req.url.to_string())))
}
