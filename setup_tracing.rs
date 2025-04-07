use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use tracing::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use super::async_graphql_sentry_extension;

use async_graphql::SchemaBuilder;

pub struct SetupGuard {
    sentry_guard: Option<sentry::ClientInitGuard>,
    provider: Option<opentelemetry_sdk::trace::TracerProvider>,
}

impl SetupGuard {
    pub fn add_extension<Q, M, S>(
        &self,
        schema_builder: SchemaBuilder<Q, M, S>,
    ) -> SchemaBuilder<Q, M, S> {
        let schema_builder = if self.sentry_guard.is_some() {
            schema_builder.extension(async_graphql_sentry_extension::Sentry)
        } else {
            schema_builder
        };
        if let Some(provider) = self.provider.as_ref() {
            schema_builder.extension(async_graphql::extensions::OpenTelemetry::new(
                provider.tracer("graphql"),
            ))
        } else {
            schema_builder
        }
    }
}

impl Drop for SetupGuard {
    fn drop(&mut self) {
        self.sentry_guard.take();

        if let Some(client) = sentry::Hub::current().client() {
            client.close(Some(std::time::Duration::from_secs(2)));
        }

        if let Some(provider) = self.provider.take() {
            provider.force_flush();
            if let Err(err) = provider.shutdown() {
                warn!("otel shtdown error: {err:?}");
            }
        }

        opentelemetry::global::shutdown_tracer_provider();
    }
}

pub fn setup() -> anyhow::Result<SetupGuard> {
    let provider = if let Ok(otel_exporter) = std::env::var("OTEL_EXPORTER") {
        let service_name = if let Ok(service_name) = std::env::var("OTEL_SERVICE_NAME") {
            service_name
        } else {
            std::env::var("HOSTNAME").unwrap_or("not-set".to_string())
        };

        opentelemetry::global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );

        let provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_batch_exporter(
                opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(otel_exporter)
                    .with_timeout(std::time::Duration::from_secs(5))
                    .build()?,
                    opentelemetry_sdk::runtime::Tokio
            )
            .with_config(
                opentelemetry_sdk::trace::Config::default()
                    .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
                    .with_id_generator(opentelemetry_sdk::trace::RandomIdGenerator::default())
                    .with_max_events_per_span(32)
                    .with_max_attributes_per_event(16)
                    .with_resource(opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                    ])),
            )
            // .with_batch_config(
            //     opentelemetry_sdk::trace::BatchConfigBuilder::default()
            //         .with_scheduled_delay(std::time::Duration::from_secs(10))
            //         .build(),
            // )
            .build();

        // install_simpleだと動作しない・・・？
        // #[cfg(debug_assertions)]
        // let provider = pipeline.install_simple()?;

        // #[cfg(not(debug_assertions))]
        // let provider = pipeline.install_batch(opentelemetry_sdk::runtime::Tokio)?;

        opentelemetry::global::set_tracer_provider(provider.clone());

        Some(provider)
    } else {
        None
    };

    {
        let builder = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::Layer::new()
                    .with_ansi(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_level(true),
            )
            .with(tracing_subscriber::EnvFilter::from_default_env());

        let builder = builder.with(sentry_tracing::layer());

        // TODO: もっときれいにかけないものか
        if let Some(provider) = provider.as_ref() {
            builder
                .with(
                    tracing_opentelemetry::layer()
                        .with_tracer(provider.tracer("tracing-otel-subscriber")),
                )
                .try_init()?;
        } else {
            builder.try_init()?;
        }
    }
    Ok(SetupGuard {
        sentry_guard: if let Ok(sentry_dsn) = std::env::var("SENTRY_DSN") {
            Some(sentry::init(sentry_dsn))
        } else {
            None
        },
        provider,
    })
}
