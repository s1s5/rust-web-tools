use async_graphql::{
    Response, ServerError, ServerResult, ValidationResult, Value, Variables,
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextRequest,
        NextResolve, NextSubscribe, NextValidation, ResolveInfo,
    },
    parser::types::ExecutableDocument,
};
use futures_util::stream::BoxStream;

use std::sync::Arc;

#[derive(Clone)]
pub struct AsyncGraphqlResolveFilterExtension {
    inner: Arc<dyn Extension + Send>,
    exclude_filter: Arc<dyn Fn(&ResolveInfo<'_>) -> bool + Send + Sync>,
}

impl AsyncGraphqlResolveFilterExtension {
    pub fn new(
        extension: Arc<dyn Extension + Send>,
        exclude_filter: Arc<dyn Fn(&ResolveInfo<'_>) -> bool + Send + Sync>,
    ) -> Self {
        Self {
            inner: extension,
            exclude_filter,
        }
    }
}

impl ExtensionFactory for AsyncGraphqlResolveFilterExtension {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(self.clone())
    }
}

#[async_trait::async_trait]
impl Extension for AsyncGraphqlResolveFilterExtension {
    // 既存のOpenTelemetry実装の他のメソッド（解析開始など）はinnerに委譲...
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        self.inner.request(ctx, next).await
    }

    fn subscribe<'s>(
        &self,
        ctx: &ExtensionContext<'_>,
        stream: BoxStream<'s, Response>,
        next: NextSubscribe<'_>,
    ) -> BoxStream<'s, Response> {
        self.inner.subscribe(ctx, stream, next)
    }

    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        self.inner.parse_query(ctx, query, variables, next).await
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        self.inner.validation(ctx, next).await
    }
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        self.inner.execute(ctx, operation_name, next).await
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        if self.exclude_filter.as_ref()(&info) {
            return next.run(ctx, info).await;
        }

        self.inner.resolve(ctx, info, next).await
    }
}
