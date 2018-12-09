use crate::app::response::ok;
use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::state::{FromState, State};
use http::header;
use hyper::{Body, HeaderMap, StatusCode};
use lazy_static::lazy_static;
use std::error;
use std::fmt;
use std::io;
use url::form_urlencoded;

lazy_static! {
    static ref TEXT_PLAIN: header::HeaderValue =
        header::HeaderValue::from_static("text/plain");
    static ref FORM_URL_ENCODED: header::HeaderValue =
        header::HeaderValue::from_static("application/x-www-form-urlencoded");
}

#[derive(Debug)]
struct BodyParseError(String);

impl fmt::Display for BodyParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BodyParseError: {}", self.0)
    }
}

impl error::Error for BodyParseError {
    fn description(&self) -> &str {
        "Failed to parse the body"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

fn parse_url_encoded_body(raw_body: &[u8]) -> io::Result<String> {
    Ok(form_urlencoded::parse(&raw_body[..])
        .map(|(key, value)| format!("{} = {}", key, value))
        .collect::<Vec<String>>()
        .join("\n"))
}

enum ContentTypeDecoder {
    UrlEncoded,
    Raw,
}

fn content_type_decoder(state: &State) -> ContentTypeDecoder {
    if HeaderMap::borrow_from(&state)
        .get(header::CONTENT_TYPE)
        .unwrap_or(&TEXT_PLAIN)
        .to_str()
        .unwrap()
        == FORM_URL_ENCODED.to_str().unwrap()
    {
        ContentTypeDecoder::UrlEncoded
    } else {
        ContentTypeDecoder::Raw
    }
}

pub fn parse_body(mut state: State) -> Box<HandlerFuture> {
    let f = Body::take_from(&mut state).concat2().then(|raw_body| {
        let valid_body = future_try_or_error_response!(state, raw_body);
        let content = future_try_or_error_response!(
            StatusCode::BAD_REQUEST,
            state,
            match content_type_decoder(&state) {
                ContentTypeDecoder::UrlEncoded => {
                    parse_url_encoded_body(&valid_body)
                        .map_err(|e| BodyParseError(e.to_string()))
                }
                ContentTypeDecoder::Raw => {
                    String::from_utf8(valid_body.to_vec())
                        .map_err(|e| BodyParseError(e.to_string()))
                }
            }
        );
        future::ok(ok(state, content))
    });

    Box::new(f)
}
