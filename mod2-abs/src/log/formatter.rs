use tracing::field::{Field, Visit};
use tracing_subscriber::{
  field::RecordFields,
  fmt::{
    format::Writer,
    FormatFields
  }
};

pub(crate) struct CustomFieldFormatter;

impl<'writer> FormatFields<'writer> for CustomFieldFormatter {
  fn format_fields<R: RecordFields>(
    &self,
    writer: Writer<'writer>,
    fields: R,
  ) -> std::fmt::Result {
    let mut visitor = FieldFilterVisitor { writer };
    fields.record(&mut visitor);
    Ok(())
  }

  /*
  fn add_fields(
    &self,
    current: &'writer mut DebugMap<'_>,
    fields: &span::Record<'_>,
  ) -> std::fmt::Result {
    let mut visitor = DebugMapVisitor { map: current };
    fields.record(&mut visitor);
    Ok(())
  }
  */
}

struct FieldFilterVisitor<'writer> {
  writer: Writer<'writer>,
}

impl<'writer> Visit for FieldFilterVisitor<'writer> {
  fn record_i64(&mut self, field: &Field, value: i64) {
    if field.name() != "threshold" {
      let _ = write!(self.writer, "{}={} ", field.name(), value);
    }
  }

  fn record_u64(&mut self, field: &Field, value: u64) {
    if field.name() != "threshold" {
      let _ = write!(self.writer, "{}={} ", field.name(), value);
    }
  }

  fn record_bool(&mut self, field: &Field, value: bool) {
    if field.name() != "critical" {
      let _ = write!(self.writer, "{}={} ", field.name(), value);
    } else if value{
      // Format a critical error.
      let _ = write!(self.writer, "[CRITICAL] ");
    }
  }

  fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
    match field.name() {

      "message" => {
        let _ = write!(self.writer, "{:?}", value);
      }

      "critical" => {
        let _ = write!(self.writer, "[CRITICAL] ");
      }

      "threshold" => {
        // Do not print.
      }

      name => {
        let _ = write!(self.writer, "{}={:?} ", name, value);
      }

    }
  }

}

/*
struct DebugMapVisitor<'a, 'writer> {
  map: &'writer mut fmt::DebugMap<'a>,
}

impl<'a, 'writer> Visit for DebugMapVisitor<'a, 'writer> {
  fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
    if field.name() != "threshold" {
      self.map.entry(&field.name(), value);
    }
  }

  // Implement other `record_*` methods as needed
}
*/
