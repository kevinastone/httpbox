extern crate router;

use self::router::Router;

mod bytes;
mod hello;
mod random;
mod status_code;
mod stream;
mod user_agent;

pub fn app() -> Router {
    let mut router = Router::new();
    router.get("/", hello::hello);
    router.get("/bytes/:n", bytes::bytes);
    router.get("/stream-bytes/:n", bytes::stream_bytes);
    router.get("/status/:code", status_code::status_code);
    router.get("/user-agent", user_agent::user_agent);

    router
}
