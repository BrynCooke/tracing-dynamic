use tracing_dynamic::SpanFactory;

fn main() {
    let attrs = vec!["dyn_attr_1", "dyn_attr_2"];
    let span_factory = SpanFactory::new("span_name", "span_target", tracing::Level::INFO, None, None, None, &attrs);

    let subscriber = tracing_subscriber::fmt().pretty().finish();
    tracing::subscriber::with_default(subscriber, || {
        let span = span_factory.create();
        let _guard = span.enter();
        span.record("dyn_attr_1", "dyn_attr_1");
        span.record("dyn_attr_2", "dyn_attr_2");
        span.record("dyn_attr_4", "dyn_attr_4"); // Not in the original metadata, it'll be ignored.
        tracing::error!("Hello world");
    });
}
