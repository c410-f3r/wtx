//! Collection types
//!
//! Hash-based collections are just re-exports of the hashbrown crate.

#[macro_use]
mod macros;

mod array_str;
mod array_vector;
mod array_vector_copy;
mod array_vector_inner;
mod array_wrapper;
mod auto_clear;
mod blocks_deque;
mod capacity_upper_bound;
mod clear;
mod collections_error;
mod csr;
mod deque;
mod dyn_str;
mod dyn_vector;
mod expansion_ty;
mod fixed_str;
mod linear_storage;
mod maybe_uninit_slice;
mod misc;
mod short_box_slice;
mod short_box_str;
mod short_slice;
mod short_str;
mod single_type_storage;
mod str;
mod suffix_guard;
mod truncate;
mod try_extend;
mod vector;

pub use array_str::{ArrayStr, ArrayStrError, ArrayStrU8, ArrayStrU16, ArrayStrU32, ArrayStrUsize};
pub use array_vector::{
  ArrayVector, ArrayVectorError, ArrayVectorU8, ArrayVectorU16, ArrayVectorU32, ArrayVectorUsize,
};
pub use array_vector_copy::ArrayVectorCopy;
pub use array_vector_inner::ArrayIntoIter;
pub use array_wrapper::ArrayWrapper;
pub use auto_clear::AutoClear;
pub use blocks_deque::{
  Block, BlocksDeque, BlocksDequeError, BlocksDequeU8, BlocksDequeU16, BlocksDequeU32,
  BlocksDequeUsize,
};
pub use capacity_upper_bound::CapacityUpperBound;
pub use clear::Clear;
pub use collections_error::CollectionsError;
pub use csr::{Csr, CsrError, CsrLineBuilder, CsrRow, CsrRowIter, CsrU8, CsrU16, CsrU32, CsrUsize};
pub use deque::{Deque, DequeU8, DequeU16, DequeU32, DequeUsize, DequeueError};
pub use dyn_str::{DynStr, DynStrU7, DynStrU15, DynStrU31};
pub use dyn_vector::{DynVector, DynVectorError, DynVectorU7, DynVectorU15, DynVectorU31};
pub use expansion_ty::ExpansionTy;
pub use fixed_str::{FixedStr, FixedStrError};
#[cfg(feature = "hashbrown")]
pub use hashbrown::{HashMap, HashSet, hash_map, hash_set};
#[cfg(target_pointer_width = "64")]
pub use linear_storage::linear_storage_len::U63Len;
pub use linear_storage::linear_storage_len::{
  LinearStorageLen, SparseLinearStorageLen, U7Len, U15Len, U31Len,
};
pub use maybe_uninit_slice::MaybeUninitSlice;
pub use misc::{backward_deque_idx, concat_slices};
pub use short_box_slice::{ShortBoxSlice, ShortBoxSliceU8, ShortBoxSliceU16};
pub use short_box_str::{ShortBoxStr, ShortBoxStrU8, ShortBoxStrU16};
pub use short_slice::{ShortSlice, ShortSliceU8, ShortSliceU16};
pub use short_str::{ShortStr, ShortStrU8, ShortStrU16};
pub use single_type_storage::SingleTypeStorage;
pub use str::{Str, StrU8, StrU16, StrU32, StrUsize};
pub use suffix_guard::{SensitiveSuffixGuard, SuffixGuard, SuffixGuardVectorMut};
pub use truncate::Truncate;
pub use try_extend::TryExtend;
pub use vector::{Vector, VectorError, VectorU8, VectorU16, VectorU32, VectorUsize};
