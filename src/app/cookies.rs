extern crate iron;

use self::iron::{Request, Response, IronResult};
use self::iron::status;

use self::super::cookie::parse_cookies;

pub fn cookies(req: &mut Request) -> IronResult<Response> {
    let cookies = parse_cookies(&req.headers)
        .iter()
        .map(|c| format!("{}", c))
        .collect::<Vec<String>>()
        .join("\n");
    Ok(Response::with((status::Ok, cookies.to_string())))
}
