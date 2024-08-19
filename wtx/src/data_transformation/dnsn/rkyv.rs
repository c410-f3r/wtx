use crate::misc::Vector;
use rkyv::{
  ser::{
    serializers::{
      AlignedSerializer, AllocScratch, CompositeSerializer, FallbackScratch, HeapScratch,
      SharedSerializeMap,
    },
    Serializer,
  },
  AlignedVec, Serialize,
};

pub(crate) type _InnerSerializer<'this> = CompositeSerializer<
  AlignedSerializer<&'this mut AlignedVec>,
  FallbackScratch<&'this mut HeapScratch<4096>, AllocScratch>,
  SharedSerializeMap,
>;

/// Type that indicates the usage of the `rkyv` dependency.
#[derive(Debug, Default)]
pub struct Rkyv(AlignedVec, HeapScratch<4096>);

impl Rkyv {
  pub(crate) fn _serialize<'this, T>(
    &'this mut self,
    bytes: &mut Vector<u8>,
    elem: &T,
  ) -> crate::Result<()>
  where
    T: Serialize<_InnerSerializer<'this>>,
  {
    let mut serializer = _InnerSerializer::new(
      AlignedSerializer::new(&mut self.0),
      FallbackScratch::new(&mut self.1, AllocScratch::default()),
      SharedSerializeMap::default(),
    );
    let _ = serializer.serialize_value(elem)?;
    let aligned_vec = serializer.into_serializer().into_inner();
    bytes.extend_from_slice(aligned_vec)?;
    aligned_vec.clear();
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  _create_dnsn_test!(
    verbatim,
    (VerbatimRequest, VerbatimResponse),
    Rkyv as Rkyv::default(),
    (
      [102, 111, 111, 0, 252, 255, 255, 255, 3, 0, 0, 0][..].into(),
      [98, 97, 114, 0, 0, 0, 0, 3][..].into()
    ),
    (
      VerbatimRequest { data: Foo { foo: "foo" } },
      VerbatimResponse { data: Bar { bar: "bar".into() } }
    ),
  );
}
