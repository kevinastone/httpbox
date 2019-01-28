#![cfg(test)]

use gotham::test::TestRequest;
use headers::{Header, HeaderMapExt};

pub trait TestRequestTypedHeader {
    fn with_typed_header<H: Header>(self, header: H) -> Self;
}

impl TestRequestTypedHeader for TestRequest<'_> {
    fn with_typed_header<H: Header>(mut self, header: H) -> Self {
        self.headers_mut().typed_insert(header);
        self
    }
}
