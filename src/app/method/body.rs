use crate::app::response::ok;
use crate::headers::{ContentType, HeaderMapExt};
use crate::http::{Body, Chunk, HeaderMap, Response};
use failure::Fallible;
use futures::compat::Stream01CompatExt;
use futures::prelude::*;
use gotham::handler::HandlerFuture;
use gotham::state::{FromState, State};
use itertools::Itertools;
use std::str;
use url::form_urlencoded;

fn parse_url_encoded_body(raw_body: &[u8]) -> Fallible<String> {
    Ok(form_urlencoded::parse(&raw_body[..])
        .format_with("\n", |(key, value), f| {
            f(&format_args!("{} = {}", key, value))
        })
        .to_string())
}

#[derive(Copy, Clone)]
enum ContentTypeDecoder {
    UrlEncoded,
    Raw,
}

fn content_type_decoder(state: &State) -> ContentTypeDecoder {
    let content_type = HeaderMap::borrow_from(&state)
        .typed_get::<ContentType>()
        .map(mime::Mime::from)
        .unwrap_or(mime::TEXT_PLAIN);

    match (content_type.type_(), content_type.subtype()) {
        (mime::APPLICATION, mime::WWW_FORM_URLENCODED) => {
            ContentTypeDecoder::UrlEncoded
        }
        _ => ContentTypeDecoder::Raw,
    }
}

fn parse_body(state: &State, chunk: &Chunk) -> Fallible<String> {
    match content_type_decoder(&state) {
        ContentTypeDecoder::UrlEncoded => Ok(parse_url_encoded_body(&chunk)?),
        ContentTypeDecoder::Raw => Ok(str::from_utf8(&chunk[..])?.to_string()),
    }
}

async fn _body(mut state: State) -> (State, Response) {
    let body = etry!(
        state,
        Body::take_from(&mut state).compat().try_concat().await
    );
    let content =
        etry!(state, parse_body(&state, &body).map_err(|e| e.compat()));
    ok(state, content)
}

pub fn body(state: State) -> Box<HandlerFuture> {
    async_response!(_body(state))
}

#[cfg(test)]
mod test {
    use super::{
        content_type_decoder, parse_url_encoded_body, ContentTypeDecoder,
    };
    use gotham::state::State;
    use http::{header, HeaderMap};

    #[test]
    fn test_parse_url_encoded_body() {
        assert_eq!(
            parse_url_encoded_body(
                "first=one&second=two&third=three".as_bytes()
            )
            .unwrap(),
            "first = one\nsecond = two\nthird = three"
        )
    }

    #[test]
    fn test_missing_header() {
        State::with_new(|state| {
            state.put(HeaderMap::new());
            match content_type_decoder(&state) {
                ContentTypeDecoder::Raw => (),
                _ => panic!("Incorrect decoder"),
            };
        });
    }

    #[test]
    fn test_form_encoded_header() {
        State::with_new(|state| {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                mime::APPLICATION_WWW_FORM_URLENCODED
                    .to_string()
                    .parse()
                    .unwrap(),
            );
            state.put(headers);

            match content_type_decoder(&state) {
                ContentTypeDecoder::UrlEncoded => (),
                _ => panic!("Incorrect decoder"),
            };
        });
    }

    #[test]
    fn test_form_encoded_with_charset_header() {
        State::with_new(|state| {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static(
                    "application/x-www-form-urlencoded; charset=utf-8",
                ),
            );
            state.put(headers);

            match content_type_decoder(&state) {
                ContentTypeDecoder::UrlEncoded => (),
                _ => panic!("Incorrect decoder"),
            };
        });
    }
}
