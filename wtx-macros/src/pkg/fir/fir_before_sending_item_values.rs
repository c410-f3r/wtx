create_fir_hook_item_values!(
  FirBeforeSendingItemValues,
  fbsiv_fn_call_idents,
  fbsiv_item,
  "before_sending",
  |arg| {
    Some(match arg {
      "api" => quote::quote!(_api),
      "params" => quote::quote!(&mut self.params),
      "req_bytes" => quote::quote!(_req_bytes),
      "req_params" => quote::quote!(_ext_req_params),
      _ => return None,
    })
  },
  BadBeforeSending,
);
