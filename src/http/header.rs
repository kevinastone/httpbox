use crate::headers::{Header, HeaderMapExt};

pub trait ResponseTypedHeaderExt {
    fn typed_header<H: Header>(self, header: H) -> Self;
}

impl ResponseTypedHeaderExt for hyper::http::response::Builder {
    fn typed_header<H: Header>(mut self, header: H) -> Self {
        self.headers_mut().unwrap().typed_insert(header);
        self
    }
}
