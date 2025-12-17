use crate::http::{Request, Result, ok};

pub async fn healthz(_req: Request) -> Result {
    ok("OK")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::*;
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_healthz() {
        let res = request().handle(healthz).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        assert_eq!(body, "OK");
    }
}
