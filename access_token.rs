use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization},
    http::request::Parts,
    response::Response,
    TypedHeader,
};

#[derive(Debug, Clone)]
pub struct AccessToken(pub Option<String>);

#[async_trait]
impl<S> FromRequestParts<S> for AccessToken
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await;
        if let Ok(token) = token {
            Ok(AccessToken(Some(token.0 .0.token().to_string())))
        } else {
            Ok(AccessToken(None))
        }
    }
}
