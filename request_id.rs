use axum::{
    async_trait, extract::FromRequestParts, http::request::Parts, response::Response, TypedHeader,
};

#[derive(Debug, Clone)]
pub struct RequestId(pub Option<String>);

static REQUEST_ID: axum::http::HeaderName = axum::http::HeaderName::from_static("x-request-id");

impl axum::headers::Header for RequestId {
    fn name() -> &'static axum::headers::HeaderName {
        &REQUEST_ID
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        let value = values.next().ok_or_else(axum::headers::Error::invalid)?;
        let value = value
            .to_str()
            .map_err(|_| axum::headers::Error::invalid())?;

        Ok(RequestId(Some(value.to_string())))
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, _values: &mut E) {
        todo!()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = TypedHeader::<RequestId>::from_request_parts(parts, state).await;
        if let Ok(token) = token {
            Ok(token.0)
        } else {
            Ok(RequestId(None))
        }
    }
}
