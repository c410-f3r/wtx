//! Collection types

#[macro_use]
mod macros;

mod array_string;
mod array_vector;
mod array_wrapper;
mod blocks_deque;
mod clear;
mod deque;
mod expansion_ty;
mod linear_storage;
mod misc;
mod try_extend;
mod vector;

pub use array_string::{
  ArrayString, ArrayStringError, ArrayStringU8, ArrayStringU16, ArrayStringU32, ArrayStringUsize,
};
pub use array_vector::{
  ArrayIntoIter, ArrayVector, ArrayVectorError, ArrayVectorU8, ArrayVectorU16, ArrayVectorU32,
  ArrayVectorUsize,
};
pub use array_wrapper::ArrayWrapper;
pub use blocks_deque::{Block, BlocksDeque, BlocksDequeBuilder, BlocksDequeError};
pub use clear::Clear;
pub use deque::{Deque, DequeueError};
pub use expansion_ty::ExpansionTy;
pub use linear_storage::linear_storage_len::LinearStorageLen;
pub use misc::backward_deque_idx;
pub use try_extend::TryExtend;
pub use vector::{Vector, VectorError};
