use crate::headers::UserAgent;
use crate::http::{bad_request, ok, Request, Result};

pub async fn user_agent(req: Request) -> Result {
    let agent = req
        .typed_header::<UserAgent>()
        .ok_or_else(bad_request)
        .map(|ua| ua.to_string())?;
    ok(agent)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::{request, TestResponseExt};
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_user_agent_missing() {
        let res = request().handle(user_agent).await.unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_user_agent_custom() {
        let res = request()
            .header("user-agent", "HTTPBoxBot/1.0")
            .handle(user_agent)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_utf8_body().await.unwrap();
        assert_eq!(body, "HTTPBoxBot/1.0");
    }
}
