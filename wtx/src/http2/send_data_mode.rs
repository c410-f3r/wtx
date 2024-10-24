/// Dictates how HTTP/2 data frames should be delivered.
#[derive(Debug)]
pub struct SendDataMode<B, const IS_SCATTERED: bool> {
  bytes: B,
}

impl<'bytes, B, const IS_SCATTERED: bool> SendDataMode<B, IS_SCATTERED>
where
  B: SendDataModeBytes<'bytes, IS_SCATTERED>,
{
  #[inline]
  const fn new(bytes: B) -> Self {
    Self { bytes }
  }

  #[inline]
  pub(crate) fn concat<'first, 'rslt, 'this>(&'this self, first: &'first [u8]) -> [&'rslt [u8]; 3]
  where
    'first: 'rslt,
    'this: 'rslt,
  {
    self.bytes.concat(first)
  }

  #[inline]
  pub(crate) fn first_mut(&mut self) -> &mut &'bytes [u8] {
    self.bytes.first_mut()
  }

  #[inline]
  pub(crate) fn len(&self) -> usize {
    self.bytes.len()
  }
}

impl<'bytes> SendDataMode<[&'bytes [u8]; 1], true> {
  /// `bytes` is broken down into smaller pieces until everything is submitted. This process
  /// depends on the maximum payload length as well as the current window size.
  #[inline]
  pub const fn scattered_data_frames(bytes: &'bytes [u8]) -> Self {
    Self::new([bytes])
  }
}

impl<'bytes, B> SendDataMode<B, false>
where
  B: SendDataModeBytes<'bytes, false>,
{
  /// The concatenation of `bytes` is delivered in a single data frame.
  ///
  /// If the maximum frame payload or the current window size is less than the total size, then the
  /// submission will return an error.
  #[inline]
  pub const fn single_data_frame(array: B) -> Self {
    Self::new(array)
  }
}

/// Array implementations for [`SendDataMode`].
pub trait SendDataModeBytes<'bytes, const IS_SCATTERED: bool> {
  /// Returns an array of concatenated references with an initial `first` slice.
  fn concat<'first, 'rslt, 'this>(&'this self, first: &'first [u8]) -> [&'rslt [u8]; 3]
  where
    'first: 'rslt,
    'this: 'rslt;

  /// First inner mutable slice.
  fn first_mut(&mut self) -> &mut &'bytes [u8];

  /// Sum of all inner slice lengths.
  fn len(&self) -> usize;
}

impl<'bytes, const IS_SCATTERED: bool> SendDataModeBytes<'bytes, IS_SCATTERED>
  for [&'bytes [u8]; 1]
{
  #[inline]
  fn concat<'first, 'rslt, 'this>(&'this self, first: &'first [u8]) -> [&'rslt [u8]; 3]
  where
    'first: 'rslt,
    'this: 'rslt,
  {
    [first, &self[0], &[]]
  }

  #[inline]
  fn first_mut(&mut self) -> &mut &'bytes [u8] {
    &mut self[0]
  }

  #[inline]
  fn len(&self) -> usize {
    let mut len = 0usize;
    for elem in self {
      len = len.wrapping_add(elem.len());
    }
    len
  }
}

impl<'bytes> SendDataModeBytes<'bytes, false> for [&'bytes [u8]; 2] {
  #[inline]
  fn concat<'first, 'rslt, 'this>(&'this self, first: &'first [u8]) -> [&'rslt [u8]; 3]
  where
    'first: 'rslt,
    'this: 'rslt,
  {
    [first, &self[0], &self[1]]
  }

  #[inline]
  fn first_mut(&mut self) -> &mut &'bytes [u8] {
    &mut self[0]
  }

  #[inline]
  fn len(&self) -> usize {
    let mut len = 0usize;
    for elem in self {
      len = len.wrapping_add(elem.len());
    }
    len
  }
}
