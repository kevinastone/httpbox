extern crate gotham;
extern crate hyper;
extern crate mime;

use hyper::{header, Result as HyperResult};
use std::fmt;
use std::str::FromStr;

pub const WWW_AUTHENTICATE: &'static str = "WWW-Authenticate";
const BASIC_REALM_PREAMBLE: &'static str = "Basic realm=";

#[derive(Clone, Debug, PartialEq)]
pub struct BasicRealm(pub String);

impl fmt::Display for BasicRealm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(BASIC_REALM_PREAMBLE)?;
        f.write_str("\"")?;
        f.write_str(&self.0)?;
        f.write_str("\"")?;
        Ok(())
    }
}

impl FromStr for BasicRealm {
    type Err = String;

    fn from_str(s: &str) -> Result<BasicRealm, String> {
        if s.starts_with(BASIC_REALM_PREAMBLE) {
            Ok(BasicRealm(
                s[BASIC_REALM_PREAMBLE.len() + 1..s.len() - 1].to_owned(),
            ))
        } else {
            Err(format!("Unable to parse header"))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WWWAuthenticate(pub BasicRealm);

impl header::Header for WWWAuthenticate {
    fn header_name() -> &'static str {
        WWW_AUTHENTICATE
    }

    fn parse_header(raw: &header::Raw) -> HyperResult<Self> {
        header::parsing::from_one_raw_str(raw).map(WWWAuthenticate)
    }

    fn fmt_header(&self, f: &mut header::Formatter) -> fmt::Result {
        f.fmt_line(&self.0)
    }
}

#[cfg(test)]
mod test {
    use super::{BasicRealm, WWWAuthenticate};

    use hyper::header::{Header, Raw};
    use std::str::FromStr;

    #[test]
    fn test_format_basic_realm() {
        assert_eq!(
            format!("{}", BasicRealm(String::from("Test Realm"))),
            "Basic realm=\"Test Realm\""
        )
    }

    #[test]
    fn test_parse_basic_realm() {
        assert_eq!(
            BasicRealm::from_str("Basic realm=\"Test Realm\"").unwrap(),
            BasicRealm(String::from("Test Realm")),
        )
    }

    #[test]
    fn test_parse_err_basic_realm() {
        assert!(BasicRealm::from_str("Missing realm=\"Test Realm\"").is_err())
    }

    #[test]
    fn test_parse_www_authenticate() {
        let raw = Raw::from("Basic realm=\"Test Realm\"");
        assert!(WWWAuthenticate::parse_header(&raw).is_ok())
    }

    #[test]
    fn test_parse_err_www_authenticate() {
        let raw = Raw::from("Missing realm=\"Test Realm\"");
        assert!(WWWAuthenticate::parse_header(&raw).is_err())
    }
}
