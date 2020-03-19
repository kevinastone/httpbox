use crate::headers::ContentLength;
use crate::headers::ContentType;
use crate::http::{
    bad_request, body_from_stream, HTTPResponse, Request,
    ResponseTypedHeaderExt, Result,
};
use crate::random::rng;
use futures::prelude::*;
use rand::Rng;
use serde_derive::Deserialize;
use std::iter::ExactSizeIterator;

#[derive(Deserialize)]
pub struct BytesQueryParams {
    seed: Option<u32>,
    chunk_size: Option<usize>,
}

pub fn iter_bytes(
    count: u32,
    seed: Option<u32>,
) -> impl ExactSizeIterator<Item = u8> {
    let mut rng = rng(seed);
    (0..count).map(move |_| rng.gen::<u8>())
}

pub async fn bytes(req: Request) -> Result {
    let n = req.param::<u32>("n").ok_or_else(bad_request)?;
    let query = req.query::<BytesQueryParams>().map_err(|_| bad_request())?;

    let data = iter_bytes(n, query.seed).collect::<Vec<u8>>();

    HTTPResponse::builder()
        .typed_header(ContentType::octet_stream())
        .body(data.into())
        .map_err(Into::into)
}

pub async fn stream_bytes(req: Request) -> Result {
    let n = req.param::<u32>("n").ok_or_else(bad_request)?;
    let query = req.query::<BytesQueryParams>().map_err(|_| bad_request())?;

    let data = iter_bytes(n, query.seed);
    let chunk_size = query.chunk_size;
    let content_length = data.len() as u64;

    HTTPResponse::builder()
        .typed_header(ContentType::octet_stream())
        .typed_header(ContentLength(content_length))
        .body(body_from_stream(
            stream::iter(data).chunks(chunk_size.unwrap_or(1)),
        ))
        .map_err(Into::into)
}
