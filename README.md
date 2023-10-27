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
  let attrs = vec!["dyn_attr_1", "dyn_attr_2"];
  let span_factory = SpanFactory::new("span_name", "span_target", tracing::Level::INFO, None, None, None, &attrs);

  let subscriber = tracing_subscriber::fmt().pretty().finish();
  tracing::subscriber::with_default(subscriber, || {
      let span = span_factory.create();
      let _guard = span.enter();
      span.record("dyn_attr_1", &"dyn_attr_1");
      span.record("dyn_attr_2", &"dyn_attr_2");
      span.record("dyn_attr_4", &"dyn_attr_4"); // This attribute was not in the original metadata. It'll be ignored.
      tracing::error!("Hello world");
  });
```


Output
```
  2023-10-26T08:36:43.208049Z ERROR tracing_dynamic::span_factory::test: Hello world
    at src/span_factory.rs:60
    in span_target::span_name with dyn_attr_1: "dyn_attr_1", dyn_attr_2: "dyn_attr_2"
```
