use failure::{Error, Fail, Fallible};
use headers_ext::{Header, HeaderName, HeaderValue, ToValues, Values};
use http::header;
use std::fmt;
use std::str::FromStr;

const BASIC_REALM_PREAMBLE: &str = "Basic realm=";

#[derive(Clone, Debug, PartialEq)]
pub struct WWWAuthenticate(BasicRealm);

impl WWWAuthenticate {
    pub fn basic_realm(realm: &str) -> Self {
        WWWAuthenticate(BasicRealm(realm.to_owned()))
    }
}

impl Header for WWWAuthenticate {
    const NAME: &'static HeaderName = &header::WWW_AUTHENTICATE;

    fn decode(values: &mut Values) -> Option<Self> {
        values
            .next()?
            .to_str()
            .ok()?
            .parse()
            .ok()
            .map(WWWAuthenticate)
    }

    fn encode(&self, values: &mut ToValues) {
        values.append((&self.0).into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BasicRealm(String);

impl From<&BasicRealm> for HeaderValue {
    fn from(realm: &BasicRealm) -> Self {
        format!("{}", realm).parse().unwrap()
    }
}

impl fmt::Display for BasicRealm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(BASIC_REALM_PREAMBLE)?;
        f.write_str("\"")?;
        f.write_str(&self.0)?;
        f.write_str("\"")?;
        Ok(())
    }
}

#[derive(Debug, Fail)]
enum BasicRealmError {
    #[fail(display = "invalid premable")]
    InvalidPreamble,
}

impl FromStr for BasicRealm {
    type Err = Error;

    fn from_str(s: &str) -> Fallible<BasicRealm> {
        if s.starts_with(BASIC_REALM_PREAMBLE) {
            Ok(BasicRealm(
                s[BASIC_REALM_PREAMBLE.len() + 1..s.len() - 1].to_owned(),
            ))
        } else {
            Err(BasicRealmError::InvalidPreamble)?
        }
    }
}

#[cfg(test)]
mod test {
    use super::BasicRealm;

    #[test]
    fn test_encode_basic_realm() {
        assert_eq!(
            format!("{}", BasicRealm(String::from("Test Realm"))),
            "Basic realm=\"Test Realm\""
        )
    }

    #[test]
    fn test_parse_basic_realm() {
        assert_eq!(
            "Basic realm=\"Test Realm\"".parse::<BasicRealm>().unwrap(),
            BasicRealm(String::from("Test Realm")),
        )
    }

    #[test]
    fn test_parse_err_basic_realm() {
        assert!("Missing realm=\"Test Realm\""
            .parse::<BasicRealm>()
            .is_err())
    }
}
