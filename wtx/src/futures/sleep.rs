use crate::misc::{Lease, LeaseMut};
use core::{
  pin::Pin,
  task::{Context, Poll},
  time::Duration,
};

cfg_select! {
  feature = "embassy-time" => {
    /// Waits until a certain duration has elapsed.
    #[derive(Debug)]
    pub struct Sleep {
      item: embassy_time::Timer
    }
  }
  feature = "tokio" => {
    pin_project_lite::pin_project! {
      /// Waits until a certain duration has elapsed.
      #[derive(Debug)]
      pub struct Sleep {
        #[pin]
        item: tokio::time::Sleep
      }
    }
  }
  _ => {
    /// Waits until a certain duration has elapsed.
    #[derive(Debug)]
    pub struct Sleep {
      item: (Duration, crate::calendar::Instant)
    }
  }
}

impl Sleep {
  /// New instance
  #[inline]
  pub fn new(duration: Duration) -> crate::Result<Self> {
    Ok(Self {
      item: cfg_select! {
        feature = "embassy-time" => embassy_time::Timer::after(duration.try_into()?),
        feature = "tokio" => tokio::time::sleep(duration),
        _ => (duration, crate::calendar::Instant::new())
      },
    })
  }
}

impl Lease<Sleep> for Sleep {
  #[inline]
  fn lease(&self) -> &Sleep {
    self
  }
}

impl LeaseMut<Sleep> for Sleep {
  #[inline]
  fn lease_mut(&mut self) -> &mut Sleep {
    self
  }
}

impl Future for Sleep {
  type Output = crate::Result<()>;

  #[allow(unused_mut, reason = "depends on feature")]
  #[inline]
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    cfg_select! {
      feature = "embassy-time" => {
        let Self { item } = &mut *self;
        core::task::ready!(core::pin::pin!(item).as_mut().poll(cx));
        Poll::Ready(Ok(()))
      },
      feature = "tokio" => {
        let projection = self.project();
        core::task::ready!(projection.item.poll(cx));
        Poll::Ready(Ok(()))
      },
      _ => {
        let Self { item } = &mut *self;
        if item.1.elapsed()? >= item.0 {
          return Poll::Ready(crate::Result::Ok(()));
        }
        cx.waker().wake_by_ref();
        Poll::Pending
      }
    }
  }
}
