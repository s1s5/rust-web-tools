use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
pub struct DateTimeRfc3339(chrono::DateTime<chrono::FixedOffset>);

#[allow(dead_code)]
impl DateTimeRfc3339 {
    pub fn new(t: chrono::DateTime<chrono::FixedOffset>) -> DateTimeRfc3339 {
        DateTimeRfc3339(t)
    }
}

#[Scalar]
impl ScalarType for DateTimeRfc3339 {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            chrono::DateTime::parse_from_rfc3339(value)
                .map(DateTimeRfc3339)
                .or(Err(InputValueError::custom("invalid rfc3339 format")))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339())
    }
}
