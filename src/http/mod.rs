use hyper::http::Response as HTTPResponse;
pub use hyper::http::{HeaderMap, StatusCode, Uri};
pub use hyper::{body::Bytes, Body};

mod request;
mod response;
mod stream;

pub use request::*;
pub use response::*;
pub(crate) use stream::*;

pub type Response = HTTPResponse<Body>;

#[derive(Debug)]
pub enum HandlerError {
    HyperError(hyper::http::Error),
    Failure(Response),
}

impl HandlerError {
    pub fn into_result(self) -> hyper::http::Result<Response> {
        match self {
            Self::HyperError(e) => Err(e),
            Self::Failure(res) => Ok(res),
        }
    }
}

impl From<hyper::http::Result<Response>> for HandlerError {
    fn from(result: hyper::http::Result<Response>) -> Self {
        match result {
            Ok(res) => Self::Failure(res),
            Err(e) => Self::HyperError(e),
        }
    }
}

impl From<hyper::http::Error> for HandlerError {
    fn from(error: hyper::http::Error) -> Self {
        HandlerError::HyperError(error)
    }
}

pub type Result = std::result::Result<Response, HandlerError>;
