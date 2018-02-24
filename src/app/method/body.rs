extern crate gotham;
extern crate hyper;
extern crate mime;

use app::response::ok;
use futures::{future, Future, Stream};
use gotham::state::{FromState, State};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use hyper::{Body, Headers, StatusCode};
use hyper::header::ContentType;
use std::error;
use std::fmt;
use std::io;
use url::form_urlencoded;

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

fn parse_url_encoded_body(raw_body: Vec<u8>) -> io::Result<String> {
    Ok(form_urlencoded::parse(&raw_body[..])
        .map(|(key, value)| format!("{} = {}", key, value))
        .collect::<Vec<String>>()
        .join("\n"))
}

enum ContentTypeDecoder {
    UrlEncoded,
    Raw,
}

fn content_type_decoder(mut state: &State) -> ContentTypeDecoder {
    if Headers::borrow_from(&mut state)
        .get::<ContentType>()
        .unwrap_or(&ContentType::plaintext())
        == &ContentType::form_url_encoded()
    {
        ContentTypeDecoder::UrlEncoded
    } else {
        ContentTypeDecoder::Raw
    }
}

pub fn parse_body(mut state: State) -> Box<HandlerFuture> {
    let f = Body::take_from(&mut state).concat2().then(
        |raw_body| match raw_body {
            Err(e) => return future::err((state, e.into_handler_error())),
            Ok(valid_body) => {
                let payload = match content_type_decoder(&mut state) {
                    ContentTypeDecoder::UrlEncoded => {
                        parse_url_encoded_body(valid_body.to_vec())
                            .map_err(|e| BodyParseError(e.to_string()))
                    }
                    ContentTypeDecoder::Raw => {
                        String::from_utf8(valid_body.to_vec())
                            .map_err(|e| BodyParseError(e.to_string()))
                    }
                };
                match payload {
                    Ok(content) => {
                        return future::ok(ok(state, content.into_bytes()))
                    }
                    Err(e) => {
                        return future::err((
                            state,
                            e.into_handler_error()
                                .with_status(StatusCode::BadRequest),
                        ))
                    }
                }
            }
        },
    );

    Box::new(f)
}
