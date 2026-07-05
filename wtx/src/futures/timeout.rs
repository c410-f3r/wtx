use crate::futures::Sleep;
use core::{
  pin::{Pin, pin},
  task::{Context, Poll},
  time::Duration,
};

cfg_select! {
  feature = "tokio" => {
    pin_project_lite::pin_project! {
      /// Requires a `Future` to complete within a specified duration.
      #[derive(Debug)]
      pub struct Timeout<F> {
        future: F,
        #[pin]
        sleep: Sleep,
      }
    }
  }
  _ => {
    /// Requires a `Future` to complete within a specified duration.
    #[derive(Debug)]
    pub struct Timeout<F> {
      future: F,
      sleep: Sleep,
    }
  }
}

impl<F> Timeout<F> {
  /// New instance from raw parts.
  #[inline]
  pub const fn new(future: F, sleep: Sleep) -> Self {
    Self { future, sleep }
  }
}

impl<F> Timeout<F> {
  /// New instance from a duration that creates an owned [`Sleep`].
  #[inline]
  pub fn from_duration(future: F, duration: Duration) -> crate::Result<Self> {
    Ok(Self { future, sleep: Sleep::new(duration)? })
  }
}

impl<F> Future for Timeout<F>
where
  F: Future + Unpin,
{
  type Output = crate::Result<Option<F::Output>>;

  #[allow(unused_mut, reason = "depends on feature")]
  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    #[cfg(feature = "tokio")]
    let poll = {
      let this = self.project();
      if let Poll::Ready(output) = pin!(this.future).as_mut().poll(cx) {
        return Poll::Ready(Ok(Some(output)));
      }
      this.sleep.poll(cx)
    };
    #[cfg(not(feature = "tokio"))]
    let poll = {
      if let Poll::Ready(output) = pin!(&mut self.future).as_mut().poll(cx) {
        return Poll::Ready(Ok(Some(output)));
      }
      pin!(&mut self.sleep).as_mut().poll(cx)
    };
    match poll {
      Poll::Ready(Ok(_)) => Poll::Ready(Ok(None)),
      Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
      Poll::Pending => Poll::Pending,
    }
  }
}

#[cfg(all(not(feature = "tokio"), test))]
mod tests {
  use crate::futures::{Sleep, Timeout};
  use core::{pin::pin, time::Duration};

  #[wtx::test]
  async fn timeout() {
    assert_eq!(
      Timeout::new(pin!(async { 1 }), Sleep::new(Duration::from_millis(10)).unwrap())
        .await
        .unwrap(),
      Some(1)
    );
    assert_eq!(
      Timeout::new(
        pin!(async {
          Sleep::new(Duration::from_millis(20)).unwrap().await.unwrap();
          1
        }),
        Sleep::new(Duration::from_millis(10)).unwrap()
      )
      .await
      .unwrap(),
      None
    );
  }
}
