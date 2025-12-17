#[macro_use]
mod macros;

use itertools::EitherOrBoth;
use itertools::Itertools;
#[cfg(feature = "regex")]
pub use regex;
use serde::Deserialize;
use serde::de::IntoDeserializer;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

fn segmented(str: &str) -> impl Iterator<Item = &str> {
    str.split('/').filter(|seg| !seg.is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(transparent)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct PathMatch(HashMap<&'static str, String>);

impl From<HashMap<&'static str, String>> for PathMatch {
    fn from(hashmap: HashMap<&'static str, String>) -> Self {
        Self(hashmap)
    }
}

impl Deref for PathMatch {
    type Target = HashMap<&'static str, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PathMatch {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'de, E> IntoDeserializer<'de, E> for PathMatch
where
    E: serde::de::Error,
{
    type Deserializer = <HashMap<&'static str, String> as IntoDeserializer<
        'de,
        E,
    >>::Deserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        self.0.into_deserializer()
    }
}

#[derive(Debug, Clone)]
pub struct Path(pub Vec<PathSegment>);

impl Path {
    pub fn matches(&self, path: &str) -> Option<PathMatch> {
        let mut params = PathMatch::default();
        for el in self.iter().zip_longest(segmented(path)) {
            match el {
                EitherOrBoth::Both(expected, actual) => {
                    if !expected.matches(actual) {
                        return None;
                    }
                    if let PathSegment::Dynamic(param) = expected {
                        params.insert(param.name, actual.to_owned());
                    }
                }
                _ => return None,
            }
        }
        Some(params)
    }

    pub fn replace(
        &self,
        params: &BTreeMap<&'static str, &'static str>,
    ) -> Option<PathAndQuery<'_>> {
        let mut segments = vec![];
        let mut params = params.clone();

        segments.push(""); // Workaround for leading slash

        for segment in self.iter() {
            match segment {
                PathSegment::Literal(str) => segments.push(str),
                PathSegment::Dynamic(param) => {
                    let value = params.remove(param.name)?;
                    segments.push(value)
                }
            }
        }

        Some(PathAndQuery::new(segments).with_query(params))
    }
}

impl Deref for Path {
    type Target = Vec<PathSegment>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "/{}",
            self.iter().format_with("/", |segment, f| f(segment))
        )
    }
}

impl From<&'static str> for Path {
    fn from(str: &'static str) -> Self {
        Self(segmented(str).map(PathSegment::Literal).collect())
    }
}

#[derive(Debug, Clone)]
pub enum PathToken {
    Any,
    #[cfg(feature = "regex")]
    Regex(regex::Regex),
}

impl PathToken {
    pub fn matches(&self, path: &str) -> bool {
        match self {
            Self::Any => true,
            #[cfg(feature = "regex")]
            Self::Regex(re) => re.is_match(path),
        }
    }
}

impl fmt::Display for PathToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Any => write!(f, "*"),
            #[cfg(feature = "regex")]
            Self::Regex(re) => write!(f, "{}", re),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PathParam {
    name: &'static str,
    token: PathToken,
}

impl PathParam {
    pub fn new(name: &'static str, token: PathToken) -> Self {
        Self { name, token }
    }
}

impl fmt::Display for PathParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, ":{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    Literal(&'static str),
    Dynamic(PathParam),
}

impl PathSegment {
    fn matches(&self, path: &str) -> bool {
        match self {
            Self::Literal(str) => str == &path,
            Self::Dynamic(param) => param.token.matches(path),
        }
    }
}

impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Literal(str) => write!(f, "{}", str),
            Self::Dynamic(param) => write!(f, "{}", param),
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

    pub fn with_query(
        mut self,
        query: impl IntoIterator<Item = (&'static str, &'a str)>,
    ) -> Self {
        self.query = Vec::from_iter(query);
        self
    }
}

impl fmt::Display for PathAndQuery<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.segments
                .iter()
                .format_with("/", |segment, f| f(segment))
        )?;

        if !self.query.is_empty() {
            write!(
                f,
                "?{}",
                self.query
                    .iter()
                    .format_with("&", |(k, v), f| f(&format_args!("{k}={v}")))
            )?;
        }

        Ok(())
    }
}
