use itertools::EitherOrBoth;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::iter::FromIterator;
use std::ops::Deref;

use lazy_static::lazy_static;

lazy_static! {
    static ref EMPTY_HASHMAP: HashMap<&'static str, String> = HashMap::new();
}

#[derive(Debug, Clone)]
pub enum Path {
    Literal(&'static str),
    Segmented(SegmentedPath),
}

fn segmented(str: &str) -> impl Iterator<Item = &str> {
    str.split('/').filter(|seg| !seg.is_empty())
}

impl Path {
    pub fn matches(&self, path: &str) -> Option<MatchedPath> {
        match self {
            Self::Literal(str) => {
                if str == &path {
                    Some(MatchedPath::Literal)
                } else {
                    None
                }
            }
            Self::Segmented(segmented_path) => segmented_path.matches(path),
        }
    }

    pub fn segmented(segments: Vec<PathSegment>) -> Self {
        Self::Segmented(SegmentedPath(segments))
    }

    pub fn to_uri(
        &self,
        params: &BTreeMap<&'static str, &'static str>,
    ) -> Option<PathAndQuery> {
        match self {
            Self::Literal(str) => {
                Some(PathAndQuery::from_static(str).with_query(params.clone()))
            }
            Self::Segmented(segmented_path) => segmented_path.to_uri(&params),
        }
    }
}

impl From<&'static str> for Path {
    fn from(str: &'static str) -> Self {
        Self::Literal(str)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Literal(str) => write!(f, "{}", str),
            Self::Segmented(segmented_path) => write!(f, "{}", segmented_path),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SegmentedPath(Vec<PathSegment>);

impl fmt::Display for SegmentedPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "/{}",
            self.iter()
                .format_with("/", |segment, f| f(&format_args!("{}", segment)))
        )
    }
}

impl SegmentedPath {
    fn matches(&self, path: &str) -> Option<MatchedPath> {
        let mut params = HashMap::new();
        for el in self.iter().zip_longest(segmented(path)) {
            match el {
                EitherOrBoth::Both(expected, actual) => {
                    if !expected.matches(actual) {
                        return None;
                    }
                    if let PathSegment::Param(name) = expected {
                        params.insert(*name, actual.to_owned());
                    }
                }
                _ => return None,
            }
        }
        Some(MatchedPath::Segmented(params))
    }

    pub fn to_uri(
        &self,
        params: &BTreeMap<&'static str, &'static str>,
    ) -> Option<PathAndQuery> {
        let mut segments = vec![];
        let mut params = params.clone();

        segments.push(""); // Workaround for leading slash

        for segment in self.iter() {
            match segment {
                PathSegment::Literal(str) => segments.push(*str),
                PathSegment::Param(name) => {
                    let value = params.remove(name)?;
                    segments.push(&value)
                }
            }
        }

        Some(PathAndQuery::new(segments).with_query(params))
    }
}

impl Deref for SegmentedPath {
    type Target = Vec<PathSegment>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    Literal(&'static str),
    Param(&'static str),
}

impl PathSegment {
    pub fn matches(&self, path: &str) -> bool {
        match self {
            Self::Literal(str) => str == &path,
            Self::Param(_) => true,
        }
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Literal(str) => write!(f, "{}", str),
            Self::Param(name) => write!(f, ":{}", name),
        }
    }
}

#[derive(Debug)]
pub enum MatchedPath {
    Literal,
    Segmented(HashMap<&'static str, String>),
}

impl MatchedPath {
    pub fn params(&self) -> &HashMap<&'static str, String> {
        match self {
            Self::Literal => &EMPTY_HASHMAP,
            Self::Segmented(params) => params,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathAndQuery<'a> {
    segments: Vec<&'a str>,
    query: Vec<(&'static str, &'a str)>,
}

impl<'a> PathAndQuery<'a> {
    pub fn new(segments: Vec<&'a str>) -> Self {
        Self {
            segments,
            query: vec![],
        }
    }

    pub fn from_static(input: &'static str) -> Self {
        Self::new(vec![input])
    }

    pub fn with_query(
        mut self,
        query: impl IntoIterator<Item = (&'static str, &'a str)>,
    ) -> Self {
        self.query = Vec::from_iter(query);
        self
    }
}

impl<'a> fmt::Display for PathAndQuery<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.segments
                .iter()
                .format_with("/", |segment, f| f(&format_args!("{}", segment)))
        )?;

        if !self.query.is_empty() {
            write!(
                f,
                "?{}",
                self.query.iter().format_with("&", |(k, v), f| f(
                    &format_args!("{}={}", k, v)
                ))
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_to_uri_literal() {
        let path: Path = "/test".into();
        let params = BTreeMap::new();
        assert_eq!(path.to_uri(&params).unwrap().to_string(), "/test");
    }

    #[test]
    fn test_to_uri_literal_with_single_param() {
        let path: Path = "/test".into();
        let mut params = BTreeMap::new();
        params.insert("first", "value");
        assert_eq!(
            path.to_uri(&params).unwrap().to_string(),
            "/test?first=value"
        );
    }
    #[test]
    fn test_to_uri_literal_with_multiple_params() {
        let path: Path = "/test".into();
        let mut params = BTreeMap::new();
        params.insert("first", "value");
        params.insert("second", "another");
        assert_eq!(
            path.to_uri(&params).unwrap().to_string(),
            "/test?first=value&second=another"
        );
    }

    #[test]
    fn test_to_uri_segmented_missing_param() {
        let path: Path = path!("test" / param);
        let params = BTreeMap::new();
        assert!(path.to_uri(&params).is_none());
    }

    #[test]
    fn test_to_uri_segmented_with_path_param() {
        let path: Path = path!("test" / param);
        let mut params = BTreeMap::new();
        params.insert("param", "value");
        assert_eq!(path.to_uri(&params).unwrap().to_string(), "/test/value");
    }

    #[test]
    fn test_to_uri_segmented_with_extra_params() {
        let path: Path = path!("test" / param);
        let mut params = BTreeMap::new();
        params.insert("param", "value");
        params.insert("first", "value");
        params.insert("second", "another");
        assert_eq!(
            path.to_uri(&params).unwrap().to_string(),
            "/test/value?first=value&second=another"
        );
    }
}
