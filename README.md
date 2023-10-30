# tracing-dynamic

> :warning: Read this before using this library :warning:

This is a small library to allow you to create dynamic attributes on spans. 

Things to consider before using this:
* It will leak memory. This happens on each instantiation of SpanFactory. You'll want to create span factories sparingly and reuse them.
* I didn't test it in a real program.
* It'll be slower than the tracing macros, but you came here for flexibility right?

If after reading the above you want to try it, here's how you can use it:


```toml
tracing_dynamic = "0.2.0"
```

```rust
  use tracing_dynamic::{EventFactory, SpanFactory};
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
    let span = span_factory.create();
    let _guard = span.enter();
    span.record("dyn_attr_1", "dyn_attr_1");
    span.record("dyn_attr_2", "dyn_attr_2");
    span.record("dyn_attr_4", "dyn_attr_4"); // Not in the original metadata, it'll be ignored.
    
    // Create an event with attributes defined at runtime
    event_factory
      .create()
      .with("dyn_attr_1", &"dyn_attr_1")
      .with("dyn_attr_2", &"dyn_attr_2")
      .with("dyn_attr_4", &"dyn_attr_4") // Not in the original metadata, it'll be ignored.
      .finish();
  });
```


Output
```
  2023-10-30T08:44:52.178162Z  INFO event_target: dyn_attr_1: "dyn_attr_1", dyn_attr_2: "dyn_attr_2"
    in span_target::span_name with dyn_attr_1: "dyn_attr_1", dyn_attr_2: "dyn_attr_2"

```
