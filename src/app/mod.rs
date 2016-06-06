extern crate iron;
extern crate router;

use self::iron::Iron;
use self::iron::middleware::Chain;
use self::router::Router;

mod bytes;
mod cookies;
mod headers;
mod index;
mod ip;
mod random;
mod redirect;
mod status_code;
mod stream;
mod user_agent;

pub fn app() -> Iron<Chain> {

    let mut router = Router::new();
    router.get("/", index::index);
    router.get("/bytes/:n", bytes::bytes);
    router.get("/cookies", cookies::cookies);
    router.get("/cookies/set", cookies::set_cookies);
    router.get("/headers", headers::headers);
    router.get("/ip", ip::ip);
    router.get("/redirect-to", redirect::to);
    router.get("/absolute-redirect/:n", redirect::absolute);
    router.get("/relative-redirect/:n", redirect::relative);
    router.get("/stream-bytes/:n", bytes::stream_bytes);
    router.get("/status/:code", status_code::status_code);
    router.get("/user-agent", user_agent::user_agent);

    let chain = Chain::new(router);
    Iron::new(chain)
}
