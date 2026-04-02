pub const TRACEPARENT_HEADER: &str = "traceparent";
pub const TRACESTATE_HEADER: &str = "tracestate";

#[derive(Debug, Clone)]
pub struct ParentTraceContext {
    parent: Option<String>,
    state: Option<String>,
}

impl ParentTraceContext {
    pub fn new(parent: Option<String>, state: Option<String>) -> Self {
        Self { parent, state }
    }

    pub fn get(&self) -> opentelemetry::Context {
        opentelemetry::global::get_text_map_propagator(|prop| prop.extract(self))
    }
}

impl opentelemetry::propagation::Extractor for ParentTraceContext {
    fn get(&self, key: &str) -> Option<&str> {
        if key == TRACEPARENT_HEADER {
            self.parent.as_deref()
        } else if key == TRACESTATE_HEADER {
            self.state.as_deref()
        } else {
            None
        }
    }

    fn keys(&self) -> Vec<&str> {
        self.parent
            .as_deref()
            .into_iter()
            .chain(self.state.as_deref())
            .collect()
    }
}
