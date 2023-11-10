use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn setup_tracing_and_opentelemetry(
    service_name: &str,
    simple: bool,
) -> anyhow::Result<Option<opentelemetry::sdk::trace::Tracer>> {
    let tracer = if let Ok(otel_exporter) = std::env::var("OTEL_EXPORTER") {
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
            );
        let tracer = if simple {
            pipeline.install_simple()
        } else {
            pipeline.install_batch(opentelemetry::runtime::Tokio)
        }?;
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

        // TODO: もっときれいにかけないものか
        if let Some(tracer) = tracer.clone() {
            builder
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .try_init()?;
        } else {
            builder.try_init()?;
        }
    }
    Ok(tracer)
}
