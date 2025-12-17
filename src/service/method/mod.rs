mod body;
use self::body::body;
use crate::http::{Request, Result, bad_request, ok};
use itertools::Itertools;

pub async fn get(req: Request) -> Result {
    let body = {
        req.query::<Vec<(String, String)>>()
            .map_err(|_| bad_request())?
            .into_iter()
            .format_with("\n", |(key, value), f| {
                f(&format_args!("{key} = {value}"))
            })
            .to_string()
    };

    ok(body)
}

pub async fn post(req: Request) -> Result {
    body(req).await
}

pub async fn put(req: Request) -> Result {
    body(req).await
}

pub async fn patch(req: Request) -> Result {
    body(req).await
}

pub async fn delete(req: Request) -> Result {
    body(req).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::ContentType;
    use crate::test::*;
    use hyper::{Method, StatusCode};
    use std::collections::HashSet;
    use std::iter::FromIterator;

    #[tokio::test]
    async fn test_get() {
        let res = request().path("/?key=val").handle(get).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        assert_eq!(body, "key = val");
    }

    #[tokio::test]
    async fn test_multi_get() {
        let res = request()
            .path("/?key=val&other=something&key=another")
            .handle(get)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        let result: HashSet<&str> = HashSet::from_iter(body.split("\n"));
        let expected = HashSet::from_iter(vec![
            "key = val",
            "other = something",
            "key = another",
        ]);
        assert_eq!(expected, result)
    }

    #[tokio::test]
    async fn test_post() {
        let res = request()
            .method(Method::POST)
            .typed_header(ContentType::form_url_encoded())
            .body("key=val")
            .handle(post)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        assert_eq!(body, "key = val");
    }

    #[tokio::test]
    async fn test_multi_post() {
        let res = request()
            .method(Method::POST)
            .typed_header(ContentType::form_url_encoded())
            .body("key=val&other=something&key=another")
            .handle(post)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        let result: HashSet<&str> = HashSet::from_iter(body.split("\n"));
        let expected = HashSet::from_iter(vec![
            "key = val",
            "other = something",
            "key = another",
        ]);
        assert_eq!(expected, result)
    }

    #[tokio::test]
    async fn test_put() {
        let res = request()
            .method(Method::PUT)
            .typed_header(ContentType::form_url_encoded())
            .body("key=val")
            .handle(put)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        assert_eq!(body, "key = val");
    }

    #[tokio::test]
    async fn test_multi_put() {
        let res = request()
            .method(Method::PUT)
            .typed_header(ContentType::form_url_encoded())
            .body("key=val&other=something&key=another")
            .handle(put)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        let result: HashSet<&str> = HashSet::from_iter(body.split("\n"));
        let expected = HashSet::from_iter(vec![
            "key = val",
            "other = something",
            "key = another",
        ]);
        assert_eq!(expected, result)
    }

    #[tokio::test]
    async fn test_patch() {
        let res = request()
            .method(Method::PATCH)
            .typed_header(ContentType::form_url_encoded())
            .body("key=val")
            .handle(patch)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        assert_eq!(body, "key = val");
    }

    #[tokio::test]
    async fn test_multi_patch() {
        let res = request()
            .method(Method::PATCH)
            .typed_header(ContentType::form_url_encoded())
            .body("key=val&other=something&key=another")
            .handle(patch)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        let result: HashSet<&str> = HashSet::from_iter(body.split("\n"));
        let expected = HashSet::from_iter(vec![
            "key = val",
            "other = something",
            "key = another",
        ]);
        assert_eq!(expected, result)
    }

    #[tokio::test]
    async fn test_delete() {
        let res = request()
            .method(Method::DELETE)
            .handle(delete)
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}
