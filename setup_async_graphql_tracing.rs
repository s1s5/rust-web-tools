use super::async_graphql_sentry_extension;
use opentelemetry::trace::TracerProvider;

use async_graphql::SchemaBuilder;

pub fn add_extension<Q, M, S>(
    setup_guard: &super::setup_tracing::SetupGuard,
    schema_builder: SchemaBuilder<Q, M, S>,
) -> SchemaBuilder<Q, M, S> {
    let schema_builder = if setup_guard.sentry_guard.is_some() {
        schema_builder.extension(async_graphql_sentry_extension::Sentry)
    } else {
        schema_builder
    };
    if let Some(provider) = setup_guard.provider.as_ref() {
        schema_builder.extension(
            super::async_graphql_extensions_opentelemetry::OpenTelemetry::new(
                provider.tracer("graphql"),
            ),
        )
    } else {
        schema_builder
    }
}
