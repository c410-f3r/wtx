use core::{mem, slice};

#[derive(Debug)]
#[repr(transparent)]
pub struct Header<'buffer>(httparse::Header<'buffer>);

impl<'buffer> Header<'buffer> {
  pub(crate) const EMPTY: Header<'static> = Header(httparse::EMPTY_HEADER);

  pub(crate) fn new(name: &'buffer str, value: &'buffer [u8]) -> Self {
    Self(httparse::Header { name, value })
  }

  pub(crate) fn name(&self) -> &str {
    self.0.name
  }

  pub(crate) fn value(&self) -> &[u8] {
    self.0.value
  }
}

pub(crate) struct HeaderSlice<T>(pub(crate) T);

impl<'buffer, 'headers> From<&'headers mut [Header<'buffer>]>
  for HeaderSlice<&'headers mut [httparse::Header<'buffer>]>
{
  #[inline]
  fn from(from: &'headers mut [Header<'buffer>]) -> Self {
    assert!(mem::size_of::<Header>() == mem::size_of::<httparse::Header>());
    assert!(mem::align_of::<Header>() == mem::align_of::<httparse::Header>());
    let len = from.len();
    // SAFETY: `Header` and `httparse::Header` have the same size and alignment due
    // to `#[transparent]`
    Self(unsafe { slice::from_raw_parts_mut(from.as_mut_ptr().cast(), len) })
  }
}

impl<'buffer, 'headers> From<&'headers [httparse::Header<'buffer>]>
  for HeaderSlice<&'headers [Header<'buffer>]>
{
  #[inline]
  fn from(from: &'headers [httparse::Header<'buffer>]) -> Self {
    assert!(mem::size_of::<Header>() == mem::size_of::<httparse::Header>());
    assert!(mem::align_of::<Header>() == mem::align_of::<httparse::Header>());
    let len = from.len();
    // SAFETY: `Header` and `httparse::Header` have the same size and alignment due
    // to `#[transparent]`
    Self(unsafe { slice::from_raw_parts(from.as_ptr().cast(), len) })
  }
}

#[test]
fn does_not_trigger_a_panic() {
  let inner_header0 = httparse::Header { name: "foo", value: &[1, 2, 3] };
  let inner_header1 = httparse::Header { name: "bar", value: &[4, 5, 6] };
  let _headers = HeaderSlice::from(&[inner_header0, inner_header1][..]);
}
