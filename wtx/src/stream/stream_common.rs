/// Common stream properties for both readers and writers
pub trait StreamCommon {}

impl<T> StreamCommon for &mut T where T: StreamCommon {}

impl StreamCommon for () {}
