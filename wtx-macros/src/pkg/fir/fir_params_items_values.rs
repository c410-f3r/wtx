create_fir_custom_item_values!(
  "Custom local parameters used before and after the issuing of a request.",
  FirParamsItemValues,
  fpiv_fields_attrs,
  fpiv_ident,
  fpiv_item,
  fpiv_params,
  fpiv_ty,
  fpiv_where_predicates,
  |this| {
    if !this.fpiv_ident.to_string().ends_with("Params") {
      return Err(crate::Error::BadParams(this.fpiv_ident.span()));
    }
    Ok(())
  },
);
