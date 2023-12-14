/// Type that indicates the usage of the `simd-json` dependency.
#[derive(Debug)]
pub struct SimdJson;

#[cfg(test)]
mod tests {
  _create_dnsn_test!(
    json,
    (JsonRequest, JsonResponse),
    SimdJson as SimdJson,
    (r#"{"foo":"foo"}"#.into(), r#"{"bar":"bar"}"#.into()),
    (JsonRequest { data: Foo { foo: "foo" } }, JsonResponse { data: Bar { bar: "bar".into() } }),
  );

  _create_dnsn_test!(
    json_rpc,
    (JsonRpcRequest, JsonRpcResponse),
    SimdJson as SimdJson,
    (
      r#"{"jsonrpc":"2.0","method":"method","params":{"foo":"foo"},"id":0}"#.into(),
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
