use tracing::Span;
use tracing_core::{Callsite, Interest, Kind, Level, Metadata};
use tracing_core::callsite::DefaultCallsite;
use tracing_core::field::FieldSet;

/// SpanFactory allows creating spans with attributes defined at runtime.
/// It leaks memory to do this, so create span factories once and reuse them.
pub struct SpanFactory {
    callsite: &'static dyn Callsite,
}

impl SpanFactory {
    /// Create a span factory
    pub fn new(name: &str,
               target: &str,
               level: Level,
               file: Option<&str>,
               line: Option<u32>,
               module_path: Option<&str>,
               fields: &[&str]) -> Self {

        // tracing expects everything to be static, so we need to leak everything
        let name = name.to_string().leak();
        let target = target.to_string().leak();
        let file = file.map(|s| s.to_string().leak() as &'static str);
        let module_path = module_path.map(|s| s.to_string().leak() as &'static str);


        let callsite : &'static mut LateCallsite = Box::leak(Box::new(LateCallsite::default()));
        let fields = Box::leak(Box::new(fields.iter().map(|k| k.to_string().leak() as &'static str).collect::<Vec<&'static str>>()));
        let metadata = Box::leak(Box::new(Metadata::new(name, target, level, file, line, module_path, FieldSet::new(fields, tracing_core::identify_callsite!(callsite)), Kind::SPAN)));

        callsite.delegate.set( DefaultCallsite::new(metadata)).expect("cannot initialize callsite");

        SpanFactory {
            callsite,
        }
    }

    pub fn create(&self) -> Span {
        Span::new(self.callsite.metadata(), &self.callsite.metadata().fields().value_set(&[]))
    }
}

#[derive(Default)]
struct LateCallsite {
    delegate: std::sync::OnceLock<DefaultCallsite>
}

impl Callsite for LateCallsite {
    fn set_interest(&self, interest: Interest) {
        self.delegate.get().unwrap().set_interest(interest);
    }

    fn metadata(&self) -> &Metadata<'_> {
        self.delegate.get().unwrap().metadata()
    }
}

#[cfg(test)]
mod test {
    use crate::span_factory::SpanFactory;

    #[test]
    fn test() {
        let attrs = vec!["dyn_attr_1", "dyn_attr_2"];
        let span_factory = SpanFactory::new("span_name", "span_target", tracing::Level::INFO, None, None, None, &attrs);

        let subscriber = tracing_subscriber::fmt().pretty().finish();
        tracing::subscriber::with_default(subscriber, || {
            let span = span_factory.create();
            let _guard = span.enter();
            span.record("dyn_attr_1", &"dyn_attr_1");
            span.record("dyn_attr_2", &"dyn_attr_2");
            span.record("dyn_attr_4", &"dyn_attr_4");
            tracing::error!("Hello world");
        });
    }
}
