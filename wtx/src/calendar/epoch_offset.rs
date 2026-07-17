use crate::{calendar::CalendarError, sync::AtomicU64};
use core::sync::atomic::Ordering;

const NTP_TO_UNIX_SECS: u64 = 2_208_988_800;

#[derive(Debug)]
pub(crate) struct EpochOffset(AtomicU64);

impl EpochOffset {
  #[inline]
  pub(crate) const fn new() -> Self {
    Self(AtomicU64::new(0))
  }

  #[inline]
  pub(crate) fn get(&self) -> u64 {
    self.0.load(Ordering::Relaxed)
  }

  #[inline]
  pub(crate) fn set(&self, ntp_seconds: u64) -> crate::Result<()> {
    if self.get() > 0 {
      return Err(CalendarError::EpochWasAlreadyAdjusted.into());
    }
    let uptime_secs = crate::calendar::Instant::since_boot_secs()?;
    let offset = ntp_seconds.saturating_sub(NTP_TO_UNIX_SECS).saturating_sub(uptime_secs);
    self.0.store(offset, Ordering::Relaxed);
    Ok(())
  }
}
