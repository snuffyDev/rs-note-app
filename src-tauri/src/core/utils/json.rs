use serde::Serialize;
use serde_json::Value;

pub fn to_json<T: Serialize>(data: &T) -> Result<Value, String> {
    match serde_json::to_value(data) {
        Ok(v) => Ok(v),
        Err(e) => throw!("Error serializing {}", e),
    }
}
