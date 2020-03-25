use crate::headers::ContentType;
use crate::http::{bad_request, ok, Bytes, Request, Result};
use itertools::Itertools;
use std::str;

fn parse_url_encoded_body(raw_body: &[u8]) -> anyhow::Result<String> {
    Ok(
        serde_urlencoded::from_bytes::<Vec<(String, String)>>(&raw_body[..])?
            .iter()
            .format_with("\n", |(key, value), f| {
                f(&format_args!("{} = {}", key, value))
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
    match content_type_decoder(&req) {
        ContentTypeDecoder::UrlEncoded => Ok(parse_url_encoded_body(&chunk)?),
        ContentTypeDecoder::Raw => Ok(str::from_utf8(&chunk[..])?.to_string()),
    }
}

pub async fn body(mut req: Request) -> Result {
    let body = hyper::body::to_bytes(req.body())
        .await
        .map_err(|_| bad_request())?;
    let content = parse_body(&req, &body).map_err(|_| bad_request())?;
    ok(content)
}

#[cfg(test)]
mod test {
    use super::{
        content_type_decoder, parse_url_encoded_body, ContentTypeDecoder,
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
}
