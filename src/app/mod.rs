extern crate iron;
extern crate router;

use self::iron::Iron;
use self::iron::Handler;
use self::iron::middleware::Chain;

mod auth;
mod bytes;
mod cache;
mod cookies;
mod headers;
mod index;
mod ip;
mod random;
mod redirect;
mod routes;
mod status_code;
mod stream;
mod user_agent;

use self::routes::Routes;


pub fn app() -> Iron<Box<Handler>> {

    let mut routes = Routes::new();
    routes.get("/", index::index);
    routes.get("/basic-auth/:user/:passwd", auth::basic);
    routes.get("/bearer-auth/:token", auth::bearer);
    routes.get("/bytes/:n", bytes::bytes);
    routes.get("/cache", cache::cache);
    routes.get("/cache/:n", cache::set_cache);
    routes.get("/cookies", cookies::cookies);
    routes.get("/cookies/set", cookies::set_cookies);
    routes.get("/headers", headers::headers);
    routes.get("/ip", ip::ip);
    routes.get("/redirect/:n", redirect::redirect);
    routes.get("/redirect-to", redirect::to);
    routes.get("/absolute-redirect/:n", redirect::absolute);
    routes.get("/stream-bytes/:n", bytes::stream_bytes);
    routes.get("/relative-redirect/:n", redirect::relative);
    routes.get("/response-headers", headers::response_headers);
    routes.get("/status/:code", status_code::status_code);
    routes.get("/user-agent", user_agent::user_agent);

    let chain = Chain::new(routes.to_router());
    Iron::new(Box::new(chain))
}
