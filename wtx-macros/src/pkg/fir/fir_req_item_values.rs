create_fir_custom_item_values!(
  "Expected data request that will be sent to the server.",
  FirReqItemValues,
  freqdiv_fields_attrs,
  freqdiv_ident,
  freqdiv_item,
  freqdiv_params,
  freqdiv_ty,
  freqdiv_where_predicates,
  |this| {
    if !this.freqdiv_ident.to_string().ends_with("Req") {
      return Err(crate::Error::BadReq(this.freqdiv_ident.span()));
    }
    Ok(())
  },
);
