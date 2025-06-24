use crate::misc::{Lease, LeaseMut};

/// Type that indicates the usage of the `serde_json` dependency.
#[derive(Debug)]
pub struct SerdeJson;

impl Lease<SerdeJson> for SerdeJson {
  #[inline]
  fn lease(&self) -> &SerdeJson {
    self
  }
}

impl LeaseMut<SerdeJson> for SerdeJson {
  #[inline]
  fn lease_mut(&mut self) -> &mut SerdeJson {
    self
  }
}

_impl_se_collections!(
  (SerdeJson, serde::Serialize),
  array: |this, bytes, _drsr| { serde_json::to_writer(bytes, &this[..])?; }
  arrayvector: |this, bytes, _drsr| { serde_json::to_writer(bytes, this)?; }
  slice_ref: |this, bytes, _drsr| { serde_json::to_writer(bytes, this)?; }
  vec: |this, bytes, _drsr| { serde_json::to_writer(bytes, this)?; }
);

#[cfg(all(feature = "client-api-framework", test))]
mod tests {
  _create_dnsn_test!(
    json,
    (VerbatimEncoder, VerbatimDecoder),
    SerdeJson as SerdeJson,
    (r#"{"foo":"foo"}"#, r#"{"bar":"bar"}"#.into()),
    (
      VerbatimEncoder { data: _Foo { foo: "foo" } },
      VerbatimDecoder { data: _Bar { bar: "bar".into() } }
    ),
  );

  _create_dnsn_test!(
    json_rpc,
    (JsonRpcEncoder, JsonRpcDecoder),
    SerdeJson as SerdeJson,
    (
      r#"{"jsonrpc":"2.0","method":"method","params":{"foo":"foo"},"id":0}"#,
      r#"{"jsonrpc":"2.0","method":"method","result":{"bar":"bar"},"id":0}"#.into()
    ),
    (
      JsonRpcEncoder { id: 0, method: "method", params: _Foo { foo: "foo" } },
      JsonRpcDecoder {
        id: 0,
        method: Some("method".into()),
        result: Ok(_Bar { bar: "bar".into() })
      }
    ),
  );
}
