use crate::misc::{
  BlocksDeque, BlocksDequeError, BufferMode, collections::blocks_deque::metadata::Metadata,
};
use core::slice;

/// Allows the construction of a single block through the insertion of indivial elements.
#[derive(Debug)]
pub struct BlocksDequeBuilder<'db, D, M, const IS_BACK: bool> {
  bd: &'db mut BlocksDeque<D, M>,
  inserted: usize,
  was_built: bool,
}

impl<'db, D, M, const IS_BACK: bool> BlocksDequeBuilder<'db, D, M, IS_BACK> {
  #[inline]
  pub(crate) fn new(bd: &'db mut BlocksDeque<D, M>) -> Self {
    Self { bd, inserted: 0, was_built: false }
  }

  /// Finishes the construction of the block
  #[inline]
  pub fn build(mut self, misc: M) -> Result<(), BlocksDequeError> {
    self.was_built = true;
    let rslt = if IS_BACK {
      let begin = self.bd.data.tail().wrapping_sub(self.inserted);
      let metadata = Metadata { begin, len: self.inserted, misc };
      self.bd.metadata.push_back(metadata)
    } else {
      let metadata = Metadata { begin: self.bd.data.head(), len: self.inserted, misc };
      self.bd.metadata.push_front(metadata)
    };
    rslt.map_err(|_err| BlocksDequeError::PushOverflow)
  }

  /// Appends or prepends elements so that the current length is equal to `bp`.
  #[inline]
  pub fn expand(&mut self, bm: BufferMode, value: D) -> Result<&mut Self, BlocksDequeError>
  where
    D: Clone,
  {
    let additional = if IS_BACK {
      let rslt = self.bd.data.expand_back(bm, value);
      rslt.map_err(|_err| BlocksDequeError::PushOverflow)?
    } else {
      let rslt = self.bd.data.expand_front(bm, value);
      let (additional, head_shift) = rslt.map_err(|_err| BlocksDequeError::PushOverflow)?;
      self.bd.adjust_metadata(head_shift, 1);
      additional
    };
    self.inserted = self.inserted.wrapping_add(additional);
    Ok(self)
  }

  /// The elements inserted so far by this builder
  #[inline]
  pub fn inserted_elements(&mut self) -> &mut [D] {
    let ptr = self.bd.data.as_ptr_mut();
    let shifted_ptr = if IS_BACK {
      let begin = self.bd.data.tail().wrapping_sub(self.inserted);
      // SAFETY: We are in a "back-only" environment so the tail index will never be less than
      // the number of inserted elements
      unsafe { ptr.add(begin) }
    } else {
      let begin = self.bd.data.head();
      // SAFETY: We are in a "front-only" environment so there will always be `inserted` elements
      // after the starting head.
      unsafe { ptr.add(begin) }
    };
    // SAFETY: the above checks ensure valid memory
    unsafe { slice::from_raw_parts_mut(shifted_ptr, self.inserted) }
  }
}

// The `was_built` parameter is used to enforce a valid instance in case of an external error.
//
// ```
// builder.expand(...);
// some_fallible_operation(...)?;
// builder.build();
// ```
impl<D, M, const IS_BACK: bool> Drop for BlocksDequeBuilder<'_, D, M, IS_BACK> {
  #[inline]
  fn drop(&mut self) {
    if !self.was_built {
      let previous_len = self.bd.elements_len().wrapping_sub(self.inserted);
      if IS_BACK {
        self.bd.data.truncate_back(previous_len);
      } else {
        self.bd.data.truncate_front(previous_len);
      }
    }
  }
}
