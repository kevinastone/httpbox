use crate::{Path, PathMatch, PathParam, PathSegment};
use std::collections::HashMap;

/// Result of a successful router match containing a reference to the matched
/// handler/value and the extracted path parameters.
#[derive(Debug, PartialEq, Eq)]
pub struct Match<'a, T> {
    pub value: &'a T,
    pub params: PathMatch,
}

#[derive(Debug)]
struct Node<T> {
    literals: HashMap<&'static str, Node<T>>,
    dynamic: Vec<(PathParam, Node<T>)>,
    catch_all: Option<(PathParam, T)>,
    value: Option<T>,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Self {
            literals: HashMap::new(),
            dynamic: Vec::new(),
            catch_all: None,
            value: None,
        }
    }
}

/// A trie-based router for matching URI paths against route patterns.
#[derive(Debug)]
pub struct Router<T> {
    root: Node<T>,
}

impl<T> Default for Router<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Router<T> {
    /// Creates an empty router.
    pub fn new() -> Self {
        Self {
            root: Node::default(),
        }
    }

    /// Inserts a route pattern and its associated value into the router.
    pub fn insert(&mut self, path: Path, value: T) {
        let mut node = &mut self.root;

        for segment in path.0 {
            match segment {
                PathSegment::Literal(literal) => {
                    node = node.literals.entry(literal).or_default();
                }
                PathSegment::Dynamic(param) => {
                    if param.is_catch_all() {
                        node.catch_all = Some((param, value));
                        return;
                    }
                    if let Some(pos) = node.dynamic.iter().position(|(p, _)| {
                        p.name() == param.name()
                            && p.token().to_string()
                                == param.token().to_string()
                    }) {
                        node = &mut node.dynamic[pos].1;
                    } else {
                        node.dynamic.push((param, Node::default()));
                        node = &mut node.dynamic.last_mut().unwrap().1;
                    }
                }
            }
        }

        node.value = Some(value);
    }

    /// Matches a request URI path against the registered routes.
    pub fn at<'a>(&'a self, path: &str) -> Option<Match<'a, T>> {
        let trimmed = path.trim_matches('/');
        let (value, params) = self.match_node(&self.root, trimmed)?;
        Some(Match { value, params })
    }

    fn match_node<'a>(
        &'a self,
        node: &'a Node<T>,
        path: &str,
    ) -> Option<(&'a T, PathMatch)> {
        if path.is_empty() {
            return node.value.as_ref().map(|val| (val, PathMatch::new()));
        }

        let (first, rest) = path.split_once('/').unwrap_or((path, ""));

        // 1. Exact literal match (highest precedence)
        if let Some(child) = node.literals.get(first)
            && let Some(res) = self.match_node(child, rest)
        {
            return Some(res);
        }

        // 2. Dynamic single segment match
        for (param, child) in &node.dynamic {
            if param.token().matches(first)
                && let Some((val, mut params)) = self.match_node(child, rest)
            {
                params.insert(param.name(), first.to_owned());
                return Some((val, params));
            }
        }

        // 3. Catch-all tail match (lowest precedence)
        if let Some((param, val)) = &node.catch_all
            && param.token().matches(path)
        {
            let mut params = PathMatch::new();
            params.insert(param.name(), path.to_owned());
            return Some((val, params));
        }

        None
    }
}
