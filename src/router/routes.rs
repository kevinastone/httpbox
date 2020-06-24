use hyper::{Body, Method, Request as HTTPRequest};
use typed_path::Path;

#[derive(Debug)]
pub struct RouteBuilder<T> {
    path: Path<T>,
    method: Method,
    description: Option<&'static str>,
    // example_params: T,
}

impl<T> RouteBuilder<T> {
    pub fn new<P: Into<Path<T>>>(path: P) -> Self {
        RouteBuilder {
            path: path.into(),
            method: Method::GET,
            description: None,
            // example_params: T::default(),
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
        // self.example_params.insert(name, value);
        self
    }

    fn example_path(self: &Self) -> Option<String> {
        if self.method != Method::GET {
            return None;
        }

        None
        // Some(self.path.replace(&self.example_params)?.to_string())
    }
}

#[derive(Debug)]
pub struct Route<T> {
    path: Path<T>,
    method: Method,
    description: Option<&'static str>,
    example_path: Option<String>,
}

impl<T> Route<T> {
    pub fn path(&self) -> &Path<T> {
        &self.path
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn description(&self) -> Option<&'static str> {
        self.description
    }

    pub fn example_path(&self) -> Option<&str> {
        self.example_path.as_ref().map(String::as_ref)
    }
}

impl<T: serde::de::DeserializeOwned> Route<T> {
    pub fn matches(&self, req: &HTTPRequest<Body>) -> Option<T> {
        if self.method() != req.method() {
            return None;
        }

        self.path.parse(req.uri().path())
    }
}

impl<T> From<RouteBuilder<T>> for Route<T> {
    fn from(route: RouteBuilder<T>) -> Self {
        let example_path = route.example_path();
        Route {
            path: route.path,
            method: route.method,
            description: route.description,
            example_path,
        }
    }
}

pub fn route<T, P: Into<Path<T>>>(path: P) -> RouteBuilder<T> {
    RouteBuilder::new(path)
}
