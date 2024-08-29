use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DateTimeRfc3339(
    #[serde(with = "datetime_serializer")] pub chrono::DateTime<chrono::Utc>,
);

impl DateTimeRfc3339 {
    pub fn new(t: chrono::DateTime<chrono::Utc>) -> Self {
        DateTimeRfc3339(t)
    }
    pub fn from_timestamp(ts: i64) -> anyhow::Result<Self> {
        Ok(Self(chrono::DateTime::from_timestamp(ts, 0).ok_or(
            anyhow::anyhow!("failed to convert date timestamp"),
        )?))
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

mod datetime_serializer {
    use chrono::{DateTime, Utc};
    use serde::{de::Error, Deserialize, Deserializer, Serialize as _, Serializer};

    pub fn serialize<S: Serializer>(
        time: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        time.to_rfc3339().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        let date_str: String = Deserialize::deserialize(deserializer)?;
        chrono::DateTime::parse_from_rfc3339(&date_str)
            .map(|x| x.to_utc())
            .or(Err(D::Error::custom(anyhow::anyhow!(
                "failed to parse datetime {date_str}"
            ))))
    }
}
