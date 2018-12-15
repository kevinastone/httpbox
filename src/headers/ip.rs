use headers_ext::{Error, HeaderValue};
use std::net::IpAddr;

pub const X_FORWARDED_FOR: &str = "X-Forwarded-For";

pub struct XForwardedFor(pub IpAddr);

impl XForwardedFor {
    pub fn ip_addr(&self) -> IpAddr {
        self.0
    }

    pub fn try_for(value: &HeaderValue) -> Result<Self, Error> {
        Some(value)
            .and_then(|v| v.to_str().ok()?.parse().ok())
            .map(XForwardedFor)
            .ok_or_else(Error::invalid)
    }
}
