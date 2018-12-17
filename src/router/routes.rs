use hyper::Method;
use std::collections::HashMap;

#[derive(Debug)]
pub struct RouteBuiler<'a> {
    path: &'a str,
    method: Method,
    description: Option<&'a str>,
    example_params: HashMap<&'a str, &'a str>,
}

impl<'a> RouteBuiler<'a> {
    pub fn new(path: &'a str) -> Self {
        RouteBuiler {
            path,
            method: Method::GET,
            description: None,
            example_params: HashMap::new(),
        }
    }

    pub fn set_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn set_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn add_example_param(mut self, name: &'a str, value: &'a str) -> Self {
        self.example_params.insert(name, value);
        self
    }

    fn example_path(self: &Self) -> Option<String> {
        if self.method != Method::GET {
            return None;
        }

        let mut path = self.path.to_owned();
        let mut query: Vec<_> = vec![];

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

#[derive(Debug, Clone)]
pub struct Route<'a> {
    path: &'a str,
    method: Method,
    description: Option<&'a str>,
    example_path: Option<String>,
}

impl<'a> Route<'a> {
    pub fn path(&self) -> &'a str {
        self.path
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description
    }

    pub fn example_path(&self) -> Option<&str> {
        self.example_path.as_ref().map(String::as_ref)
    }
}

impl<'a> From<RouteBuiler<'a>> for Route<'a> {
    fn from(route: RouteBuiler<'a>) -> Self {
        let example_path = route.example_path();
        Route {
            path: route.path,
            method: route.method,
            description: route.description,
            example_path,
        }
    }
}
