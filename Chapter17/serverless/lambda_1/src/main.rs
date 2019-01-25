use lambda_runtime::{error::HandlerError, lambda, Context};
use serde_json::Value;

fn main() {
    lambda!(handler)
}

fn handler(
    event: Value,
    _: Context,
) -> Result<Value, HandlerError> {
    Ok(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn handler_handles() {
        let event = json!({
            "answer": 42
        });
        assert_eq!(
            handler(event.clone(), Context::default()).expect("expected Ok(_) value"),
            event
        )
    }
}
