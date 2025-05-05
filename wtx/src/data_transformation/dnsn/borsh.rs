use crate::misc::{Lease, LeaseMut};

/// Type that indicates the usage of the `borsh` dependency.
#[derive(Debug, Default)]
pub struct Borsh;

impl Lease<Borsh> for Borsh {
  #[inline]
  fn lease(&self) -> &Borsh {
    self
  }
}

impl LeaseMut<Borsh> for Borsh {
  #[inline]
  fn lease_mut(&mut self) -> &mut Borsh {
    self
  }
}

#[cfg(test)]
mod tests {
  _create_dnsn_test!(
    borsh,
    (VerbatimRequest, VerbatimResponse),
    Borsh as Borsh,
    ([3, 0, 0, 0, 102, 111, 111][..].into(), [3, 0, 0, 0, 98, 97, 114][..].into()),
    (
      VerbatimRequest { data: _Foo { foo: "foo" } },
      VerbatimResponse { data: _Bar { bar: "bar".into() } }
    ),
  );
}
