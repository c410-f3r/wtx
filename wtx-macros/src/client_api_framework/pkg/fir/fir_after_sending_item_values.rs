create_fir_hook_item_values!(
  FirAfterSendingItemValues,
  fasiv_fn_call_idents,
  fasiv_item,
  |arg| {
    Some(match arg {
      "api" => quote::quote!(_api.lease_mut()),
      "params" => quote::quote!(&mut self.params),
      "res_params" => quote::quote!(_ext_res_params),
      _ => return None,
    })
  },
  BadAfterSending,
);
