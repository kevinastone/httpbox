use itertools::EitherOrBoth;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt;
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
