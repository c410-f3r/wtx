#[cfg(feature = "crossbeam-channel")]
pub use crossbeam::*;
#[cfg(all(feature = "std", feature = "nightly"))]
pub use nightly::*;

use crate::SendError;

/// Multi-producer multi-consumer channel.
pub trait Mpmc<T> {
  /// See [`Receiver`].
  type Receiver: Receiver<T>;
  /// See [`Sender`].
  type Sender: Sender<T>;

  /// Creates a channel of unbounded capacity.
  fn unbounded() -> (Self::Sender, Self::Receiver);
}

/// The receiving side of a channel.
pub trait Receiver<T>: Clone {
  /// Attempts to receive a message from the channel.
  fn recv(&self) -> crate::Result<T>;
}

/// The sending side of a channel.
pub trait Sender<T>: Clone {
  /// Attempts to send a value on this channel, returning it back if it could not be sent.
  fn send(&self, msg: T) -> Result<(), SendError<T>>;
}

#[cfg(feature = "crossbeam-channel")]
mod crossbeam {
  use crate::{RecvError, SendError, sync::Mpmc};
  use crossbeam_channel::{self, TryRecvError, TrySendError};

  /// Uses the channel provided by the `Crossbeam` project.
  #[derive(Debug)]
  pub struct CrossbeamMpmc {
    _nothing: (),
  }

  impl<T> Mpmc<T> for CrossbeamMpmc {
    type Receiver = crossbeam_channel::Receiver<T>;
    type Sender = crossbeam_channel::Sender<T>;

    #[inline]
    fn unbounded() -> (Self::Sender, Self::Receiver) {
      crossbeam_channel::unbounded()
    }
  }

  impl<T> crate::sync::Receiver<T> for crossbeam_channel::Receiver<T> {
    #[inline]
    fn recv(&self) -> crate::Result<T> {
      Ok(self.try_recv().map_err(|err| match err {
        TryRecvError::Empty => RecvError::Empty,
        TryRecvError::Disconnected => RecvError::Disconnected,
      })?)
    }
  }

  impl<T> crate::sync::Sender<T> for crossbeam_channel::Sender<T> {
    #[inline]
    fn send(&self, msg: T) -> Result<(), SendError<T>> {
      self.try_send(msg).map_err(|err| match err {
        TrySendError::Full(elem) => SendError::Full(elem),
        TrySendError::Disconnected(elem) => SendError::Disconnected(elem),
      })
    }
  }
}

#[cfg(all(feature = "std", feature = "nightly"))]
mod nightly {
  use crate::{RecvError, SendError, sync::Mpmc};
  use std::sync::mpmc::{self, TryRecvError, TrySendError};

  /// Uses the channel provided by the standard library.
  #[derive(Debug)]
  pub struct StdMpmc {
    _nothing: (),
  }

  impl<T> Mpmc<T> for StdMpmc {
    type Receiver = mpmc::Receiver<T>;
    type Sender = mpmc::Sender<T>;

    #[inline]
    fn unbounded() -> (Self::Sender, Self::Receiver) {
      mpmc::channel()
    }
  }

  impl<T> crate::sync::Receiver<T> for mpmc::Receiver<T> {
    #[inline]
    fn recv(&self) -> crate::Result<T> {
      Ok(self.try_recv().map_err(|err| match err {
        TryRecvError::Empty => RecvError::Empty,
        TryRecvError::Disconnected => RecvError::Disconnected,
      })?)
    }
  }

  impl<T> crate::sync::Sender<T> for mpmc::Sender<T> {
    #[inline]
    fn send(&self, msg: T) -> Result<(), SendError<T>> {
      self.try_send(msg).map_err(|err| match err {
        TrySendError::Full(elem) => SendError::Full(elem),
        TrySendError::Disconnected(elem) => SendError::Disconnected(elem),
      })
    }
  }
}
