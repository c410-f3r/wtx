#[allow(unused_imports, reason = "Depends on the selection of features")]
use alloc::boxed::Box;

#[cfg(feature = "crypto-ruco")]
impl From<aead::Error> for crate::Error {
  #[inline]
  #[track_caller]
  fn from(from: aead::Error) -> Self {
    Self::AeadError(from)
  }
}

#[cfg(feature = "argon2")]
impl From<argon2::Error> for crate::Error {
  #[inline]
  #[track_caller]
  fn from(from: argon2::Error) -> Self {
    Self::Argon2(from)
  }
}

#[cfg(feature = "crypto-ruco")]
impl From<crypto_common::InvalidLength> for crate::Error {
  #[inline]
  #[track_caller]
  fn from(from: crypto_common::InvalidLength) -> Self {
    Self::CryptoCommonInvalidLength(from)
  }
}

#[cfg(feature = "crypto-ruco")]
impl From<elliptic_curve::Error> for crate::Error {
  #[inline]
  fn from(from: elliptic_curve::Error) -> Self {
    Self::EllipticCurveError(from)
  }
}

#[cfg(feature = "embassy-net")]
impl From<embassy_net::tcp::Error> for crate::Error {
  #[inline]
  fn from(from: embassy_net::tcp::Error) -> Self {
    Self::EmbassyNetTcp(from)
  }
}

#[cfg(feature = "embassy-net")]
impl From<embassy_net::udp::BindError> for crate::Error {
  #[inline]
  fn from(from: embassy_net::udp::BindError) -> Self {
    Self::EmbassyNetUdpBind(from)
  }
}

#[cfg(feature = "embassy-net")]
impl From<embassy_net::udp::RecvError> for crate::Error {
  #[inline]
  fn from(from: embassy_net::udp::RecvError) -> Self {
    Self::EmbassyNetUdpRecv(from)
  }
}

#[cfg(feature = "embassy-net")]
impl From<embassy_net::udp::SendError> for crate::Error {
  #[inline]
  fn from(from: embassy_net::udp::SendError) -> Self {
    Self::EmbassyNetUdpSend(from)
  }
}

#[cfg(feature = "getrandom")]
impl From<getrandom::Error> for crate::Error {
  #[inline]
  fn from(from: getrandom::Error) -> Self {
    Self::GetRandomError(from)
  }
}

#[cfg(feature = "crypto-graviola")]
impl From<graviola::Error> for crate::Error {
  #[inline]
  fn from(from: graviola::Error) -> Self {
    Self::GraviolaError(from)
  }
}

#[cfg(feature = "httparse")]
impl From<httparse::Error> for crate::Error {
  #[inline]
  fn from(from: httparse::Error) -> Self {
    Self::HttpParse(from)
  }
}

#[cfg(feature = "crypto-ruco")]
impl From<digest::MacError> for crate::Error {
  #[inline]
  fn from(from: digest::MacError) -> Self {
    Self::MacError(from)
  }
}

#[cfg(feature = "crypto-ruco")]
impl From<pkcs1::Error> for crate::Error {
  #[inline]
  fn from(from: pkcs1::Error) -> Self {
    Self::Pkcs1Error(from.into())
  }
}

#[cfg(feature = "crypto-ruco")]
impl From<pkcs8::Error> for crate::Error {
  #[inline]
  fn from(from: pkcs8::Error) -> Self {
    Self::Pkcs8Error(from.into())
  }
}

#[cfg(feature = "quick-protobuf")]
impl From<quick_protobuf::Error> for crate::Error {
  #[inline]
  fn from(from: quick_protobuf::Error) -> Self {
    Self::QuickProtobuf(from.into())
  }
}

#[cfg(feature = "serde")]
impl From<::serde::de::value::Error> for crate::Error {
  #[inline]
  fn from(from: ::serde::de::value::Error) -> Self {
    Self::SerdeDeValue(from.into())
  }
}

#[cfg(feature = "serde_json")]
impl From<serde_json::Error> for crate::Error {
  #[inline]
  fn from(from: serde_json::Error) -> Self {
    Self::SerdeJson(from)
  }
}

#[cfg(feature = "crypto-ruco")]
impl From<signature::Error> for crate::Error {
  #[inline]
  fn from(from: signature::Error) -> Self {
    Self::Signature(from.into())
  }
}

#[cfg(feature = "crypto-ruco")]
impl From<spki::Error> for crate::Error {
  #[inline]
  fn from(from: spki::Error) -> Self {
    Self::SpkiError(from.into())
  }
}

#[cfg(feature = "tokio")]
impl From<tokio::task::JoinError> for crate::Error {
  #[inline]
  fn from(from: tokio::task::JoinError) -> Self {
    Self::TokioJoinError(from.into())
  }
}

#[cfg(feature = "tracing-subscriber")]
impl From<tracing_subscriber::util::TryInitError> for crate::Error {
  #[inline]
  fn from(from: tracing_subscriber::util::TryInitError) -> Self {
    Self::TryInitError(from.into())
  }
}

#[cfg(feature = "std")]
impl<T> From<std::sync::TryLockError<T>> for crate::Error {
  #[inline]
  fn from(from: std::sync::TryLockError<T>) -> Self {
    Self::TryLockError(match from {
      std::sync::TryLockError::Poisoned(_) => {
        std::sync::TryLockError::Poisoned(std::sync::PoisonError::new(()))
      }
      std::sync::TryLockError::WouldBlock => std::sync::TryLockError::WouldBlock,
    })
  }
}

#[cfg(feature = "uuid")]
impl From<uuid::Error> for crate::Error {
  #[inline]
  fn from(value: uuid::Error) -> Self {
    Self::UuidError(value.into())
  }
}

#[cfg(feature = "zlib-rs")]
impl From<zlib_rs::DeflateError> for crate::Error {
  #[inline]
  fn from(value: zlib_rs::DeflateError) -> Self {
    Self::ZlibRsDeflateError(value)
  }
}

#[cfg(feature = "zlib-rs")]
impl From<zlib_rs::InflateError> for crate::Error {
  #[inline]
  fn from(value: zlib_rs::InflateError) -> Self {
    Self::ZlibRsInflateError(value)
  }
}

#[cfg(feature = "serde")]
mod serde {
  use alloc::string::ToString as _;
  use core::fmt::Display;

  impl serde::ser::Error for crate::Error {
    #[inline]
    fn custom<T>(msg: T) -> Self
    where
      T: Display,
    {
      let mut string = msg.to_string();
      string.truncate(1024);
      Self::Generic(string.try_into().unwrap_or_default())
    }
  }
}
