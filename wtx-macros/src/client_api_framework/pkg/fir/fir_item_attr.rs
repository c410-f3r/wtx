use crate::client_api_framework::pkg::keywords;
use proc_macro2::Span;
use syn::{
  parse::{Parse, ParseBuffer, ParseStream},
  spanned::Spanned,
};

#[derive(Debug)]
pub(crate) enum FirItemAttrTy {
  AfterSending,
  Aux,
  BeforeSending,
  Params,
  Req,
  Res,
}

#[derive(Debug)]
pub(crate) struct FirItemAttr {
  pub(crate) span: Span,
  pub(crate) ty: FirItemAttrTy,
}

impl FirItemAttr {
  pub(crate) fn new(span: Span, ty: FirItemAttrTy) -> Self {
    Self { span, ty }
  }
}

impl Parse for FirItemAttr {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    fn inner_attr<T>(
      content: ParseBuffer<'_>,
      cb: impl FnOnce(ParseBuffer<'_>) -> crate::Result<FirItemAttrTy>,
    ) -> syn::Result<FirItemAttr>
    where
      T: Parse + Spanned,
    {
      Ok(FirItemAttr::new(content.parse::<T>()?.span(), cb(content)?))
    }
    let _ = input.parse::<syn::Token![#]>()?;
    let content;
    syn::bracketed!(content in input);
    let _ = content.parse::<keywords::pkg>()?;
    let _ = content.parse::<syn::Token![::]>()?;
    let lookahead = content.lookahead1();
    Ok(if lookahead.peek(keywords::after_sending) {
      inner_attr::<keywords::after_sending>(content, |_| Ok(FirItemAttrTy::AfterSending))?
    } else if lookahead.peek(keywords::aux) {
      inner_attr::<keywords::aux>(content, |_| Ok(FirItemAttrTy::Aux))?
    } else if lookahead.peek(keywords::before_sending) {
      inner_attr::<keywords::before_sending>(content, |_| Ok(FirItemAttrTy::BeforeSending))?
    } else if lookahead.peek(keywords::params) {
      inner_attr::<keywords::params>(content, |_| Ok(FirItemAttrTy::Params))?
    } else if lookahead.peek(keywords::req_data) {
      inner_attr::<keywords::req_data>(content, |_| Ok(FirItemAttrTy::Req))?
    } else if lookahead.peek(keywords::res_data) {
      inner_attr::<keywords::res_data>(content, |_| Ok(FirItemAttrTy::Res))?
    } else {
      return Err(lookahead.error());
    })
  }
}
