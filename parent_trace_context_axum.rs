use std::collections::HashMap;

use axum::{extract::FromRequestParts, http::request::Parts, response::Response};

pub use super::parent_trace_context::{ParentTraceContext, TRACEPARENT_HEADER, TRACESTATE_HEADER};

#[derive(Debug, Clone)]
pub struct ParentTraceContextAxum {
    headers: HashMap<&'static str, Option<String>>,
}

impl ParentTraceContextAxum {
    async fn from_request_parts_impl<S: Send + Sync>(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Response> {
        Ok(ParentTraceContextAxum {
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

impl<S> FromRequestParts<S> for ParentTraceContextAxum
where
    S: Send + Sync,
{
    type Rejection = Response;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        ParentTraceContextAxum::from_request_parts_impl(parts, state).into_future()
    }
}

impl From<ParentTraceContextAxum> for ParentTraceContext {
    fn from(value: ParentTraceContextAxum) -> Self {
        ParentTraceContext::new(
            value.headers.get(TRACEPARENT_HEADER).cloned().unwrap_or_default(),
            value.headers.get(TRACESTATE_HEADER).cloned().unwrap_or_default(),
        )
    }
}
