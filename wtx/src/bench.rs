use crate::collection::Vector;

pub(crate) fn _data(len: usize) -> Vector<u8> {
  Vector::from_iterator((0..len).map(|el| {
    let n = el % usize::from(u8::MAX);
    n.try_into().unwrap()
  }))
  .unwrap()
}
