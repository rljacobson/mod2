use std::fmt::Debug;
use tracing::{
  field::{Field, Visit},
  Event,
  Subscriber
};
use tracing_subscriber::{
  Layer,
  layer::Context,
  registry::LookupSpan
};

use super::{get_global_logging_threshold};

/// A "layer" that causes the logging system to only log messages at or below the global logging threshold.
/// This baroque machinery is specific to the `tracing` crate.
pub(crate) struct ThresholdFilterLayer;

impl<S> Layer<S> for ThresholdFilterLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
  fn event_enabled(&self, event: &Event<'_>, _ctx: Context<'_, S>) -> bool {
    let mut visitor = ThresholdVisitor { threshold: None };
    event.record(&mut visitor);

    if let Some(threshold_value) = visitor.threshold {
      if threshold_value <= get_global_logging_threshold() {
        // Proceed to log the event by passing it to the next layer
        true
      } else {
        // Event is filtered out.
        false
      }
    } else {
      // No threshold provided; default behavior is to treat as threshold 0, i.e. log the event.
      true
    }
  }
}

/// A "visitor" used for extracting the threshold from log records. Used by `ThresholdFilterLayer`, this is how
/// the `tracing` crate does things.
struct ThresholdVisitor {
  threshold: Option<u8>,
}

impl Visit for ThresholdVisitor {
  fn record_i64(&mut self, field: &Field, value: i64) {
    if field.name() == "threshold" {
      if value >= 0 && value <= u8::MAX as i64 {
        self.threshold = Some(value as u8);
      } else {
        panic!("Invalid threshold value supplied to the logger: {:?} This is an error.", value);
      }
    }
  }

  fn record_u64(&mut self, field: &Field, value: u64) {
    if field.name() == "threshold" {
      if value <= u8::MAX as u64 {
        self.threshold = Some(value as u8);
      } else {
        panic!("Invalid threshold value supplied to the logger: {:?} This is an error.", value);
      }
    }
  }


  fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
    if field.name() == "threshold" {
      // This is an error.
      panic!("Invalid threshold value supplied to the logger: {:?} This is an error.", value);
    }
  }

}
