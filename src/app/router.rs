use crate::app::auth::{BasicAuthParams, BearerParams};
use crate::app::bytes::{BytesPathParams, BytesQueryParams};
use crate::app::cache::CacheTimeParams;
use crate::app::delay::DelayParams;
use crate::app::index::IndexNewHandler;
use crate::app::redirect::{RedirectCountParams, RedirectUrlParams};
use crate::app::status_code::StatusCodeParams;
use crate::app::*;
use crate::router::*;
use hyper::Method;

pub fn app() -> Router {
    build_simple_router(|root: &mut RouterBuilder<(), ()>| {
        let mut installer = RouteInstaller::new(root);
        installer.install(
            ip::ip,
            Route::new("/ip").set_description("Returns Origin IP"),
        );
        installer.install(
            user_agent::user_agent,
            Route::new("/user-agent").set_description("Returns user-agent"),
        );
        installer.install(
            headers::headers,
            Route::new("/headers").set_description("Returns headers"),
        );
        installer.install(
            method::get,
            Route::new("/get")
                .set_description("Returns GET data")
                .add_example_param("key", "val"),
        );
        installer.install(
            method::post,
            Route::new("/post")
                .set_method(Method::POST)
                .set_description("Returns POST data"),
        );
        installer.install(
            method::patch,
            Route::new("/patch")
                .set_method(Method::PATCH)
                .set_description("Returns PUT data"),
        );
        installer.install(
            method::put,
            Route::new("/put")
                .set_method(Method::PUT)
                .set_description("Returns PUT data"),
        );
        installer.install(
            method::delete,
            Route::new("/delete")
                .set_method(Method::DELETE)
                .set_description("Returns DELETE data"),
        );
        installer.install_with_path_extractor::<_, _, StatusCodeParams>(
            status_code::status_code,
            Route::new("/status/:code")
                .set_description("Returns given HTTP Status code")
                .add_example_param("code", "418"),
        );
        installer.install_with_path_extractor::<_, _, BasicAuthParams>(
            auth::basic,
            Route::new("/basic-auth/:user/:passwd")
                .set_description("HTTP Basic Auth Challenge")
                .add_example_param("user", "user")
                .add_example_param("passwd", "passwd"),
        );
        installer.install_with_path_extractor::<_, _, BearerParams>(
            auth::bearer,
            Route::new("/bearer-auth/:token")
                .set_description("Bearer Auth Challenge")
                .add_example_param("token", "random-token"),
        );
        installer.install(
            headers::response_headers,
            Route::new("/response-headers")
                .set_description("Returns given response headers")
                .add_example_param("key", "val"),
        );
        installer.install_with_path_extractor::<_, _, RedirectCountParams>(
            redirect::redirect,
            Route::new("/redirect/:n")
                .set_description("302 Redirects n times")
                .add_example_param("n", "5"),
        );
        installer.install_with_query_extractor::<_, _, RedirectUrlParams>(
            redirect::to,
            Route::new("/redirect-to")
                .set_description("302 Redirects to the url= URL")
                .add_example_param("url", "http://example.com"),
        );
        installer.install_with_path_extractor::<_, _, RedirectCountParams>(
            redirect::absolute,
            Route::new("/absolute-redirect/:n")
                .set_description("302 Absolute redirects n times")
                .add_example_param("n", "5"),
        );
        installer.install_with_path_extractor::<_, _, RedirectCountParams>(
            redirect::relative,
            Route::new("/relative-redirect/:n")
                .set_description("302 Relative redirects n times")
                .add_example_param("n", "5"),
        );
        installer.install(
            cookies::cookies,
            Route::new("/cookies").set_description("Returns cookie data"),
        );
        installer.install(
            cookies::set_cookies,
            Route::new("/cookies/set")
                .set_description("Sets one or more simple cookies"),
        );
        installer.install_with_path_extractor::<_, _, DelayParams>(
            delay::delay,
            Route::new("/delay/:n")
                .set_description("Delays responding for min(n, 10) seconds")
                .add_example_param("n", "3"),
        );
        installer.install(
            cache::cache,
            Route::new("/cache").set_description(
                "Returns 200 unless an If-Modified-Since or If-None-Match \
                 header is provided, then it returns a 304",
            ),
        );
        installer.install_with_path_extractor::<_, _, CacheTimeParams>(
            cache::set_cache,
            Route::new("/cache/:n")
                .set_description("Sets a Cache-Control header for n seconds")
                .add_example_param("n", "10"),
        );
        installer
            .install_with_path_and_query_extractor::<_, _,
                BytesPathParams,
                BytesQueryParams
            >(
                bytes::bytes,
                Route::new("/bytes/:n")
                    .set_description(
                        "Generates n random bytes of binary data, accepts \
                        optional seed integer parameter",
                    )
                    .add_example_param("n", "256"),
            );
        installer
            .install_with_path_and_query_extractor::<_, _,
                BytesPathParams,
                BytesQueryParams
            >(

                bytes::stream_bytes,
                Route::new("/stream-bytes/:n")
                    .set_description(
                        "Streams n random bytes of binary data, accepts \
                        optional seed and chunk_size integer parameters",
                    )
                    .add_example_param("n", "256"),
            );

        let mut routes = installer.routes();
        installer.closure(
            Route::new("/").set_description("This page"),
            move |route, builder| {
                routes.insert(0, route.clone());
                let index_handler = IndexNewHandler::from(&routes[..]);
                builder.get(route.path()).to_new_handler(index_handler);
            },
        );
    })
}
