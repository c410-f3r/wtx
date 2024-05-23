/// Type that indicates the usage of the `serde-xml-rs` dependency.
#[derive(Debug)]
pub struct SerdeYaml;

_impl_se_collections!(
  for SerdeYaml => serde::Serialize;

  array: |this, bytes, _drsr| { serde_yaml::to_writer(bytes, &this[..])?; }
  arrayvector: |this, bytes, _drsr| { serde_yaml::to_writer(bytes, this)?; }
  slice_ref: |this, bytes, _drsr| { serde_yaml::to_writer(bytes, this)?; }
  vec: |this, bytes, _drsr| { serde_yaml::to_writer(bytes, this)?; }
);

#[cfg(test)]
mod tests {
  _create_dnsn_test!(
    yaml,
    (YamlRequest, YamlResponse),
    SerdeYaml as SerdeYaml,
    ("foo: foo\n".into(), r#"bar: bar"#.into()),
    (YamlRequest { data: Foo { foo: "foo" } }, YamlResponse { data: Bar { bar: "bar".into() } }),
  );
}
