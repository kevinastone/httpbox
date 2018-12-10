#![cfg(test)]

pub fn encode<'a, H: ::headers_ext::Header>(
    header: H,
) -> ::http::header::HeaderValue {
    use headers_ext::HeaderMapExt;
    let mut map = ::http::HeaderMap::new();
    map.typed_insert(header);
    map.get(H::NAME).unwrap().clone()
}
