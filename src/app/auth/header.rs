use std::fmt;
use std::str::FromStr;

use hyperx::{Error, Result};

const BASIC_REALM_PREAMBLE: &str = "Basic realm=";

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
    type Err = Error;

    fn from_str(s: &str) -> Result<BasicRealm> {
        if s.starts_with(BASIC_REALM_PREAMBLE) {
            Ok(BasicRealm(
                s[BASIC_REALM_PREAMBLE.len() + 1..s.len() - 1].to_owned(),
            ))
        } else {
            Err(Error::Header)
        }
    }
}

#[cfg(test)]
mod test {
    use super::BasicRealm;
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
}
