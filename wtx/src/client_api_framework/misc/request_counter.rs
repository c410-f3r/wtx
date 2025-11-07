use crate::{calendar::Instant, client_api_framework::misc::RequestLimit, misc::sleep};

/// Tracks how many requests were performed in a time interval.
#[derive(Clone, Copy, Debug)]
pub struct RequestCounter {
  counter: u16,
  instant: Instant,
  rl: RequestLimit,
}

impl RequestCounter {
  /// Instance with valid initial values
  #[inline]
  pub fn new(rl: RequestLimit) -> Self {
    Self { counter: 0, instant: Instant::now(), rl }
  }

  /// How many requests within the current time-slot are still available for usage.
  #[inline]
  pub const fn remaining_requests(&self) -> u16 {
    self.rl.limit().saturating_sub(self.counter)
  }

  /// If the values defined in [`RequestLimit`] are in agreement with the `current` values
  /// of [RequestCounter], then return `T`. Otherwise, awaits until [RequestCounter] is updated.
  #[inline]
  pub async fn update_params(&mut self) -> crate::Result<()> {
    let now = Instant::now();
    let duration = *self.rl.duration();
    let elapsed = now.duration_since(self.instant)?;
    if elapsed > duration || self.counter == 0 {
      _debug!("Elapsed is greater than duration. Re-initializing");
      self.instant = now;
      self.counter = 1;
      return Ok(());
    }
    if self.counter >= self.rl.limit() {
      if let Some(diff) = duration.checked_sub(elapsed) {
        _debug!("Call needs to wait {}ms", diff.as_millis());
        sleep(diff).await?;
      }
      self.instant = Instant::now();
      self.counter = 1;
    } else {
      self.counter = self.counter.wrapping_add(1);
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    client_api_framework::misc::{RequestCounter, RequestLimit},
    executor::Runtime,
    misc::sleep,
  };
  use core::time::Duration;
  use std::time::Instant;

  #[test]
  fn allows_requests_up_to_limit_then_waits() {
    Runtime::new()
      .block_on(async {
        const LIMIT: u16 = 5;
        const DURATION: Duration = Duration::from_millis(200);

        let rl = RequestLimit::new(LIMIT, DURATION);
        let mut rc = RequestCounter::new(rl);
        let start = Instant::now();

        for i in 0..LIMIT {
          rc.update_params().await.unwrap();
          assert_eq!(rc.counter, i + 1);
        }
        assert!(start.elapsed() < Duration::from_millis(50));

        let wait_start = Instant::now();
        rc.update_params().await.unwrap();

        assert!(wait_start.elapsed() >= DURATION - Duration::from_millis(50));
        assert_eq!(rc.counter, 1);

        let final_call_start = Instant::now();
        rc.update_params().await.unwrap();
        assert!(final_call_start.elapsed() < Duration::from_millis(50));
        assert_eq!(rc.counter, 2);
      })
      .unwrap();
  }

  #[test]
  fn counter_resets_after_time_expires() {
    Runtime::new()
      .block_on(async {
        let rl = RequestLimit::new(10, Duration::from_millis(100));
        let mut rc = RequestCounter::new(rl);
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 1);
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 2);
        sleep(Duration::from_millis(150)).await.unwrap();
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 1);
      })
      .unwrap();
  }

  #[test]
  fn does_not_awaits_when_idle_is_greater_than_duration() {
    Runtime::new()
      .block_on(async {
        let rl = RequestLimit::new(2, Duration::from_millis(50));
        let mut rc = RequestCounter::new(rl);

        async fn test(rc: &mut RequestCounter) {
          let now = Instant::now();
          rc.update_params().await.unwrap();
          assert!(now.elapsed() <= Duration::from_millis(2));
        }

        test(&mut rc).await;
        sleep(Duration::from_millis(100)).await.unwrap();
        test(&mut rc).await;
        sleep(Duration::from_millis(100)).await.unwrap();
        test(&mut rc).await;
        sleep(Duration::from_millis(100)).await.unwrap();
        test(&mut rc).await;
        sleep(Duration::from_millis(100)).await.unwrap();
        test(&mut rc).await;
        sleep(Duration::from_millis(100)).await.unwrap();
        test(&mut rc).await;
        sleep(Duration::from_millis(100)).await.unwrap();
        test(&mut rc).await;
        sleep(Duration::from_millis(100)).await.unwrap();
      })
      .unwrap();
  }

  #[test]
  fn has_correct_counter_increment() {
    Runtime::new()
      .block_on(async {
        let rl = RequestLimit::new(2, Duration::from_millis(100));
        let mut rc = RequestCounter::new(rl);
        assert_eq!(rc.counter, 0);
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 1);
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 2);
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 1);
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 2);
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 1);
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 2);
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 1);
        rc.update_params().await.unwrap();
        assert_eq!(rc.counter, 2);
      })
      .unwrap();
  }

  #[test]
  fn one_request_limit_waits_each_time() {
    Runtime::new()
      .block_on(async {
        const DURATION: Duration = Duration::from_millis(100);
        let rl = RequestLimit::new(1, DURATION);
        let mut rc = RequestCounter::new(rl);

        let first_call = Instant::now();
        rc.update_params().await.unwrap();
        assert!(first_call.elapsed() < Duration::from_millis(50));
        assert_eq!(rc.counter, 1);

        let second_call = Instant::now();
        rc.update_params().await.unwrap();
        assert!(second_call.elapsed() >= DURATION - Duration::from_millis(50));
        assert_eq!(rc.counter, 1);

        let third_call = Instant::now();
        rc.update_params().await.unwrap();
        assert!(third_call.elapsed() >= DURATION - Duration::from_millis(50));
        assert_eq!(rc.counter, 1);
      })
      .unwrap();
  }

  #[test]
  fn window_starts_at_first_use_not_at_creation() {
    Runtime::new()
      .block_on(async {
        const DURATION: Duration = Duration::from_millis(200);

        let rl = RequestLimit::new(2, DURATION);
        let mut rc = RequestCounter::new(rl);

        sleep(Duration::from_millis(150)).await.unwrap();

        rc.update_params().await.unwrap();
        rc.update_params().await.unwrap();

        let start = Instant::now();
        rc.update_params().await.unwrap();
        let elapsed = start.elapsed();

        let expected_wait = DURATION.saturating_sub(Duration::from_millis(20));
        assert!(elapsed >= expected_wait);
      })
      .unwrap();
  }
}
