use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::parser::types::ExecutableDocument;
use async_graphql::{
    Response, ServerError, ServerResult, ValidationResult, Value,
    extensions::{
        Extension, ExtensionContext, ExtensionFactory, NextExecute, NextParseQuery, NextRequest,
        NextResolve, NextSubscribe, NextValidation, ResolveInfo,
    },
};
use async_graphql_value::{ConstValue, Variables};
use futures_util::{TryFutureExt, stream::BoxStream};
use opentelemetry::{
    Context as OpenTelemetryContext, Key, KeyValue,
    trace::{FutureExt, SpanKind, TraceContextExt, Tracer},
};

const KEY_SOURCE: Key = Key::from_static_str("graphql.source");
const KEY_VARIABLES: Key = Key::from_static_str("graphql.variables");
const KEY_PARENT_TYPE: Key = Key::from_static_str("graphql.parentType");
const KEY_RETURN_TYPE: Key = Key::from_static_str("graphql.returnType");
const KEY_ERROR: Key = Key::from_static_str("graphql.error");
const KEY_COMPLEXITY: Key = Key::from_static_str("graphql.complexity");
const KEY_DEPTH: Key = Key::from_static_str("graphql.depth");

/// OpenTelemetry extension
#[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry")))]
pub struct OpenTelemetry<T> {
    tracer: Arc<T>,
}

impl<T> OpenTelemetry<T> {
    /// Use `tracer` to create an OpenTelemetry extension.
    pub fn new(tracer: T) -> OpenTelemetry<T>
    where
        T: Tracer + Send + Sync + 'static,
        <T as Tracer>::Span: Sync + Send,
    {
        Self {
            tracer: Arc::new(tracer),
        }
    }
}

impl<T> ExtensionFactory for OpenTelemetry<T>
where
    T: Tracer + Send + Sync + 'static,
    <T as Tracer>::Span: Sync + Send,
{
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(OpenTelemetryExtension {
            tracer: self.tracer.clone(),
        })
    }
}

struct OpenTelemetryExtension<T> {
    tracer: Arc<T>,
}

#[async_trait::async_trait]
impl<T> Extension for OpenTelemetryExtension<T>
where
    T: Tracer + Send + Sync + 'static,
    <T as Tracer>::Span: Sync + Send,
{
    async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
        next.run(ctx)
            .with_context(OpenTelemetryContext::current_with_span(
                self.tracer
                    .span_builder("request")
                    .with_kind(SpanKind::Server)
                    .start(&*self.tracer),
            ))
            .await
    }

    fn subscribe<'s>(
        &self,
        ctx: &ExtensionContext<'_>,
        stream: BoxStream<'s, Response>,
        next: NextSubscribe<'_>,
    ) -> BoxStream<'s, Response> {
        Box::pin(
            next.run(ctx, stream)
                .with_context(OpenTelemetryContext::current_with_span(
                    self.tracer
                        .span_builder("subscribe")
                        .with_kind(SpanKind::Server)
                        .start(&*self.tracer),
                )),
        )
    }

    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        // secret情報を隠してくれなかったので除外
        let attributes = vec![
            // KeyValue::new(KEY_SOURCE, query.to_string()),
            KeyValue::new(KEY_VARIABLES, serialize_variables(variables)),
        ];
        let span = self
            .tracer
            .span_builder("parse")
            .with_kind(SpanKind::Server)
            .with_attributes(attributes)
            .start(&*self.tracer);

        async move {
            let res = next.run(ctx, query, variables).await;
            if let Ok(doc) = &res {
                OpenTelemetryContext::current()
                    .span()
                    .set_attribute(KeyValue::new(
                        KEY_SOURCE,
                        ctx.stringify_execute_doc(doc, variables),
                    ));
            }
            res
        }
        .with_context(OpenTelemetryContext::current_with_span(span))
        .await
    }

    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let span = self
            .tracer
            .span_builder("validation")
            .with_kind(SpanKind::Server)
            .start(&*self.tracer);
        next.run(ctx)
            .with_context(OpenTelemetryContext::current_with_span(span))
            .map_ok(|res| {
                let current_cx = OpenTelemetryContext::current();
                let span = current_cx.span();
                span.set_attribute(KeyValue::new(KEY_COMPLEXITY, res.complexity as i64));
                span.set_attribute(KeyValue::new(KEY_DEPTH, res.depth as i64));
                res
            })
            .await
    }

    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let span = self
            .tracer
            .span_builder("execute")
            .with_kind(SpanKind::Server)
            .start(&*self.tracer);
        next.run(ctx, operation_name)
            .with_context(OpenTelemetryContext::current_with_span(span))
            .await
    }

    async fn resolve(
        &self,
        ctx: &ExtensionContext<'_>,
        info: ResolveInfo<'_>,
        next: NextResolve<'_>,
    ) -> ServerResult<Option<Value>> {
        let span = if !info.is_for_introspection {
            let attributes = vec![
                KeyValue::new(KEY_PARENT_TYPE, info.parent_type.to_string()),
                KeyValue::new(KEY_RETURN_TYPE, info.return_type.to_string()),
            ];
            Some(
                self.tracer
                    .span_builder(info.path_node.to_string())
                    .with_kind(SpanKind::Server)
                    .with_attributes(attributes)
                    .start(&*self.tracer),
            )
        } else {
            None
        };

        let fut = next.run(ctx, info).inspect_err(|err| {
            let current_cx = OpenTelemetryContext::current();
            current_cx.span().add_event(
                "error".to_string(),
                vec![KeyValue::new(KEY_ERROR, err.to_string())],
            );
        });

        match span {
            Some(span) => {
                fut.with_context(OpenTelemetryContext::current_with_span(span))
                    .await
            }
            None => fut.await,
        }
    }
}

const CREDENTIAL_KEYS: [&str; 10] = [
    "token",
    "password",
    "secret",
    "key",
    "apiKey",
    "authToken",
    "accessToken",
    "refreshToken",
    "credential",
    "credentials",
];

fn is_credential(key: &str) -> bool {
    CREDENTIAL_KEYS.iter().any(|k| *k == key || key.contains(k))
}

fn serialize_variables(variabls: &Variables) -> String {
    let data = variabls
        .iter()
        .map(|(k, v)| {
            let value = if is_credential(k.as_str()) {
                serde_json::Value::String("<secret>".to_string())
            } else {
                serialize_const_value(v)
            };
            (k.clone(), value)
        })
        .collect::<HashMap<_, _>>();
    if let Ok(data) = serde_json::to_string(&data) {
        data
    } else {
        "failed to serialize variables".to_string()
    }
}

fn serialize_const_value(value: &ConstValue) -> serde_json::Value {
    match value {
        ConstValue::Binary(value) => {
            serde_json::Value::String(format!("<binary len={}>", value.len()))
        }
        ConstValue::Null => serde_json::Value::Null,
        ConstValue::Boolean(value) => serde_json::Value::Bool(*value),
        ConstValue::String(value) => serde_json::Value::String(value.clone()),
        ConstValue::Number(value) => serde_json::Value::Number(value.clone()),
        ConstValue::Enum(value) => serde_json::Value::String(value.to_string()),
        ConstValue::Object(value) => {
            let data = value
                .iter()
                .map(|(k, v)| {
                    let value = if is_credential(k.as_str()) {
                        serde_json::Value::String("<secret>".to_string())
                    } else {
                        serialize_const_value(v)
                    };
                    (k.as_str().to_string(), value)
                })
                .collect::<serde_json::Map<_, _>>();
            serde_json::Value::Object(data)
        }
        ConstValue::List(value) => {
            let data = value.iter().map(serialize_const_value).collect::<Vec<_>>();
            serde_json::Value::Array(data)
        }
    }
}
