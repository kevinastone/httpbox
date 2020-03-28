pub use hyper::http::{StatusCode, Uri};
pub use hyper::{body::Bytes, Body};

mod error;
mod request;
mod response;
mod stream;

pub use self::error::Error;
pub use self::request::*;
pub use self::response::*;
pub(crate) use self::stream::*;

pub type Result = std::result::Result<Response, Error>;
