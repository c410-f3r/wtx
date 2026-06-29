use crate::calendar::Instant;
use core::{fmt::Write, time::Duration};
use tracing_tree::time::FormatTime;

/// Invokes the current time for logging purposes.
#[derive(Debug)]
pub struct TracingTreeTimer {}

impl FormatTime for TracingTreeTimer {
  #[inline]
  fn format_time(&self, w: &mut impl Write) -> core::fmt::Result {
    w.write_str(Instant::now_date_time(0).unwrap_or_default().trunc_to_ms().iso8601().as_str())
  }

  #[inline]
  fn style_timestamp(&self, _: bool, elapsed: Duration, w: &mut impl Write) -> core::fmt::Result {
    let millis = elapsed.as_millis();
    write!(w, "{millis:>4}ms")
  }
}
