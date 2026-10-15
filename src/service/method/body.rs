use crate::headers::ContentType;
use crate::http::{Bytes, Request, Result, bad_request, ok, payload_too_large};
use http_body_util::{BodyExt, Limited};
use itertools::Itertools;
use std::str;

pub const MAX_BODY_SIZE: usize = 2 * 1024 * 1024; // 2 MB

fn parse_url_encoded_body(raw_body: &[u8]) -> anyhow::Result<String> {
    Ok(
        serde_urlencoded::from_bytes::<Vec<(String, String)>>(raw_body)?
            .iter()
            .format_with("\n", |(key, value), f| {
                f(&format_args!("{key} = {value}"))
            })
            .to_string(),
    )
}

#[derive(Copy, Clone)]
enum ContentTypeDecoder {
    UrlEncoded,
    Raw,
}

fn content_type_decoder(req: &Request) -> ContentTypeDecoder {
    let content_type = req
        .typed_header::<ContentType>()
        .map(mime::Mime::from)
        .unwrap_or(mime::TEXT_PLAIN);

    match (content_type.type_(), content_type.subtype()) {
        (mime::APPLICATION, mime::WWW_FORM_URLENCODED) => {
            ContentTypeDecoder::UrlEncoded
        }
        _ => ContentTypeDecoder::Raw,
    }
}

fn parse_body(req: &Request, chunk: &Bytes) -> anyhow::Result<String> {
    match content_type_decoder(req) {
        ContentTypeDecoder::UrlEncoded => Ok(parse_url_encoded_body(chunk)?),
        ContentTypeDecoder::Raw => Ok(str::from_utf8(&chunk[..])?.to_string()),
    }
}

pub async fn body(mut req: Request) -> Result {
    let req_body = std::mem::take(req.body_mut());
    let collected = Limited::new(req_body, MAX_BODY_SIZE)
        .collect()
        .await
        .map_err(|err| {
            if err
                .downcast_ref::<http_body_util::LengthLimitError>()
                .is_some()
            {
                payload_too_large()
            } else {
                bad_request()
            }
        })?;
    let body = collected.to_bytes();
    let content = parse_body(&req, &body).map_err(|_| bad_request())?;
    ok(content)
}

#[cfg(test)]
mod test {
    use super::{
        ContentTypeDecoder, content_type_decoder, parse_url_encoded_body,
    };
    use crate::headers::ContentType;
    use crate::test::*;

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
        let req = request().build();

        match content_type_decoder(&req) {
            ContentTypeDecoder::Raw => (),
            _ => panic!("Incorrect decoder"),
        };
    }

    #[test]
    fn test_form_encoded_header() {
        let req = request()
            .typed_header(ContentType::form_url_encoded())
            .build();

        match content_type_decoder(&req) {
            ContentTypeDecoder::UrlEncoded => (),
            _ => panic!("Incorrect decoder"),
        };
    }

    #[test]
    fn test_form_encoded_with_charset_header() {
        let req = request()
            .header(
                hyper::http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=utf-8",
            )
            .build();

        match content_type_decoder(&req) {
            ContentTypeDecoder::UrlEncoded => (),
            _ => panic!("Incorrect decoder"),
        };
    }

    #[tokio::test]
    async fn test_body_payload_too_large() {
        let large_body = vec![b'a'; super::MAX_BODY_SIZE + 1];
        let res = request()
            .body(large_body)
            .handle(super::body)
            .await
            .unwrap();

        assert_eq!(res.status(), hyper::http::StatusCode::PAYLOAD_TOO_LARGE);
    }
}
