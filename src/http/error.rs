use super::Response;

#[derive(Debug)]
pub enum Error {
    HyperError(hyper::http::Error),
    Failure(Response),
}

impl Error {
    pub async fn into_result(self) -> hyper::http::Result<Response> {
        match self {
            Self::HyperError(e) => Err(e),
            Self::Failure(res) => Ok(res),
        }
    }
}

impl From<hyper::http::Result<Response>> for Error {
    fn from(result: hyper::http::Result<Response>) -> Self {
        match result {
            Ok(res) => Self::Failure(res),
            Err(e) => Self::HyperError(e),
        }
    }
}

impl From<hyper::http::Error> for Error {
    fn from(error: hyper::http::Error) -> Self {
        Error::HyperError(error)
    }
}
