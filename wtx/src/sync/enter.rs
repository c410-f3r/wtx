use core::{
  cell::Cell,
  fmt::{Debug, Formatter},
};

std::thread_local!(static ENTERED: Cell<bool> = const { Cell::new(false) });

/// An executor context.
pub struct Enter {}

impl Enter {
  /// Marks the current thread to avoid nested usage.
  #[inline]
  pub fn new() -> Option<Self> {
    ENTERED.with(|cell| {
      if cell.get() {
        None
      } else {
        cell.set(true);
        Some(Enter {})
      }
    })
  }
}

impl Debug for Enter {
  #[inline]
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Enter").finish()
  }
}

impl Drop for Enter {
  #[inline]
  fn drop(&mut self) {
    ENTERED.with(|cell| {
      cell.set(false);
    });
  }
}
