#![cfg(test)]

pub fn encode<'a, H: ::headers::Header>(
    header: H,
) -> ::http::header::HeaderValue {
    use headers::HeaderMapExt;
    let mut map = ::http::HeaderMap::new();
    map.typed_insert(header);
    map.get(H::name()).unwrap().clone()
}
