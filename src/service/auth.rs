use crate::headers::Authorization;
use crate::headers::WWWAuthenticate;
use crate::headers::authorization::{Basic, Bearer};
use crate::http::{Error, Request, Result, StatusCode, ok, response};

use serde_derive::Deserialize;

pub(crate) const REALM: &str = "User Visible Realm";

fn unauthorized_authenticate() -> Error {
    response()
        .status(StatusCode::UNAUTHORIZED)
        .typed_header(WWWAuthenticate::basic_realm(REALM))
        .into()
}

fn unauthorized() -> Error {
    response().status(StatusCode::UNAUTHORIZED).into()
}

#[derive(Deserialize)]
pub struct BasicAuthParams {
    user: String,
    passwd: String,
}

pub async fn basic(req: Request) -> Result {
    let BasicAuthParams { user, passwd } = req
        .params::<BasicAuthParams>()
        .ok_or_else(unauthorized_authenticate)?;

    let headers = req
        .typed_header::<Authorization<Basic>>()
        .map(|header| header.0)
        .filter(|basic| basic.username() == user && basic.password() == passwd);

    let _ = headers.ok_or_else(unauthorized_authenticate)?;
    ok("Authenticated")
}

pub async fn bearer(req: Request) -> Result {
    let token = req.param::<String>("token").ok_or_else(unauthorized)?;

    let headers = req
        .typed_header::<Authorization<Bearer>>()
        .map(|header| header.0)
        .filter(|bearer| bearer.token() == token);

    let _ = headers.ok_or_else(unauthorized)?;
    ok("Authenticated")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::HeaderMapExt;
    use crate::test::*;
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_basic_no_authorization() {
        let res = request()
            .param("user", "my-username")
            .param("passwd", "my-password")
            .handle(basic)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            res.headers().typed_get::<WWWAuthenticate>().unwrap(),
            WWWAuthenticate::basic_realm(REALM),
        )
    }

    #[tokio::test]
    async fn test_basic_authorized() {
        let auth = Authorization::basic("my-username", "my-password");

        let res = request()
            .param("user", "my-username")
            .param("passwd", "my-password")
            .typed_header(auth)
            .handle(basic)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_basic_unauthorized() {
        let auth = Authorization::basic("my-username", "not-my-password");

        let res = request()
            .param("user", "my-username")
            .param("passwd", "my-password")
            .typed_header(auth)
            .handle(basic)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            res.headers().typed_get::<WWWAuthenticate>().unwrap(),
            WWWAuthenticate::basic_realm(REALM),
        )
    }

    #[tokio::test]
    async fn test_basic_missing_params() {
        let res = request().handle(basic).await.unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            res.headers().typed_get::<WWWAuthenticate>().unwrap(),
            WWWAuthenticate::basic_realm(REALM),
        );
    }

    #[tokio::test]
    async fn test_bearer_no_authorization() {
        let res = request()
            .param("token", "my-token")
            .handle(bearer)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_bearer_authorized() {
        let auth = Authorization::bearer("my-token").unwrap();

        let res = request()
            .param("token", "my-token")
            .typed_header(auth)
            .handle(bearer)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_bearer_unauthorized() {
        let auth = Authorization::bearer("not-my-token").unwrap();

        let res = request()
            .param("token", "my-token")
            .typed_header(auth)
            .handle(bearer)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_basic_non_basic_auth_header() {
        let auth = Authorization::bearer("some-token").unwrap();

        let res = request()
            .param("user", "my-username")
            .param("passwd", "my-password")
            .typed_header(auth)
            .handle(basic)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            res.headers().typed_get::<WWWAuthenticate>().unwrap(),
            WWWAuthenticate::basic_realm(REALM),
        )
    }

    #[tokio::test]
    async fn test_basic_invalid_basic_auth_header() {
        // This header is technically valid as a HeaderValue, but not as a Basic auth value
        let res = request()
            .param("user", "my-username")
            .param("passwd", "my-password")
            .header("Authorization", "Basic invalid-format")
            .handle(basic)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            res.headers().typed_get::<WWWAuthenticate>().unwrap(),
            WWWAuthenticate::basic_realm(REALM),
        )
    }

    #[tokio::test]
    async fn test_bearer_non_bearer_auth_header() {
        let auth = Authorization::basic("user", "pass");

        let res = request()
            .param("token", "my-token")
            .typed_header(auth)
            .handle(bearer)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_bearer_invalid_bearer_auth_header() {
        // This header is technically valid as a HeaderValue, but not as a Bearer auth value
        let res = request()
            .param("token", "my-token")
            .header("Authorization", "Bearer invalid-format")
            .handle(bearer)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_bearer_missing_param() {
        let auth = Authorization::bearer("my-token").unwrap();

        let res = request().typed_header(auth).handle(bearer).await.unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }
}
