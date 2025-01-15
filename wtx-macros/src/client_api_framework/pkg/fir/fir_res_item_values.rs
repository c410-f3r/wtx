create_fir_custom_item_values!(
  "Expected data response returned by the server.",
  FirResItemValues,
  fresdiv_fields_attrs,
  fresdiv_ident,
  fresdiv_item,
  fresdiv_params,
  fresdiv_ty,
  fresdiv_where_predicates,
  |this| {
    if !this.fresdiv_ident.to_string().ends_with("Res") {
      return Err(crate::Error::BadRes(this.fresdiv_ident.span()));
    }
    Ok(())
  },
);
