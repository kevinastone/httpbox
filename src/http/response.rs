use super::{
    Body, HTTPResponse, HandlerError, ResponseTypedHeaderExt, Result,
    StatusCode, Uri,
};
use crate::headers::{ContentType, Location};

pub fn html<B: Into<Body>>(body: B) -> Result {
    HTTPResponse::builder()
        .typed_header(ContentType::html())
        .body(body.into())
        .map_err(Into::into)
}

pub fn ok<B>(body: B) -> Result
where
    B: Into<Body>,
{
    HTTPResponse::builder()
        .typed_header(ContentType::text())
        .body(body.into())
        .map_err(Into::into)
}

pub fn not_found() -> HandlerError {
    HTTPResponse::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .into()
}

pub fn bad_request() -> HandlerError {
    HTTPResponse::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())
        .into()
}

pub fn empty_response(status: StatusCode) -> Result {
    HTTPResponse::builder()
        .status(status)
        .body(Body::empty())
        .map_err(Into::into)
}

pub fn redirect_to(uri: Uri) -> Result {
    HTTPResponse::builder()
        .status(StatusCode::FOUND)
        .typed_header(Location::from(uri))
        .body(Body::empty())
        .map_err(Into::into)
}
