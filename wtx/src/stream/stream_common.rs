/// Common stream properties for both readers and writers
pub trait StreamCommon {
  /// Is a Kernel TLS implementation?
  const IS_KTLS: bool = false;
}

impl<T> StreamCommon for &mut T where T: StreamCommon {}

impl StreamCommon for () {}
