use crate::{
  client_api_framework::misc::RequestLimit,
  misc::{sleep, GenericTime},
};
use core::time::Duration;

/// Tracks how many requests were performed in a time interval
#[derive(Clone, Copy, Debug)]
pub struct RequestCounter {
  counter: u16,
  instant: GenericTime,
}

impl RequestCounter {
  /// Instance with valid initial values
  #[inline]
  pub fn new() -> crate::Result<Self> {
    Ok(Self { counter: 0, instant: GenericTime::now()? })
  }

  /// How many requests within the current time-slot are still available for usage.
  #[inline]
  pub fn remaining_requests(&self, rl: &RequestLimit) -> u16 {
    rl.limit().wrapping_sub(self.counter)
  }

  /// If the values defined in [RequestLimit] are in agreement with the `current` values
  /// of [RequestCounter], then return `T`. Otherwise, awaits until [RequestCounter] is updated.
  #[inline]
  pub async fn update_params(&mut self, rl: &RequestLimit) -> crate::Result<()> {
    let now = GenericTime::now()?;
    let duration = *rl.duration();
    let elapsed = now.duration_since(self.instant)?;
    if elapsed > duration {
      _debug!("Elapsed is greater than duration. Re-initializing");
      self.counter = 1;
      self.instant = now;
    } else if self.counter == 0 {
      _debug!("First instance call");
      self.counter = 2;
    } else if self.counter > rl.limit() {
      _debug!("Counter exceeded its limit within max duration");
      self.manage_sleep(elapsed, duration).await?;
      self.counter = 1;
    } else if self.counter == 1 {
      _debug!("First recurrent call");
      self.manage_sleep(elapsed, duration).await?;
      self.counter = self.counter.wrapping_add(1);
    } else if self.counter == rl.limit() {
      _debug!("Counter equals its limit within max duration");
      self.counter = 1;
    } else {
      self.counter = self.counter.wrapping_add(1);
    }
    Ok(())
  }

  async fn manage_sleep(&mut self, elapsed: Duration, duration: Duration) -> crate::Result<()> {
    if let Some(diff) = duration.checked_sub(elapsed) {
      _debug!("Call needs to wait {}ms", diff.as_millis());
      sleep(diff).await?;
      self.instant = GenericTime::now()?;
    }
    Ok(())
  }
}

#[cfg(all(feature = "tokio", test))]
mod tests {
  use crate::client_api_framework::misc::{RequestCounter, RequestLimit};
  use core::time::Duration;
  use std::time::Instant;
  use tokio::time::sleep;

  // Minus 1ms because awaiting is `duration - elapsed` instead of `duration`
  #[tokio::test]
  async fn awaits_when_called_with_counter_reinitialized() {
    const DURATION: Duration = Duration::from_millis(1000);

    let rl = RequestLimit::new(2, DURATION).unwrap();
    let mut rc = RequestCounter::new().unwrap();

    async fn test(first_ms: Duration, rc: &mut RequestCounter, rl: &RequestLimit) {
      let first = Instant::now();
      rc.update_params(rl).await.unwrap();
      assert!(first.elapsed() >= first_ms);

      let second = Instant::now();
      rc.update_params(rl).await.unwrap();
      assert!(second.elapsed() <= Duration::from_millis(2));
    }

    test(Duration::from_millis(0), &mut rc, &rl).await;
    test(DURATION - Duration::from_millis(1), &mut rc, &rl).await;
    test(DURATION - Duration::from_millis(1), &mut rc, &rl).await;
    test(DURATION - Duration::from_millis(1), &mut rc, &rl).await;
    test(DURATION - Duration::from_millis(1), &mut rc, &rl).await;
  }

  #[tokio::test]
  async fn counter_is_reinitialized_when_time_expires() {
    let rl = RequestLimit::new(10, Duration::from_millis(1000)).unwrap();
    let mut rc = RequestCounter::new().unwrap();
    assert_eq!(rc.counter, 0);
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 2);
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 3);
    rc.update_params(&rl).await.unwrap();
    sleep(Duration::from_millis(1110)).await;
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 1);
  }

  #[tokio::test]
  async fn does_not_awaits_when_idle_is_greater_than_duration() {
    let rl = RequestLimit::new(2, Duration::from_millis(50)).unwrap();
    let mut rc = RequestCounter::new().unwrap();

    async fn test(rc: &mut RequestCounter, rl: &RequestLimit) {
      let now = Instant::now();
      rc.update_params(rl).await.unwrap();
      assert!(now.elapsed() <= Duration::from_millis(2));
    }

    test(&mut rc, &rl).await;
    sleep(Duration::from_millis(200)).await;
    test(&mut rc, &rl).await;
    sleep(Duration::from_millis(200)).await;
    test(&mut rc, &rl).await;
    sleep(Duration::from_millis(200)).await;
    test(&mut rc, &rl).await;
    sleep(Duration::from_millis(200)).await;
    test(&mut rc, &rl).await;
    sleep(Duration::from_millis(200)).await;
    test(&mut rc, &rl).await;
    sleep(Duration::from_millis(200)).await;
    test(&mut rc, &rl).await;
    sleep(Duration::from_millis(200)).await;
  }

  #[tokio::test]
  async fn has_correct_counter_increment() {
    let rl = RequestLimit::new(2, Duration::from_millis(100)).unwrap();
    let mut rc = RequestCounter::new().unwrap();
    assert_eq!(rc.counter, 0);
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 2);
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 1);
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 2);
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 1);
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 2);
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 1);
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 2);
    rc.update_params(&rl).await.unwrap();
    assert_eq!(rc.counter, 1);
  }

  #[tokio::test]
  async fn one_value_limit_has_correct_behavior() {
    async fn test(rc: &mut RequestCounter, rl: &RequestLimit, duration: Duration) {
      let now = Instant::now();
      rc.update_params(rl).await.unwrap();
      assert!(now.elapsed() >= duration);
    }

    let _100 = Duration::from_millis(100);
    let rl = RequestLimit::new(1, _100).unwrap();
    let mut rc = RequestCounter::new().unwrap();
    assert_eq!(rc.counter, 0);
    test(&mut rc, &rl, Duration::default()).await;
    assert_eq!(rc.counter, 2);
    test(&mut rc, &rl, _100 - Duration::from_millis(1)).await;
    assert_eq!(rc.counter, 1);
    test(&mut rc, &rl, Duration::default()).await;
    assert_eq!(rc.counter, 2);
    test(&mut rc, &rl, _100 - Duration::from_millis(1)).await;
    assert_eq!(rc.counter, 1);
    test(&mut rc, &rl, Duration::default()).await;
    assert_eq!(rc.counter, 2);
    test(&mut rc, &rl, _100 - Duration::from_millis(1)).await;
    assert_eq!(rc.counter, 1);
    test(&mut rc, &rl, Duration::default()).await;
    assert_eq!(rc.counter, 2);
    test(&mut rc, &rl, _100 - Duration::from_millis(1)).await;
    assert_eq!(rc.counter, 1);
  }
}
