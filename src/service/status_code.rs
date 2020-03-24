use crate::http::{
    bad_request, Body, HTTPResponse, Request, Result, StatusCode,
};

pub async fn status_code(req: Request) -> Result {
    let code = req.param::<u16>("code").ok_or_else(bad_request)?;

    HTTPResponse::builder()
        .status(StatusCode::from_u16(code).map_err(|_| bad_request())?)
        .body(Body::empty())
        .map_err(Into::into)
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::test::request;
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
            .param("code", "999")
            .handle(status_code)
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
