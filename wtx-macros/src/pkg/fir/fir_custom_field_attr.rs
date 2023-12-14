use crate::pkg::{fir::fir_custom_field_field_attr::FirCustomFieldFieldAttr, keywords};
use syn::{
  parse::{Parse, ParseStream},
  spanned::Spanned,
};

#[derive(Debug)]
pub(crate) enum FirCustomFieldAttr {
  Field(FirCustomFieldFieldAttr),
}

impl Parse for FirCustomFieldAttr {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    let _ = input.parse::<syn::Token![#]>()?;
    let content;
    syn::bracketed!(content in input);
    let endpoint = content.parse::<keywords::pkg>()?;
    let _ = content.parse::<syn::Token![::]>()?;
    let lookahead = content.lookahead1();
    Ok(if lookahead.peek(keywords::field) {
      let _ = content.parse::<keywords::field>()?;
      let content_paren;
      syn::parenthesized!(content_paren in content);
      Self::Field(content_paren.parse::<FirCustomFieldFieldAttr>()?)
    } else {
      return Err(crate::Error::BadField(endpoint.span()).into());
    })
  }
}
