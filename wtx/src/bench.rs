use alloc::vec::Vec;

pub(crate) fn _data(len: usize) -> Vec<u8> {
  (0..len)
    .map(|el| {
      let n = el % usize::try_from(u8::MAX).unwrap();
      n.try_into().unwrap()
    })
    .collect()
}
