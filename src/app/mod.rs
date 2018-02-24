extern crate gotham;

mod auth;
mod bytes;
mod cache;
mod cookies;
mod delay;
mod headers;
mod index;
mod ip;
mod method;
mod random;
mod redirect;
mod response;
mod router;
mod status_code;
mod user_agent;

pub use self::router::router;
