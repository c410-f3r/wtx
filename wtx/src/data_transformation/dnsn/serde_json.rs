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

#[cfg(test)]
mod tests {
  _create_dnsn_test!(
    json,
    (VerbatimRequest, VerbatimResponse),
    SerdeJson as SerdeJson,
    (r#"{"foo":"foo"}"#, r#"{"bar":"bar"}"#.into()),
    (
      VerbatimRequest { data: Foo { foo: "foo" } },
      VerbatimResponse { data: Bar { bar: "bar".into() } }
    ),
  );

  _create_dnsn_test!(
    json_rpc,
    (JsonRpcRequest, JsonRpcResponse),
    SerdeJson as SerdeJson,
    (
      r#"{"jsonrpc":"2.0","method":"method","params":{"foo":"foo"},"id":0}"#,
      r#"{"jsonrpc":"2.0","method":"method","result":{"bar":"bar"},"id":0}"#.into()
    ),
    (
      JsonRpcRequest { id: 0, method: "method", params: Foo { foo: "foo" } },
      JsonRpcResponse {
        id: 0,
        method: Some("method".into()),
        result: Ok(Bar { bar: "bar".into() })
      }
    ),
  );
}
