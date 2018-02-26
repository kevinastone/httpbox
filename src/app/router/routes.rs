extern crate hyper;

use hyper::Method;
use std::collections::HashMap;
use std::convert::Into;

#[derive(Debug)]
pub struct Route<'a> {
    pub path: &'a str,
    pub method: Method,
    pub description: Option<&'a str>,
    pub example_params: HashMap<String, String>,
}

impl<'a> Route<'a> {
    pub fn new(path: &'a str) -> Self {
        Route {
            path: path,
            method: Method::Get,
            description: None,
            example_params: HashMap::new(),
        }
    }

    pub fn set_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    #[allow(dead_code)]
    pub fn set_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn add_example_param(mut self, name: &str, value: &str) -> Self {
        self.example_params
            .insert(name.to_owned(), value.to_owned());
        self
    }
}

#[derive(Debug, Clone)]
pub struct FrozenRoute<'a> {
    path: &'a str,
    method: Method,
    description: Option<&'a str>,
    example_params: HashMap<String, String>,
}

impl<'a> FrozenRoute<'a> {
    pub fn path(&self) -> &'a str {
        return self.path;
    }

    pub fn method(&self) -> Method {
        return self.method.clone();
    }

    pub fn description(&self) -> Option<&'a str> {
        return self.description;
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

impl<'a> Into<FrozenRoute<'a>> for Route<'a> {
    fn into(self) -> FrozenRoute<'a> {
        FrozenRoute {
            path: self.path,
            method: self.method,
            description: self.description,
            example_params: self.example_params.clone(),
        }
    }
}
