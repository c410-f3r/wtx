use crate::client_api_framework::pkg::keywords;
use proc_macro2::Ident;
use syn::{
  LitStr, Token,
  parse::{Parse, ParseStream},
};

#[derive(Debug)]
pub(crate) struct FirCustomFieldFieldAttr {
  pub(crate) name: Ident,
}

impl Parse for FirCustomFieldFieldAttr {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    let mut name = None;
    while !input.is_empty() {
      let lookahead = input.lookahead1();
      if lookahead.peek(keywords::name) {
        let _ = input.parse::<keywords::name>()?;
        let _ = input.parse::<Token![=]>()?;
        let s = input.parse::<LitStr>()?;
        name = Some(Ident::new(&s.value(), s.span()));
      } else {
        return Err(lookahead.error());
      }
    }
    Ok(Self { name: name.ok_or_else(|| crate::Error::BadField(input.span()))? })
  }
}
