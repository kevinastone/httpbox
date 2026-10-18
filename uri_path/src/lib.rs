#[macro_use]
mod macros;
pub mod router;

pub use self::router::Router;

#[cfg(feature = "regex")]
#[doc(hidden)]
pub use lazy_regex;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::iter::FromIterator;
use std::ops::{Deref, DerefMut};

pub(crate) fn segmented(str: &str) -> impl Iterator<Item = &str> {
    str.split('/').filter(|seg| !seg.is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "serde", serde(bound(deserialize = "'de: 'static")))]
pub struct PathMatch(HashMap<&'static str, String>);

impl PathMatch {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    #[cfg(feature = "serde")]
    pub fn deserialize<'de, T: serde::de::Deserialize<'de>>(
        &'de self,
    ) -> Result<T, serde::de::value::Error> {
        let iter = self.0.iter().map(|(&k, v)| (k, v.as_str()));
        let deserializer = serde::de::value::MapDeserializer::new(iter);
        T::deserialize(deserializer)
    }
}

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

#[derive(Debug, Clone)]
pub struct Path(pub Vec<PathSegment>);

impl Path {
    pub fn matches(&self, path: &str) -> Option<PathMatch> {
        let mut params = PathMatch::default();
        let mut path_segs = segmented(path);

        for expected in self.iter() {
            match expected {
                PathSegment::Literal(expected_str) => {
                    if path_segs.next()? != *expected_str {
                        return None;
                    }
                }
                PathSegment::Dynamic(param) => {
                    let matched_val = param.matches(&mut path_segs)?;
                    params.insert(param.name, matched_val);
                }
            }
        }

        // If all expected segments were consumed, ensure no actual path segments remain
        if path_segs.next().is_none() {
            Some(params)
        } else {
            None
        }
    }

    pub fn replace<'a>(
        &self,
        params: &BTreeMap<&'static str, &'a str>,
    ) -> Option<PathAndQuery<'a>> {
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
        for segment in self.iter() {
            write!(f, "/{segment}")?;
        }
        Ok(())
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

impl PartialEq for PathToken {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, Self::Any) => true,
            #[cfg(feature = "regex")]
            (Self::Regex(a), Self::Regex(b)) => a.as_str() == b.as_str(),
            #[cfg(feature = "regex")]
            (Self::Any, Self::Regex(_)) | (Self::Regex(_), Self::Any) => false,
        }
    }
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

#[derive(Debug, Clone, PartialEq)]
pub struct PathParam {
    name: &'static str,
    token: PathToken,
    catch_all: bool,
}

impl PathParam {
    pub fn new(name: &'static str, token: PathToken) -> Self {
        Self {
            name,
            token,
            catch_all: false,
        }
    }

    pub fn catch_all(name: &'static str, token: PathToken) -> Self {
        Self {
            name,
            token,
            catch_all: true,
        }
    }

    pub fn is_catch_all(&self) -> bool {
        self.catch_all
    }

    pub(crate) fn name(&self) -> &'static str {
        self.name
    }

    pub(crate) fn token(&self) -> &PathToken {
        &self.token
    }

    pub fn matches<'a>(
        &self,
        path_segs: &mut impl Iterator<Item = &'a str>,
    ) -> Option<String> {
        if self.catch_all {
            let remaining = path_segs.collect::<Vec<_>>();
            if remaining.is_empty() {
                return None;
            }
            let joined = remaining.join("/");
            if !self.token.matches(&joined) {
                return None;
            }
            Some(joined)
        } else {
            let actual = path_segs.next()?;
            if !self.token.matches(actual) {
                return None;
            }
            Some(actual.to_owned())
        }
    }
}

impl fmt::Display for PathParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prefix = if self.catch_all { "*" } else { ":" };
        write!(f, "{}{}", prefix, self.name)
    }
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    Literal(&'static str),
    Dynamic(PathParam),
}

impl PathSegment {
    pub fn matches(&self, path: &str) -> bool {
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
        let mut first = true;
        for segment in &self.segments {
            if first {
                write!(f, "{segment}")?;
                first = false;
            } else {
                write!(f, "/{segment}")?;
            }
        }

        let mut first = true;
        for (k, v) in &self.query {
            if first {
                write!(f, "?{k}={v}")?;
                first = false;
            } else {
                write!(f, "&{k}={v}")?;
            }
        }

        Ok(())
    }
}
