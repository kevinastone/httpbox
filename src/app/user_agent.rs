extern crate iron;

use self::iron::{Request, Response, IronResult};
use self::iron::headers::UserAgent;
use self::iron::status;

pub fn user_agent(req: &mut Request) -> IronResult<Response> {

    let user_agent = iexpect!(req.headers.get::<UserAgent>());
    Ok(Response::with((status::Ok, user_agent.to_string())))
}
