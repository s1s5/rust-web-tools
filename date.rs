use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Date(#[serde(with = "date_serializer")] pub chrono::NaiveDate);

impl Date {
    pub fn new(t: chrono::NaiveDate) -> Date {
        Date(t)
    }
}

#[Scalar]
impl ScalarType for Date {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(value) = &value {
            chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map(Date)
                .map_err(|e| {
                    InputValueError::custom(format!("invalid date format yyyy-mm-dd. error={e:?}"))
                })
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

mod date_serializer {
    use chrono::NaiveDate;
    use serde::{de::Error, Deserialize, Deserializer, Serialize as _, Serializer};

    pub fn serialize<S: Serializer>(time: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error> {
        time.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDate, D::Error> {
        let date_str: String = Deserialize::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() -> anyhow::Result<()> {
        let d = Date(chrono::NaiveDate::from_ymd_opt(2000, 1, 2).unwrap());
        let value = d.to_value();
        match &value {
            Value::String(s) => {
                assert_eq!(s, "2000-01-02");
            }
            _ => {
                panic!();
            }
        }
        let parsed = Date::parse(value).unwrap();
        assert_eq!(d, parsed);
        Ok(())
    }
}
