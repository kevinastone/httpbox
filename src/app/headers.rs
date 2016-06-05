extern crate iron;

use self::iron::{Request, Response, IronResult};
use self::iron::status;

pub fn headers(req: &mut Request) -> IronResult<Response> {
    let headers = req.headers.iter().map(|h| format!("{}", h)).collect::<Vec<String>>().join("\n");
    Ok(Response::with((status::Ok, headers.to_string())))
}
