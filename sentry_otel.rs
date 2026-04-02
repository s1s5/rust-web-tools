use opentelemetry::trace::TraceContextExt;

pub fn set_otel_sentry_scope() {
    sentry::configure_scope(|scope| {
        let otel_context = opentelemetry::Context::current();
        let span = otel_context.span();

        let mut map = std::collections::BTreeMap::new();
        let trace_id = format!("{:0>32}", span.span_context().trace_id());
        map.insert(
            String::from("trace_id"),
            serde_json::Value::String(trace_id.clone()),
        );
        map.insert(
            String::from("span_id"),
            serde_json::Value::String(format!("{:0>32}", span.span_context().span_id())),
        );

        scope.set_context("opentelemetry", sentry::protocol::Context::Other(map));
        scope.set_tag("otel.trace_id", trace_id);
    });
}
