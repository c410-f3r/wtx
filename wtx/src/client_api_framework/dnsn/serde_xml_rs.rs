/// Type that indicates the usage of the `serde-xml-rs` dependency.
#[derive(Debug)]
pub struct SerdeXmlRs;

_impl_se_collections!(
  for SerdeXmlRs => serde::Serialize;

  array: |this, bytes, _drsr| { serde_xml_rs::to_writer(bytes, &&this[..])?; }
  arrayvec: |this, bytes, _drsr| { serde_xml_rs::to_writer(bytes, this)?; }
  slice_ref: |this, bytes, _drsr| { serde_xml_rs::to_writer(bytes, this)?; }
  vec: |this, bytes, _drsr| { serde_xml_rs::to_writer(bytes, this)?; }
);

#[cfg(test)]
mod tests {
  _create_dnsn_test!(
    xml,
    (XmlRequest, XmlResponse),
    SerdeXmlRs as SerdeXmlRs,
    (
      r#"<?xml version="1.0" encoding="UTF-8"?><Foo><foo>foo</foo></Foo>"#.into(),
      r#"<?xml version="1.0" encoding="UTF-8"?><Bar><bar>bar</bar></Bar>"#.into()
    ),
    (XmlRequest { data: Foo { foo: "foo" } }, XmlResponse { data: Bar { bar: "bar".into() } }),
  );
}
