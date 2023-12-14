create_fir_hook_item_values!(
  FirAfterSendingItemValues,
  fasiv_fn_call_idents,
  fasiv_item,
  "after_sending",
  |arg| {
    Some(match arg {
      "api" => quote::quote!(_api),
      "params" => quote::quote!(&mut self.params),
      "res_params" => quote::quote!(_ext_res_params),
      _ => return None,
    })
  },
  BadAfterSending,
);
