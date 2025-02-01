create_fir_hook_item_values!(
  FirBeforeSendingItemValues,
  fbsiv_fn_call_idents,
  fbsiv_item,
  _fasiv_params,
  fbsiv_where_predicates,
  |arg| {
    Some(match arg {
      "api" => quote::quote!(_api.lease_mut()),
      "bytes" => quote::quote!(_bytes),
      "drsr" => quote::quote!(_drsr),
      "params" => quote::quote!(&mut self.params),
      "trans" => quote::quote!(_trans),
      "trans_params" => quote::quote!(_trans_params),
      _ => return None,
    })
  },
  BadBeforeSending,
);
