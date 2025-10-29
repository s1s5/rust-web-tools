use std::{fmt::Write, sync::Arc};

use async_graphql::{
    PathSegment, Response,
    extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute},
};

pub struct Sentry;

impl ExtensionFactory for Sentry {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(SentryExtension)
    }
}

struct SentryExtension;

#[async_graphql::async_trait::async_trait]
impl Extension for SentryExtension {
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        super::sentry_otel::set_otel_sentry_scope();

        let resp = next.run(ctx, operation_name).await;

        if resp.is_err() {
            let mut error_message = None;
            let mut paths = vec![];
            let mut errors = vec![];
            for err in &resp.errors {
                if !err.path.is_empty() {
                    let mut path = String::new();
                    for (idx, s) in err.path.iter().enumerate() {
                        if idx > 0 {
                            path.push('.');
                        }
                        match s {
                            PathSegment::Index(idx) => {
                                let _ = write!(&mut path, "{}", idx);
                            }
                            PathSegment::Field(name) => {
                                let _ = write!(&mut path, "{}", name);
                            }
                        }
                    }
                    paths.push(path);
                    errors.push(err.message.clone());
                }
                if error_message.is_none() {
                    error_message = Some(err.message.clone());
                }
            }
            sentry::configure_scope(|scope| {
                let mut map = std::collections::BTreeMap::new();
                map.insert(String::from("path"), serde_json::json!(paths));
                map.insert(String::from("errors"), serde_json::json!(errors));

                scope.set_context("graphql", sentry::protocol::Context::Other(map));
            });

            let error_message = error_message.unwrap_or("GraphqlError".to_string());
            // sentry::capture_message(&error_message, sentry::Level::Error);
            tracing::error!("{}", error_message);
        }
        resp
    }
}
