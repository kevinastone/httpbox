use crate::headers::XForwardedFor;
use crate::http::{bad_request, ok, Request, Result};

pub async fn ip(req: Request) -> Result {
    let ip = req
        .typed_header::<XForwardedFor>()
        .map(|header| header.ip_addr())
        .or_else(|| Some(req.client_addr()?.ip()))
        .ok_or_else(bad_request)?;

    ok(format!("{}", ip))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::headers::XForwardedFor;
    use crate::test::*;
    use hyper::http::StatusCode;

    #[tokio::test]
    async fn test_ip_x_forwarded_for() {
        let res = request()
            .typed_header(XForwardedFor::client("1.2.3.4".parse().unwrap()))
            .handle(ip)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        assert_eq!(body, "1.2.3.4");
    }

    #[tokio::test]
    async fn test_ip() {
        let res = request()
            .client_addr("127.0.0.1:1234".parse().unwrap())
            .handle(ip)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        assert_eq!(body, "127.0.0.1");
    }

    #[tokio::test]
    async fn test_ip_missing_addr() {
        let res = request().handle(ip).await.unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_ip_ignore_bad_header() {
        let res = request()
            .header("x-forwarded-for", "abc")
            .client_addr("127.0.0.1:1234".parse().unwrap())
            .handle(ip)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body = res.read_body_utf8().await.unwrap();
        assert_eq!(body, "127.0.0.1");
    }
}
