extern crate modifier;

use iron::Response;
use iron::headers::ContentLength;
use self::modifier::Modifier;
use iron::response::{ResponseBody, WriteBody};
use std::io::{self, Write};

pub struct StreamResponse {
    data: Vec<u8>,
    chunk_size: usize,
}

impl StreamResponse {
    pub fn new(data: Vec<u8>, chunk_size: usize) -> Self {
        StreamResponse {
            data: data,
            chunk_size: chunk_size,
        }
    }
}

impl WriteBody for StreamResponse {
    fn write_body(&mut self, res: &mut ResponseBody) -> io::Result<()> {

        for chunk in self.data.chunks(self.chunk_size) {
            try!(res.write(chunk));
            try!(res.flush());
        }

        Ok(())
    }
}

impl Modifier<Response> for StreamResponse {
    fn modify(self, res: &mut Response) {
        res.headers.set(ContentLength(self.data.len() as u64));
        res.body = Some(Box::new(self));
    }
}
