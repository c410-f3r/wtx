use alloc::{string::String, vec};
use core::{
  iter,
  net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
  option, slice,
};

use crate::executor::Executor;

/// A trait for objects which can be converted or resolved to one or more
/// [`SocketAddr`] values.
pub trait ToSocketAddrs {
  /// Returned iterator over socket addresses which this type may correspond
  /// to.
  type Iter: Iterator<Item = SocketAddr>;

  /// Converts this object to an iterator of resolved [`SocketAddr`]s.
  fn to_socket_addrs<EX>(&self, executor: &EX) -> impl Future<Output = crate::Result<Self::Iter>>
  where
    EX: Executor;
}

impl ToSocketAddrs for SocketAddr {
  type Iter = option::IntoIter<SocketAddr>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, _executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    Ok(Some(*self).into_iter())
  }
}

impl ToSocketAddrs for SocketAddrV4 {
  type Iter = option::IntoIter<SocketAddr>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    SocketAddr::V4(*self).to_socket_addrs(executor).await
  }
}

impl ToSocketAddrs for SocketAddrV6 {
  type Iter = option::IntoIter<SocketAddr>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    SocketAddr::V6(*self).to_socket_addrs(executor).await
  }
}

impl ToSocketAddrs for (IpAddr, u16) {
  type Iter = option::IntoIter<SocketAddr>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    let (ip, port) = *self;
    match ip {
      IpAddr::V4(el) => (el, port).to_socket_addrs(executor).await,
      IpAddr::V6(el) => (el, port).to_socket_addrs(executor).await,
    }
  }
}

impl ToSocketAddrs for (Ipv4Addr, u16) {
  type Iter = option::IntoIter<SocketAddr>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    let (ip, port) = *self;
    SocketAddrV4::new(ip, port).to_socket_addrs(executor).await
  }
}

impl ToSocketAddrs for (Ipv6Addr, u16) {
  type Iter = option::IntoIter<SocketAddr>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    let (ip, port) = *self;
    SocketAddrV6::new(ip, port, 0, 0).to_socket_addrs(executor).await
  }
}

impl ToSocketAddrs for (&str, u16) {
  type Iter = vec::IntoIter<SocketAddr>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, _executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    cfg_select! {
      feature = "tokio" => {
        if EX::TY.is_std() {
          Ok(<Self as std::net::ToSocketAddrs>::to_socket_addrs(self)?)
        } else {
          Ok(vec::Vec::from_iter(tokio::net::lookup_host(self).await?).into_iter())
        }
      },
      feature = "std" => Ok(<Self as std::net::ToSocketAddrs>::to_socket_addrs(self)?),
      _ => return Err(crate::net::NetError::NoResolutionBackend.into()),
    }
  }
}

impl ToSocketAddrs for (String, u16) {
  type Iter = vec::IntoIter<SocketAddr>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    (&*self.0, self.1).to_socket_addrs(executor).await
  }
}

impl ToSocketAddrs for str {
  type Iter = vec::IntoIter<SocketAddr>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, _executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    cfg_select! {
      feature = "tokio" => {
        if EX::TY.is_std() {
          Ok(<Self as std::net::ToSocketAddrs>::to_socket_addrs(self)?)
        } else {
          Ok(vec::Vec::from_iter(tokio::net::lookup_host(self).await?).into_iter())
        }
      }
      feature = "std" => Ok(<Self as std::net::ToSocketAddrs>::to_socket_addrs(self)?),
      _ => return Err(crate::net::NetError::NoResolutionBackend.into()),
    }
  }
}
impl<'any> ToSocketAddrs for &'any [SocketAddr] {
  type Iter = iter::Copied<slice::Iter<'any, SocketAddr>>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, _executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    Ok(self.iter().copied())
  }
}

impl<T: ToSocketAddrs + ?Sized> ToSocketAddrs for &T {
  type Iter = T::Iter;

  #[inline]
  async fn to_socket_addrs<EX>(&self, executor: &EX) -> crate::Result<T::Iter>
  where
    EX: Executor,
  {
    (**self).to_socket_addrs(executor).await
  }
}

impl ToSocketAddrs for String {
  type Iter = vec::IntoIter<SocketAddr>;

  #[inline]
  async fn to_socket_addrs<EX>(&self, executor: &EX) -> crate::Result<Self::Iter>
  where
    EX: Executor,
  {
    (**self).to_socket_addrs(executor).await
  }
}
