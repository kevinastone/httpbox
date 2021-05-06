use crate::http::{bad_request, response, Request, Result, StatusCode};

pub async fn status_code(req: Request) -> Result {
    let status = req.param::<StatusCode>("code").ok_or_else(bad_request)?;

    response().status(status).into()
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test::*;
    use hyper::http::StatusCode;

    #[tokio::test]

    async fn test_status_code() {
        let res = request()
            .param("code", "429")
            .handle(status_code)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_bad_status_code() {
        let res = request()
            .param("code", "1000")
            .handle(status_code)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
