extern crate iron;

use self::iron::{Request, Response, IronResult};
use self::iron::status;

pub fn index(req: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, format!("Welcome to {}", req.url.to_string()))))
}
