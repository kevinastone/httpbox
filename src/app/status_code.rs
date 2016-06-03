extern crate iron;
extern crate router;

use self::iron::{Request, Response, IronResult};
use self::iron::status;
use self::router::Router;

pub fn status_code(req: &mut Request) -> IronResult<Response> {

    let code = req.extensions.get::<Router>().unwrap().find("code").unwrap_or("200");
    let code = itry!(code.parse::<u16>(), status::BadRequest);

    Ok(Response::with(status::Status::from_u16(code)))
}
