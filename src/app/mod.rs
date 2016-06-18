extern crate iron;
extern crate router;

use self::iron::Handler;
use self::iron::middleware::Chain;

mod auth;
mod bytes;
mod cache;
mod cookies;
mod headers;
mod index;
mod ip;
mod method;
mod random;
mod redirect;
mod routes;
mod status_code;
mod stream;
mod user_agent;

use self::index::IndexBuilder;
use self::router::Router;
use self::routes::Route;


pub fn app() -> Box<Handler> {

    let mut routes = IndexBuilder::new();
    routes.add(Route::new("/ip").set_description("Returns Origin IP").handle(ip::ip));
    routes.add(Route::new("/user-agent")
        .set_description("Returns user-agent")
        .handle(user_agent::user_agent));
    routes.add(Route::new("/headers").set_description("Returns headers").handle(headers::headers));
    routes.add(Route::new("/get")
        .set_description("Returns GET data")
        .add_example_param("key", "val")
        .handle(method::get));
    routes.add(Route::new("/status/:code")
        .set_description("Returns given HTTP Status code")
        .add_example_param("code", "418")
        .handle(status_code::status_code));
    routes.add(Route::new("/basic-auth/:user/:passwd")
        .set_description("HTTP Basic Auth Challenge")
        .add_example_param("user", "user")
        .add_example_param("passwd", "passwd")
        .handle(auth::basic));
    routes.add(Route::new("/bearer-auth/:token")
        .set_description("Bearer Auth Challenge")
        .add_example_param("token", "random-token")
        .handle(auth::bearer));
    routes.add(Route::new("/response-headers")
        .set_description("Returns given response headers")
        .add_example_param("key", "val")
        .handle(headers::response_headers));
    routes.add(Route::new("/redirect/:n")
        .set_description("302 Redirects n times")
        .add_example_param("n", "5")
        .handle(redirect::redirect));
    routes.add(Route::new("/redirect-to")
        .set_description("302 Redirects to the url= URL")
        .add_example_param("url", "http://example.com")
        .handle(redirect::to));
    routes.add(Route::new("/absolute-redirect/:n")
        .set_description("302 Absolute redirects n times")
        .add_example_param("n", "5")
        .handle(redirect::absolute));
    routes.add(Route::new("/relative-redirect/:n")
        .set_description("302 Relative redirects n times")
        .add_example_param("n", "5")
        .handle(redirect::relative));
    routes.add(Route::new("/cookies")
        .set_description("Returns cookie data")
        .handle(cookies::cookies));
    routes.add(Route::new("/cookies/set")
        .set_description("Sets one or more simple cookies")
        .handle(cookies::set_cookies));
    routes.add(Route::new("/cache")
        .set_description("Returns 200 unless an If-Modified-Since or If-None-Match header is \
                          provided, when it returns a 304")
        .handle(cache::cache));
    routes.add(Route::new("/cache/:n")
        .set_description("Sets a Cache-Control header for n seconds")
        .add_example_param("n", "10")
        .handle(cache::set_cache));
    routes.add(Route::new("/bytes/:n")
        .set_description("Generates n random bytes of binary data, accepts optional seed \
                          integer parameter")
        .add_example_param("n", "256")
        .handle(bytes::bytes));
    routes.add(Route::new("/stream-bytes/:n")
        .set_description("Streams n random bytes of binary data, accepts optional seed parameter")
        .add_example_param("n", "256")
        .handle(bytes::stream_bytes));

    let chain = Chain::new(Router::from(routes));
    Box::new(chain)
}
