use hashbrown::HashMap;
use ratatui::prelude::Stylize;
use std::sync::{Arc, Mutex};
use tracing::Level;
use tracing_subscriber::Layer;

use crate::tui::widget::LogBuffer;

pub struct LogBufferLayer {
    lb: Arc<Mutex<LogBuffer>>,
}

impl LogBufferLayer {
    pub fn new(lb: Arc<Mutex<LogBuffer>>) -> Self {
        Self { lb }
    }
}

impl<S> Layer<S> for LogBufferLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let now = chrono::Utc::now();
        let level = match *event.metadata().level() {
            Level::TRACE => "TRACE".cyan(),
            Level::DEBUG => "DEBUG".light_magenta(),
            Level::INFO => "INFO".light_green(),
            Level::WARN => "WARN".light_yellow(),
            Level::ERROR => "ERROR".light_red(),
        };

        let mut visitor = HashMapVisitor::default();
        event.record(&mut visitor);
        let fields = visitor.unwrap();

        let content = if fields.len() == 1 && fields.contains_key("message") {
            fields.get("message").unwrap().clone()
        } else {
            use std::fmt::Write;
            let mut buf = String::new();

            for (key, value) in fields {
                write!(buf, "  {key}={value}").expect("Infallible write failed!");
            }

            buf.trim().to_owned()
        }
        .into();

        self.lb
            .lock()
            .expect("Tracing LogBuffer was poisoned!")
            .push_line(now, level.into(), content);
    }
}

#[derive(Default)]
struct HashMapVisitor {
    buf: HashMap<String, String>,
}

impl HashMapVisitor {
    pub fn unwrap(self) -> HashMap<String, String> {
        self.buf
    }
}

impl tracing::field::Visit for HashMapVisitor {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        self.buf.insert(field.name().to_string(), value.to_string());
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.buf.insert(field.name().to_string(), value.to_string());
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.buf.insert(field.name().to_string(), value.to_string());
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.buf.insert(field.name().to_string(), value.to_string());
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.buf.insert(field.name().to_string(), value.to_string());
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.buf.insert(field.name().to_string(), value.to_string());
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.buf
            .insert(field.name().to_string(), format!("{value:?}"));
    }
}
