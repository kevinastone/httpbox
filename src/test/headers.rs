#![cfg(test)]

use headers::{Header, HeaderMapExt, HeaderValue};

pub fn encode<'a, H: Header>(header: H) -> HeaderValue {
    let mut map = ::http::HeaderMap::new();
    map.typed_insert(header);
    map.get(H::name()).unwrap().clone()
}
