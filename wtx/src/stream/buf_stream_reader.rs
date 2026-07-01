use crate::{
  collections::Vector,
  misc::{Lease, LeaseMut},
  stream::StreamReader,
};
use core::{fmt::Debug, hint::cold_path, mem::MaybeUninit};

/// Buffered stream reader error
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BufStreamReaderError {
  /// A connection was unexpectedly closed by an external actor.
  ClosedConnection,
  /// External actor sent a payload greater than the maximum capacity
  CapacityOverflow,
  /// The instance is configured to prevent the removal of contents
  ForbiddenClear,
}

/// Buffered stream reader
///
/// ```txt
/// Antecedent | Current | Following | Trailing | Unallocated |
///            |         |           |          |             |
///            |         |           |          |             |-> capacity_ub
///            |         |           |          |
///            |         |           |          |---------------> buffer.capacity()
///            |         |           |
///            |         |           |--------------------------> buffer.len()
///            |         |
///            |         |--------------------------------------> current_end_idx
///            |
///            |------------------------------------------------> antecedent_end_idx
/// ```
pub struct BufStreamReader {
  antecedent_end_idx: usize,
  buffer: Vector<u8>,
  capacity_ub: usize,
  current_end_idx: usize,
  forbid_clear: bool,
}

impl BufStreamReader {
  /// Empty instance with a default upper bound capacity
  #[inline]
  pub const fn new() -> Self {
    Self {
      antecedent_end_idx: 0,
      buffer: Vector::new(),
      capacity_ub: 1024 * 1024 * 32,
      current_end_idx: 0,
      forbid_clear: false,
    }
  }

  /// The antecedent (already consumed) region.
  #[inline]
  pub fn antecedent(&self) -> &[u8] {
    let range = 0..self.antecedent_end_idx;
    // SAFETY: All methods ensure that `antecedent_end_idx` will never be greater than
    //         the buffer's length
    unsafe { self.buffer.get(range).unwrap_unchecked() }
  }

  /// The end index of the antecedent (already consumed) region.
  #[inline]
  pub const fn antecedent_end_idx(&self) -> usize {
    self.antecedent_end_idx
  }

  /// Capacity Upper Bound
  ///
  /// The maximum buffer's length.
  #[inline]
  pub const fn capacity_ub(&self) -> usize {
    self.capacity_ub
  }

  /// Clears the internal buffer if all fetched data has been fully consumed. In other words,
  /// if [`Self::following`] is empty.
  ///
  /// NO-OP if [`Self::forbid_clear`] is `true`.
  #[inline]
  pub fn clear_if_exhausted(&mut self) {
    if self.current_end_idx == self.buffer.len() {
      self.clear();
    }
  }

  /// The current readable region.
  #[inline]
  pub fn current(&self) -> &[u8] {
    let range = self.antecedent_end_idx..self.current_end_idx;
    // SAFETY: All methods ensure that `antecedent_end_idx` and `current_end_idx`
    //         will never be greater than the buffer's length
    unsafe { self.buffer.get(range).unwrap_unchecked() }
  }

  /// Mutable version of [`Self::current`].
  #[inline]
  pub fn current_mut(&mut self) -> &mut [u8] {
    let range = self.antecedent_end_idx..self.current_end_idx;
    // SAFETY: All methods ensure that `antecedent_end_idx` and `current_end_idx`
    //         will never be greater than the buffer's length
    unsafe { self.buffer.get_mut(range).unwrap_unchecked() }
  }

  /// The end index of the current readable region.
  #[inline]
  pub const fn current_end_idx(&self) -> usize {
    self.current_end_idx
  }

  /// The entire internal buffer as a slice.
  #[inline]
  pub fn filled(&self) -> &[u8] {
    &self.buffer
  }

  /// The filled but unread region.
  #[inline]
  pub fn following(&self) -> &[u8] {
    // SAFETY: All methods ensure that `current_end_idx` will never be greater than the
    //         buffer's length
    unsafe { self.buffer.get(self.current_end_idx..).unwrap_unchecked() }
  }

  /// Whether buffer clearing is currently forbidden.
  ///
  /// Should be used temporally to avoid capacity overflow.
  #[inline]
  pub const fn forbid_clear(&self) -> bool {
    self.forbid_clear
  }

  /// Mutable version of [`Self::forbid_clear`].
  #[inline]
  pub const fn forbid_clear_mut(&mut self) -> &mut bool {
    &mut self.forbid_clear
  }

  /// Reads `LEN` buffer that are intended to form the header of a protocol message.
  ///
  /// Also removes the references that compose the current readable region.
  #[inline]
  pub async fn read_header<SR, const LEN: usize>(
    &mut self,
    stream_reader: &mut SR,
  ) -> crate::Result<StreamReadItem<[u8; LEN]>>
  where
    SR: StreamReader,
  {
    self.manage_capacity(LEN)?;
    let current_end_idx = self.current_end_idx;
    loop {
      let (init, uninit) = self.split_at_spare_mut();
      // SAFETY: All methods ensure that `current_end_idx` will never be greater than the
      //         buffer's length
      let following = unsafe { init.get(current_end_idx..).unwrap_unchecked() };
      if let Some(slice) = following.get(..LEN) {
        let rslt = slice.try_into().unwrap_or([0; LEN]);
        self.remove_current(LEN);
        return Ok(StreamReadItem::from_item(rslt));
      }
      let Some(len) = stream_reader.read(uninit.into()).await?.opt() else {
        cold_path();
        return Ok(StreamReadItem::empty_cold());
      };
      let new_len = init.len().wrapping_add(len.get());
      // SAFETY: `stream_reader.read` just initialized `len` buffer
      unsafe {
        self.buffer.set_len(new_len);
      }
    }
  }

  /// Reads `payload_len` buffer that are intended to form the body of a protocol message. Should
  /// be called after [`Self::read_header`].
  ///
  /// Also creates the references that compose the current readable region.
  #[inline]
  pub async fn read_payload<SR>(
    &mut self,
    payload_len: usize,
    stream_reader: &mut SR,
  ) -> crate::Result<StreamReadItem<()>>
  where
    SR: StreamReader,
  {
    self.manage_capacity(payload_len)?;
    let current_end_idx = self.current_end_idx;
    loop {
      let (init, uninit) = self.split_at_spare_mut();
      let following_len = init.len().wrapping_sub(current_end_idx);
      if following_len >= payload_len {
        self.current_end_idx = current_end_idx.wrapping_add(payload_len);
        return Ok(StreamReadItem::from_item(()));
      }
      let Some(len) = stream_reader.read(uninit.into()).await?.opt() else {
        cold_path();
        return Ok(StreamReadItem::empty_cold());
      };
      let new_len = init.len().wrapping_add(len.get());
      // SAFETY: `stream_reader.read` just initialized `len` buffer
      unsafe {
        self.buffer.set_len(new_len);
      }
    }
  }

  /// `value` can not be lesser than the current capacity upper bound.
  #[inline]
  pub fn set_capacity_ub(&mut self, value: usize) {
    self.capacity_ub = self.capacity_ub.max(value);
  }

  /// Returns vector content as a slice of `T`, along with the remaining spare capacity of the
  /// vector as a slice of `MaybeUninit<T>`.
  #[inline]
  pub fn split_at_spare_mut(&mut self) -> (&mut [u8], &mut [MaybeUninit<u8>]) {
    self.buffer.split_at_spare_mut()
  }

  #[cfg(feature = "postgres")]
  #[inline]
  pub(crate) fn buffer_mut(&mut self) -> &mut Vector<u8> {
    &mut self.buffer
  }

  /// Clears internal state
  #[inline]
  pub(crate) fn clear(&mut self) {
    let Self { antecedent_end_idx, buffer, capacity_ub: _, current_end_idx, forbid_clear } = self;
    if *forbid_clear {
      return;
    }
    *antecedent_end_idx = 0;
    buffer.clear();
    *current_end_idx = 0;
    *forbid_clear = false;
  }

  /// Useful when the actual amount of required buffer is unknown. Always increases current by the
  /// number of filled elements.
  ///
  /// `reserve_len` is only used to create a buffer to allow external reads.
  #[cfg(feature = "web-socket-handshake")]
  pub(crate) async fn read_arbitrary<SR>(
    &mut self,
    reserve_len: usize,
    stream_reader: &mut SR,
  ) -> crate::Result<StreamReadItem<core::num::NonZeroUsize>>
  where
    SR: StreamReader,
  {
    self.manage_capacity(reserve_len)?;
    let (init, uninit) = self.split_at_spare_mut();
    let Some(len) = stream_reader.read(uninit.into()).await?.opt() else {
      cold_path();
      return Ok(StreamReadItem::empty_cold());
    };
    let new_len = init.len().wrapping_add(len.get());
    // SAFETY: `stream_reader.read` just initialized `len` buffer
    unsafe {
      self.buffer.set_len(new_len);
    }
    self.current_end_idx = new_len;
    Ok(StreamReadItem::from_item(len))
  }

  /// Both indices will be capped to avoid data corruption.
  #[cfg(feature = "web-socket-handshake")]
  pub(crate) fn set_indices(&mut self, antecedent_end_idx: usize, current_end_idx: usize) {
    self.current_end_idx = current_end_idx.min(self.buffer.len());
    self.antecedent_end_idx = antecedent_end_idx.min(self.current_end_idx);
  }

  #[cfg(any(feature = "postgres", feature = "web-socket-handshake"))]
  pub(crate) fn suffix_pusher(&mut self) -> crate::collections::SuffixPusherVectorMut<'_, u8> {
    crate::collections::SuffixPusherVectorMut::from(&mut self.buffer)
  }

  /// `additional` refers `following`, `trailing` and unreserved memory. In other words, fetched
  /// but unread buffer, uninitialized buffer and unallocated buffer.
  ///
  /// In a partially filled buffer, if `additional` is greater than `current_end_idx - capacity_ub`,
  /// then everything before `current_end_idx` should be left shifted only if `additional` <= `capacity_ub`.
  #[inline]
  fn manage_capacity(&mut self, additional: usize) -> crate::Result<()> {
    let buffer_len = self.buffer.len();
    let capacity_ub = self.capacity_ub;
    let current_end_idx = self.current_end_idx;
    let following_len = buffer_len.wrapping_sub(current_end_idx);
    if additional > capacity_ub {
      cold_path();
      return Err(BufStreamReaderError::CapacityOverflow.into());
    }
    if following_len == 0 && !self.forbid_clear {
      self.clear();
      self.buffer.reserve(additional)?;
      return Ok(());
    }
    let required_capacity = current_end_idx.wrapping_add(additional);
    if self.buffer.capacity() >= required_capacity {
      return Ok(());
    }
    if required_capacity <= capacity_ub {
      self.buffer.reserve(required_capacity.wrapping_sub(buffer_len))?;
      return Ok(());
    }
    cold_path();
    if self.forbid_clear {
      return Err(BufStreamReaderError::ForbiddenClear.into());
    }
    self.buffer.copy_within(current_end_idx.., 0);
    self.buffer.truncate(following_len);
    self.buffer.reserve(additional.wrapping_sub(following_len))?;
    self.antecedent_end_idx = 0;
    self.current_end_idx = 0;
    Ok(())
  }

  #[inline]
  fn remove_current(&mut self, offset: usize) {
    let idx = self.current_end_idx.wrapping_add(offset);
    self.antecedent_end_idx = idx;
    self.current_end_idx = idx;
  }
}

impl Lease<BufStreamReader> for BufStreamReader {
  #[inline]
  fn lease(&self) -> &BufStreamReader {
    self
  }
}

impl LeaseMut<BufStreamReader> for BufStreamReader {
  #[inline]
  fn lease_mut(&mut self) -> &mut BufStreamReader {
    self
  }
}

impl Debug for BufStreamReader {
  #[inline]
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("NetReadBuffer").finish()
  }
}

impl Default for BufStreamReader {
  #[inline]
  fn default() -> Self {
    Self::new()
  }
}

/// A remote actor can always stop the exchange of data to allow a graceful stop, moreover, in
/// `WTX` a closed stream is not considered an error.
///
/// That is why all places where data is externally fetched a [`StreamReadItem`] is returned: to
/// signal if the current connection is still alive.
#[derive(Debug, Default)]
#[must_use = "You must handle potential connection closures. Use `.is_closed()`, `.opt()` or `.rslt()` to gracefully stop or abort execution."]
pub struct StreamReadItem<T>(Option<T>);

impl<T> StreamReadItem<T> {
  /// New empty instance.
  #[inline]
  pub const fn empty() -> Self {
    Self(None)
  }

  /// New empty instance that hints a cold path.
  #[cold]
  #[inline]
  pub const fn empty_cold() -> Self {
    Self(None)
  }

  /// New instance from an existent element.
  #[inline]
  pub const fn from_item(item: T) -> Self {
    Self(Some(item))
  }

  /// New instance from an optional element.
  #[inline]
  pub const fn from_opt(item: Option<T>) -> Self {
    Self(item)
  }

  /// If the current connection is still active.
  #[inline]
  pub const fn is_alive(&self) -> bool {
    self.0.is_some()
  }

  /// If the current connection is closed
  #[inline]
  pub const fn is_closed(&self) -> bool {
    self.0.is_none()
  }

  /// Converts itself into an option
  ///
  /// Returns `None` if the connection is closed.
  #[inline]
  pub fn opt(self) -> Option<T> {
    self.0
  }

  /// Converts itself into a result.
  ///
  /// Returns `Err` if the connection is closed.
  #[inline]
  pub fn rslt(self) -> crate::Result<T> {
    match self.0 {
      None => Err(BufStreamReaderError::ClosedConnection.into()),
      Some(elem) => Ok(elem),
    }
  }
}

impl<T, E> From<Result<T, E>> for StreamReadItem<T> {
  #[inline]
  fn from(value: Result<T, E>) -> Self {
    Self(value.ok())
  }
}

impl<T> From<Option<T>> for StreamReadItem<T> {
  #[inline]
  fn from(value: Option<T>) -> Self {
    Self(value)
  }
}

impl<T> From<T> for StreamReadItem<T> {
  #[inline]
  fn from(value: T) -> Self {
    Self(Some(value))
  }
}

#[cfg(test)]
mod tests {
  use crate::stream::{BufStreamReader, BytesStream, StreamWriter};

  #[wtx::test]
  async fn read_header_and_payload() {
    let mut stream = BytesStream::default();
    stream.write_all(&[0, 2, 1, 2]).await.unwrap();
    let mut nrb = BufStreamReader::default();
    let header = nrb.read_header::<_, 2>(&mut stream).await.unwrap().rslt().unwrap();
    let len = u16::from_be_bytes(header);
    nrb.read_payload(len.into(), &mut stream).await.unwrap().rslt().unwrap();
    assert_eq!(nrb.current_mut(), [1, 2]);
  }
}
