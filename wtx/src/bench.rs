use crate::collections::VectorUsize;

pub(crate) fn _data(len: usize) -> VectorUsize<u8> {
  VectorUsize::from_iterator((0..len).map(|el| {
    let n = el % usize::from(u8::MAX);
    n.try_into().unwrap()
  }))
  .unwrap()
}
