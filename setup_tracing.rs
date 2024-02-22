use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[cfg(feature = "with-graphql")]
use super::async_graphql_sentry_extension;

#[cfg(feature = "with-graphql")]
use async_graphql::SchemaBuilder;

pub struct SetupGuard {
    sentry_guard: Option<sentry::ClientInitGuard>,
    tracer: Option<opentelemetry::sdk::trace::Tracer>,
}

impl SetupGuard {
    #[cfg(feature = "with-graphql")]
    pub fn add_extension<Q, M, S>(
        &self,
        schema_builder: SchemaBuilder<Q, M, S>,
    ) -> SchemaBuilder<Q, M, S> {
        let schema_builder = if self.sentry_guard.is_some() {
            schema_builder.extension(async_graphql_sentry_extension::Sentry)
        } else {
            schema_builder
        };
        if let Some(tracer) = self.tracer.clone() {
            schema_builder.extension(async_graphql::extensions::OpenTelemetry::new(tracer))
        } else {
            schema_builder
        }
    }
}

impl Drop for SetupGuard {
    fn drop(&mut self) {
        self.sentry_guard.take();
        self.tracer.take();

        if let Some(client) = sentry::Hub::current().client() {
            client.close(Some(std::time::Duration::from_secs(2)));
        }

        opentelemetry::global::shutdown_tracer_provider();
    }
}

pub fn setup() -> anyhow::Result<SetupGuard> {
    let tracer = if let Ok(otel_exporter) = std::env::var("OTEL_EXPORTER") {
        let service_name = if let Ok(service_name) = std::env::var("OTEL_SERVICE_NAME") {
            service_name
        } else {
            std::env::var("HOSTNAME").unwrap_or("not-set".to_string())
        };

        opentelemetry::global::set_text_map_propagator(
            opentelemetry::sdk::propagation::TraceContextPropagator::new(),
        );
        let pipeline = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(otel_exporter),
            )
            .with_trace_config(
                opentelemetry::sdk::trace::config()
                    .with_sampler(opentelemetry::sdk::trace::Sampler::AlwaysOn)
                    .with_id_generator(opentelemetry::sdk::trace::RandomIdGenerator::default())
                    .with_resource(opentelemetry::sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                    ])),
            )
            .with_batch_config(
                opentelemetry::sdk::trace::BatchConfig::default()
                    .with_scheduled_delay(std::time::Duration::from_secs(10)),
            );

        #[cfg(debug_assertions)]
        let tracer = pipeline.install_simple()?;

        #[cfg(not(debug_assertions))]
        let tracer = pipeline.install_batch(opentelemetry::runtime::Tokio)?;

        Some(tracer)
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

        #[cfg(feature = "with-sentry")]
        let builder = builder.with(sentry_tracing::layer());

        // TODO: もっときれいにかけないものか
        if let Some(tracer) = tracer.clone() {
            builder
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
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
        tracer,
    })
}
