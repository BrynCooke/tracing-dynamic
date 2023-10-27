use tracing::Span;
use tracing_core::callsite::DefaultCallsite;
use tracing_core::field::FieldSet;
use tracing_core::{Callsite, Interest, Kind, Level, Metadata};

/// SpanFactory allows creating spans with attributes defined at runtime.
/// It leaks memory to do this, so create span factories once and reuse them.
pub struct SpanFactory {
    callsite: &'static LateCallsite,
}

impl SpanFactory {
    /// Create a span factory
    pub fn new(
        name: &str,
        target: &str,
        level: Level,
        file: Option<&str>,
        line: Option<u32>,
        module_path: Option<&str>,
        fields: &[&str],
    ) -> Self {
        // tracing expects everything to be static, so we need to leak everything
        let name = name.to_string().leak();
        let target = target.to_string().leak();
        let file = file.map(|s| s.to_string().leak() as &'static str);
        let module_path = module_path.map(|s| s.to_string().leak() as &'static str);

        let callsite: &'static mut LateCallsite = Box::leak(Box::default());
        let fields = Box::leak(Box::new(
            fields
                .iter()
                .map(|k| k.to_string().leak() as &'static str)
                .collect::<Vec<&'static str>>(),
        ));
        let metadata = Box::leak(Box::new(Metadata::new(
            name,
            target,
            level,
            file,
            line,
            module_path,
            FieldSet::new(fields, tracing_core::identify_callsite!(callsite)),
            Kind::SPAN,
        )));

        callsite
            .delegate
            .set(DefaultCallsite::new(metadata))
            .expect("cannot initialize callsite");

        SpanFactory { callsite }
    }

    pub fn create(&self) -> Span {
        let enabled = ::tracing::Level::INFO <= ::tracing::level_filters::STATIC_MAX_LEVEL
            && *self.callsite.metadata().level()
                <= ::tracing::level_filters::LevelFilter::current()
            && {
                let interest = self.callsite.interest();
                !interest.is_never()
                    && ::tracing::__macro_support::__is_enabled(self.callsite.metadata(), interest)
            };
        if enabled {
            Span::new(
                self.callsite.metadata(),
                &self.callsite.metadata().fields().value_set(&[]),
            )
        } else {
            Span::new_disabled(self.callsite.metadata())
        }
    }
}

#[derive(Default)]
struct LateCallsite {
    delegate: std::sync::OnceLock<DefaultCallsite>,
}

impl Callsite for LateCallsite {
    fn set_interest(&self, interest: Interest) {
        self.delegate.get().unwrap().set_interest(interest);
    }

    fn metadata(&self) -> &Metadata<'_> {
        self.delegate.get().unwrap().metadata()
    }
}

impl LateCallsite {
    #[inline]
    fn interest(&'static self) -> Interest {
        self.delegate
            .get()
            .expect("cannot get delegate callsite, this is a bug")
            .interest()
    }
}

#[cfg(test)]
mod test {
    use crate::span_factory::SpanFactory;
    
    
    
    use std::sync::{Arc, Mutex};
    use tracing_core::Level;
    

    #[derive(Clone, Default)]
    struct TestWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl std::io::Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let buf_len = buf.len();
            self.buffer.lock().unwrap().append(&mut buf.to_vec());
            Ok(buf_len)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn test_unfiltered() {
        test(Level::INFO)
    }
    #[test]
    fn test_filtered() {
        test(Level::ERROR)
    }

    fn test(subscriber_level: Level) {
        let test_writer = TestWriter::default();
        let attrs = vec!["dyn_attr_1", "dyn_attr_2"];
        let span_factory = SpanFactory::new(
            "span_name",
            "span_target",
            Level::INFO,
            None,
            None,
            None,
            &attrs,
        );
        let subscriber = tracing_subscriber::fmt()
            .json()
            .without_time()
            .with_file(false)
            .with_line_number(false)
            .with_max_level(subscriber_level)
            .with_writer(Mutex::new(test_writer.clone()))
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            let span = span_factory.create();
            let _guard = span.enter();
            span.record("dyn_attr_1", "dyn_attr_1");
            span.record("dyn_attr_2", "dyn_attr_2");
            span.record("dyn_attr_4", "dyn_attr_4");
            tracing::error!("Hello world");
        });

        insta::assert_snapshot!(
            String::from_utf8(test_writer.buffer.lock().unwrap().to_vec()).unwrap()
        );
    }
}
