extern crate iron;
extern crate router;

use self::iron::Iron;
use self::iron::Handler;
use self::iron::middleware::Chain;
use self::router::Router;

mod auth;
mod bytes;
mod cache;
mod cookies;
mod headers;
mod index;
mod ip;
mod random;
mod redirect;
mod status_code;
mod stream;
mod user_agent;

pub fn app() -> Iron<Box<Handler>> {

    let mut router = Router::new();
    router.get("/", index::index);
    router.get("/basic-auth/:user/:passwd", auth::basic);
    router.get("/bytes/:n", bytes::bytes);
    router.get("/cache", cache::cache);
    router.get("/cache/:n", cache::set_cache);
    router.get("/cookies", cookies::cookies);
    router.get("/cookies/set", cookies::set_cookies);
    router.get("/headers", headers::headers);
    router.get("/ip", ip::ip);
    router.get("/redirect/:n", redirect::redirect);
    router.get("/redirect-to", redirect::to);
    router.get("/absolute-redirect/:n", redirect::absolute);
    router.get("/stream-bytes/:n", bytes::stream_bytes);
    router.get("/relative-redirect/:n", redirect::relative);
    router.get("/status/:code", status_code::status_code);
    router.get("/user-agent", user_agent::user_agent);

    let chain = Chain::new(router);
    Iron::new(Box::new(chain))
}
