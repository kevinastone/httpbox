extern crate iron;

use self::iron::{Handler, IronResult, Request, Response};
use self::iron::method::Method;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Route {
    pub path: &'static str,
    pub method: Method,
    pub description: Option<&'static str>,
    pub example_params: HashMap<String, String>,
}

impl Route {
    pub fn new(path: &'static str) -> Self {
        Route {
            path: path,
            method: Method::Get,
            description: None,
            example_params: HashMap::new(),
        }
    }

    pub fn set_description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn set_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn add_example_param(mut self, name: &str, value: &str) -> Self {
        self.example_params
            .insert(name.to_owned(), value.to_owned());
        self
    }

    pub fn handle<H: Handler>(self, handler: H) -> RouteHandler {
        RouteHandler::new(self, Box::new(handler))
    }

    pub fn example_path(&self) -> Option<String> {
        if self.method != Method::Get {
            return None;
        }

        let mut path = self.path.to_owned();
        let mut query: Vec<String> = vec![];

        for (key, value) in self.example_params.iter() {
            let param = format!(":{}", key);
            if path.contains(&param[..]) {
                path = path.replace(&param[..], value);
            } else {
                query.push(format!("{}={}", key, value))
            }
        }

        if !query.is_empty() {
            path = format!("{}?{}", path, query.join("&"));
        }
        Some(path)
    }
}

pub struct RouteHandler {
    pub route: Route,
    pub handler: Box<Handler>,
}

impl RouteHandler {
    pub fn new(route: Route, handler: Box<Handler>) -> Self {
        RouteHandler {
            route: route,
            handler: handler,
        }
    }
}

impl Handler for RouteHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        self.handler.handle(req)
    }
}
