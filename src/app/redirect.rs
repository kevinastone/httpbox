extern crate iron;
extern crate urlencoded;

use self::iron::{Request, Response, IronResult, Url};
use self::iron::Plugin;
use self::iron::modifiers::Redirect;
use self::iron::status;
use self::urlencoded::UrlEncodedQuery;


const URL_QUERY_PARAM: &'static str = "url";


pub fn to(req: &mut Request) -> IronResult<Response> {
    let url = iexpect!(req.get_ref::<UrlEncodedQuery>()
        .ok()
        .and_then(|hashmap| hashmap.get(URL_QUERY_PARAM))
        .and_then(|vals| vals.first())
        .and_then(|url| Url::parse(url).ok()));

    Ok(Response::with((status::Found, Redirect(url))))
}
