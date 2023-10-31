use tracing_dynamic::{EventFactory, SpanFactory};

fn main() {
    // Some dynamic attributes, these could be read from a file or a database.
    let attrs = vec!["dyn_attr_1", "dyn_attr_2"];

    // Factories leak, create these once and reuse them.
    let span_factory = SpanFactory::new(
        "span_name",
        "span_target",
        tracing::Level::INFO,
        None,
        None,
        None,
        &attrs,
    );
    let event_factory = EventFactory::new(
        "event_name",
        "event_target",
        tracing::Level::INFO,
        None,
        None,
        None,
        &attrs,
    );

    let subscriber = tracing_subscriber::fmt().pretty().finish();
    tracing::subscriber::with_default(subscriber, || {
        // Create a span with attributes defined at runtime
        let span = span_factory
            .create()
            .with("dyn_attr_1", &"dyn_attr_1")
            .build();
        let _guard = span.enter();
        span.record("dyn_attr_2", "dyn_attr_2");
        span.record("dyn_attr_4", "dyn_attr_4"); // Not in the original metadata, it'll be ignored.

        // Create an event with attributes defined at runtime
        event_factory
            .create()
            .with("dyn_attr_1", &"dyn_attr_1")
            .with("dyn_attr_2", &"dyn_attr_2")
            .with("dyn_attr_4", &"dyn_attr_4") // Not in the original metadata, it'll be ignored.
            .build();
    });
}
