use crate::misc::Vector;

pub(crate) fn _data(len: usize) -> Vector<u8> {
  Vector::from_iter((0..len).map(|el| {
    let n = el % usize::try_from(u8::MAX).unwrap();
    n.try_into().unwrap()
  }))
  .unwrap()
}
