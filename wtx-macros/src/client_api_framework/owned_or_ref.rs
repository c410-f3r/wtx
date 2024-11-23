use quote::ToTokens;

#[derive(Debug)]
pub(crate) enum OwnedOrRef<'any, T> {
  Owned(T),
  Ref(&'any T),
}

impl<T> ToTokens for OwnedOrRef<'_, T>
where
  T: ToTokens,
{
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    match self {
      OwnedOrRef::Owned(elem) => elem.to_tokens(tokens),
      OwnedOrRef::Ref(elem) => elem.to_tokens(tokens),
    }
  }
}
