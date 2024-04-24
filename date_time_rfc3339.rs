use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use chrono::serde::ts_seconds;
use serde::Serialize;

#[derive(Serialize)]
pub struct DateTimeRfc3339(#[serde(with = "ts_seconds")] chrono::DateTime<chrono::Utc>);

#[allow(dead_code)]
impl DateTimeRfc3339 {
    pub fn new(t: chrono::DateTime<chrono::Utc>) -> DateTimeRfc3339 {
        DateTimeRfc3339(t)
    }
}

#[Scalar]
impl ScalarType for DateTimeRfc3339 {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            chrono::DateTime::parse_from_rfc3339(value)
                .map(|x| DateTimeRfc3339(x.into()))
                .or(Err(InputValueError::custom("invalid rfc3339 format")))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339())
    }
}
