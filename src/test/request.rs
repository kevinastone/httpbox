#![cfg(test)]

use gotham::test::{Server, TestRequest};
use headers::{Header, HeaderMapExt};
use hyper::client::connect::Connect;

pub trait TestRequestTypedHeader {
    fn with_typed_header<H: Header>(self, header: H) -> Self;
}

impl<S: Server, C: Connect> TestRequestTypedHeader for TestRequest<'_, S, C> {
    fn with_typed_header<H: Header>(mut self, header: H) -> Self {
        self.headers_mut().typed_insert(header);
        self
    }
}
