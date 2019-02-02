pub use http::{HeaderMap, Response as HTTPResponse, StatusCode, Uri};
pub use hyper::{Body, Chunk};

pub type Response = HTTPResponse<Body>;
