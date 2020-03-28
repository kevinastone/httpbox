use crate::router::{route, Route, Router};
use hyper::http::Method;

pub mod auth;
pub mod bytes;
pub mod cache;
pub mod cookies;
pub mod delay;
pub mod headers;
pub mod index;
pub mod ip;
pub mod method;
pub mod redirect;
pub mod status_code;
pub mod user_agent;

pub fn router() -> Router {
    let builder = Router::builder()
        .install(
            crate::service::ip::ip,
            route(path!("ip")).description("Returns Origin IP"),
        )
        .install(
            crate::service::user_agent::user_agent,
            route(path!("user-agent")).description("Returns user-agent"),
        )
        .install(
            crate::service::headers::headers,
            route(path!("headers")).description("Returns headers"),
        )
        .install(
            crate::service::method::get,
            route(path!("get"))
                .description("Returns GET data")
                .add_example_param("key", "val"),
        )
        .install(
            crate::service::method::post,
            route(path!("post"))
                .method(Method::POST)
                .description("Returns POST data"),
        )
        .install(
            crate::service::method::patch,
            route(path!("patch"))
                .method(Method::PATCH)
                .description("Returns PUT data"),
        )
        .install(
            crate::service::method::put,
            route(path!("put"))
                .method(Method::PUT)
                .description("Returns PUT data"),
        )
        .install(
            crate::service::method::delete,
            route(path!("delete"))
                .method(Method::DELETE)
                .description("Returns DELETE data"),
        )
        .install(
            crate::service::status_code::status_code,
            route(path!("status" / code))
                .description("Returns given HTTP Status code")
                .add_example_param("code", "418"),
        )
        .install(
            crate::service::auth::basic,
            route(path!("basic-auth" / user / passwd))
                .description("HTTP Basic Auth Challenge")
                .add_example_param("user", "user")
                .add_example_param("passwd", "passwd"),
        )
        .install(
            crate::service::auth::bearer,
            route(path!("bearer-auth" / token))
                .description("Bearer Auth Challenge")
                .add_example_param("token", "random-token"),
        )
        .install(
            crate::service::headers::response_headers,
            route(path!("response-headers"))
                .description("Returns given response headers")
                .add_example_param("key", "val"),
        )
        .install(
            crate::service::redirect::redirect,
            route(path!("redirect" / n))
                .description("302 Redirects n times")
                .add_example_param("n", "5"),
        )
        .install(
            crate::service::redirect::to,
            route(path!("redirect-to"))
                .description("302 Redirects to the url= URL")
                .add_example_param("url", "http://example.com"),
        )
        .install(
            crate::service::redirect::absolute,
            route(path!("absolute-redirect" / n))
                .description("302 Absolute redirects n times")
                .add_example_param("n", "5"),
        )
        .install(
            crate::service::redirect::relative,
            route(path!("relative-redirect" / n))
                .description("302 Relative redirects n times")
                .add_example_param("n", "5"),
        )
        .install(
            crate::service::cookies::cookies,
            route(path!("cookies")).description("Returns cookie data"),
        )
        .install(
            crate::service::cookies::set_cookies,
            route(path!("cookies/set"))
                .description("Sets one or more simple cookies")
                .add_example_param("key", "val"),
        )
        .install(
            crate::service::delay::delay,
            route(path!("delay" / n))
                .description("Delays responding for min(n, 10) seconds")
                .add_example_param("n", "3"),
        )
        .install(
            crate::service::cache::cache,
            route(path!("cache")).description(
                "Returns 200 unless an If-Modified-Since or If-None-Match \
                 header is provided, then it returns a 304",
            ),
        )
        .install(
            crate::service::cache::set_cache,
            route(path!("cache" / n))
                .description("Sets a Cache-Control header for n seconds")
                .add_example_param("n", "10"),
        )
        .install(
            crate::service::bytes::bytes,
            route(path!("bytes" / n))
                .description(
                    "Generates n random bytes of binary data, accepts \
                        optional seed integer parameter",
                )
                .add_example_param("n", "256"),
        )
        .install(
            crate::service::bytes::stream_bytes,
            route(path!("stream-bytes" / n))
                .description(
                    "Streams n random bytes of binary data, accepts \
                        optional seed and chunk_size integer parameters",
                )
                .add_example_param("n", "256"),
        );

    let mut routes = builder.routes();
    let index_route: Route = route(path!()).description("This page").into();
    routes.insert(0, &index_route);

    let index: crate::service::index::Index = routes.into();
    builder.install(index, index_route).build()
}
