use std::collections::HashMap;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts, response::Response};
const TRACEPARENT_HEADER: &str = "traceparent";
const TRACESTATE_HEADER: &str = "tracestate";

#[derive(Debug, Clone)]
pub struct Trace {
    headers: HashMap<&'static str, Option<String>>,
}

#[async_trait]
impl<S> FromRequestParts<S> for Trace
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Trace {
            headers: HashMap::from_iter(
                [
                    (
                        TRACEPARENT_HEADER,
                        parts
                            .headers
                            .get(TRACEPARENT_HEADER)
                            .map(|x| x.to_str().unwrap().to_string()),
                    ),
                    (
                        TRACESTATE_HEADER,
                        parts
                            .headers
                            .get(TRACESTATE_HEADER)
                            .map(|x| x.to_str().unwrap().to_string()),
                    ),
                ]
                .into_iter()
                .filter(|x| x.1.is_some()),
            ),
        })
    }
}

impl opentelemetry::propagation::Extractor for Trace {
    fn get(&self, key: &str) -> Option<&str> {
        self.headers
            .get(key)
            .unwrap_or(&None)
            .as_ref()
            .map(|x| x as &str)
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|x| *x).collect::<Vec<_>>()
    }
}
