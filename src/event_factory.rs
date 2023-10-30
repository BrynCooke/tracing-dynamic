use std::collections::BTreeMap;
use tracing_core::callsite::DefaultCallsite;
use tracing_core::field::{FieldSet, Value};
use tracing_core::{Callsite, Field, Interest, Kind, Level, Metadata};

/// Event allows creating events with attributes defined at runtime.
/// It leaks memory to do this, so create event factories once and reuse them.
pub struct EventFactory {
    callsite: &'static LateCallsite,
    fields: &'static BTreeMap<&'static str, Field>,
}

impl EventFactory {
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

        let fields = Box::leak(Box::new(
            metadata.fields().iter().map(|f| (f.name(), f)).collect(),
        ));
        EventFactory { callsite, fields }
    }

    pub fn create(&self) -> Event {
        let enabled = ::tracing::Level::INFO <= ::tracing::level_filters::STATIC_MAX_LEVEL
            && *self.callsite.metadata().level()
                <= ::tracing::level_filters::LevelFilter::current()
            && {
                let interest = self.callsite.interest();
                !interest.is_never()
                    && ::tracing::__macro_support::__is_enabled(self.callsite.metadata(), interest)
            };
        if enabled {
            Event::Enabled {
                metadata: self.callsite.metadata(),
                values: vec![],
                fields: self.fields,
            }
        } else {
            Event::Disabled
        }
    }
}
pub enum Event<'a> {
    Enabled {
        metadata: &'static Metadata<'static>,
        fields: &'static BTreeMap<&'static str, Field>,
        values: Vec<(&'static Field, Option<&'a dyn Value>)>,
    },
    Disabled,
}

impl<'a> Event<'a> {
    #[inline]
    pub fn with(&mut self, name: &str, value: &'a dyn Value) -> &mut Self {
        if let Event::Enabled { values, fields, .. } = self {
            if let Some(field) = fields.get(name) {
                values.push((field, Some(value)));
            }
        }
        self
    }
    #[inline]
    pub fn finish(&self) {
        if let Event::Enabled {
            metadata, values, ..
        } = self
        {
            macro_rules! finish {
                ($n:literal) => {
                    if $n == values.len() {
                        let values: [(&'static Field, Option<&'a (dyn Value)>); $n] =
                            values.as_slice().try_into().unwrap();
                        let value_set = metadata.fields().value_set(&values);
                        ::tracing::Event::dispatch(metadata, &value_set);
                        return;
                    }
                };
            }
            if values.len() < 10 {
                finish!(0);
                finish!(1);
                finish!(2);
                finish!(3);
                finish!(4);
                finish!(5);
                finish!(6);
                finish!(7);
                finish!(8);
                finish!(9);
            }
            if values.len() < 20 {
                finish!(10);
                finish!(11);
                finish!(12);
                finish!(13);
                finish!(14);
                finish!(15);
                finish!(16);
                finish!(17);
                finish!(18);
                finish!(19);
            }
            if values.len() < 30 {
                finish!(20);
                finish!(21);
                finish!(22);
                finish!(23);
                finish!(24);
                finish!(25);
                finish!(26);
                finish!(27);
                finish!(28);
                finish!(29);
            }
            if values.len() < 40 {
                finish!(30);
                finish!(31);
                finish!(32);
                finish!(33);
                finish!(34);
                finish!(35);
                finish!(36);
                finish!(37);
                finish!(38);
                finish!(39);
            }
            if values.len() < 50 {
                finish!(40);
                finish!(41);
                finish!(42);
                finish!(43);
                finish!(44);
                finish!(45);
                finish!(46);
                finish!(47);
                finish!(48);
                finish!(49);
            }
            if values.len() < 60 {
                finish!(50);
                finish!(51);
                finish!(52);
                finish!(53);
                finish!(54);
                finish!(55);
                finish!(56);
                finish!(57);
                finish!(58);
                finish!(59);
            }
            if values.len() < 70 {
                finish!(60);
                finish!(61);
                finish!(62);
                finish!(63);
                finish!(64);
                finish!(65);
                finish!(66);
                finish!(67);
                finish!(68);
                finish!(69);
            }
            if values.len() < 80 {
                finish!(70);
                finish!(71);
                finish!(72);
                finish!(73);
                finish!(74);
                finish!(75);
                finish!(76);
                finish!(77);
                finish!(78);
                finish!(79);
            }
            if values.len() < 90 {
                finish!(90);
                finish!(91);
                finish!(92);
                finish!(93);
                finish!(94);
                finish!(95);
                finish!(96);
                finish!(97);
                finish!(98);
                finish!(99);
            }
            if values.len() < 110 {
                finish!(100);
            }
            panic!("too many attributes, up to 100 are supported")
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
    use std::sync::{Arc, Mutex};

    use crate::event_factory::EventFactory;
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
        let event_factory = EventFactory::new(
            "event_name",
            "event_target",
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
            let mut event = event_factory.create();
            event.with("dyn_attr_1", &"dyn_attr_1");
            event.with("dyn_attr_2", &"dyn_attr_2");
            event.with("dyn_attr_4", &"dyn_attr_4");
            event.finish();
        });

        insta::assert_snapshot!(
            subscriber_level.to_string(),
            String::from_utf8(test_writer.buffer.lock().unwrap().to_vec()).unwrap()
        );
    }
}
