#![allow(clippy::result_large_err)]

pub use hyper::body::Bytes;
pub use hyper::http::{StatusCode, Uri};

mod error;
mod request;
mod response;
mod stream;

pub use self::error::Error;
pub use self::request::*;
pub use self::response::*;
pub(crate) use self::stream::*;
pub use hyper_body::Body;

pub type Result = std::result::Result<Response, Error>;
