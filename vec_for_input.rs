use std::borrow::Cow;

use async_graphql::{registry, InputType, InputValueError, InputValueResult, Value};

pub struct VecForInput<T>(Vec<T>);

impl<T: InputType> InputType for VecForInput<T> {
    type RawValueType = Self;

    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("[{}]", T::qualified_type_name()))
    }

    fn qualified_type_name() -> String {
        format!("[{}]!", T::qualified_type_name())
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        Self::qualified_type_name()
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value.unwrap_or_default() {
            Value::List(values) => values
                .into_iter()
                .map(|value| InputType::parse(Some(value)))
                .collect::<Result<_, _>>()
                .map_err(InputValueError::propagate)
                .map(Self),
            value => Ok(Self(vec![
                InputType::parse(Some(value)).map_err(InputValueError::propagate)?
            ])),
        }
    }

    fn to_value(&self) -> Value {
        Value::List(self.0.iter().map(InputType::to_value).collect())
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    // async-graphqlのVecにこの行がないせいで、keyの生成がうまくいってない
    fn federation_fields() -> Option<String> {
        T::federation_fields()
    }
}
