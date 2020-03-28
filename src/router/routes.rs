use crate::path::{MatchedPath, Path};
use hyper::{Body, Method, Request as HTTPRequest};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct RouteBuilder {
    path: Path,
    method: Method,
    description: Option<&'static str>,
    example_params: BTreeMap<&'static str, &'static str>,
}

impl RouteBuilder {
    pub fn new<P: Into<Path>>(path: P) -> Self {
        RouteBuilder {
            path: path.into(),
            method: Method::GET,
            description: None,
            example_params: BTreeMap::new(),
        }
    }

    pub fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn add_example_param(
        mut self,
        name: &'static str,
        value: &'static str,
    ) -> Self {
        self.example_params.insert(name, value);
        self
    }

    fn example_path(self: &Self) -> Option<String> {
        if self.method != Method::GET {
            return None;
        }

        Some(self.path.replace(&self.example_params)?.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct Route {
    path: Path,
    method: Method,
    description: Option<&'static str>,
    example_path: Option<String>,
}

impl Route {
    pub fn path(&self) -> Path {
        self.path.clone()
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }

    pub fn description(&self) -> Option<&'static str> {
        self.description
    }

    pub fn example_path(&self) -> Option<&str> {
        self.example_path.as_ref().map(String::as_ref)
    }

    pub fn matches(&self, req: &HTTPRequest<Body>) -> Option<MatchedPath> {
        if self.method() != req.method() {
            return None;
        }

        self.path.matches(req.uri().path())
    }
}

impl From<RouteBuilder> for Route {
    fn from(route: RouteBuilder) -> Self {
        let example_path = route.example_path();
        Route {
            path: route.path,
            method: route.method,
            description: route.description,
            example_path,
        }
    }
}

pub fn route<P: Into<Path>>(path: P) -> RouteBuilder {
    RouteBuilder::new(path)
}
