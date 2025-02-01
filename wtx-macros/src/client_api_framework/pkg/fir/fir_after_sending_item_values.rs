create_fir_hook_item_values!(
  FirAfterSendingItemValues,
  fasiv_fn_call_idents,
  fasiv_item,
  _fasiv_params,
  fasiv_where_predicates,
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
  BadAfterSending,
);
