use crate::calendar::Instant;
use core::time::Duration;
use std::fmt::Write;
use tracing_tree::time::FormatTime;

/// Invokes the current time for logging purposes.
#[derive(Debug)]
pub struct TracingTreeTimer;

impl FormatTime for TracingTreeTimer {
  #[inline]
  fn format_time(&self, w: &mut impl Write) -> core::fmt::Result {
    w.write_str(Instant::now_date_time(0).unwrap_or_default().iso_8601().as_str())?;
    Ok(())
  }

  #[inline]
  fn style_timestamp(&self, _: bool, elapsed: Duration, w: &mut impl Write) -> std::fmt::Result {
    let millis = elapsed.as_millis();
    let secs = elapsed.as_secs();
    let (num, unit) = if millis < 1000 {
      (millis as _, "ms")
    } else if secs < 60 {
      (secs, "s ")
    } else {
      (secs / 60, "m ")
    };
    w.write_fmt(format_args!("{num:>3}{unit}"))?;
    Ok(())
  }
}
