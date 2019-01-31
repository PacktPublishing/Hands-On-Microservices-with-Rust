use lambda_http::{lambda, IntoResponse, Request};
use lambda_runtime::{error::HandlerError, Context};
use serde_json::json;

fn main() {
    lambda!(handler)
}

fn handler(
    _: Request,
    _: Context,
) -> Result<impl IntoResponse, HandlerError> {
    Ok(json!({
    "message": "Go Serverless v1.0! Your function executed successfully!"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_handles() {
        let request = Request::default();
        let expected = json!({
        "message": "Go Serverless v1.0! Your function executed successfully!"
        })
        .into_response();
        let response = handler(request, Context::default())
            .expect("expected Ok(_) value")
            .into_response();
        assert_eq!(response.body(), expected.body())
    }
}