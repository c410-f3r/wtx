/// Marker that imposes different bounds depending on the selected feature.
#[cfg(not(feature = "async-send"))]
pub trait AsyncBounds {}

#[cfg(not(feature = "async-send"))]
impl<T> AsyncBounds for T where T: ?Sized {}

/// Marker that imposes different bounds depending on the selected feature.
#[cfg(feature = "async-send")]
pub trait AsyncBounds: Send {}

#[cfg(feature = "async-send")]
impl<T> AsyncBounds for T where T: Send + ?Sized {}
