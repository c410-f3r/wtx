use syn::{Data, DeriveInput, Fields};

pub(crate) fn from_vars(item: proc_macro::TokenStream) -> crate::Result<proc_macro::TokenStream> {
  let input = syn::parse::<DeriveInput>(item)?;
  let struct_name = &input.ident;
  let fields = match &input.data {
    Data::Struct(data) => match &data.fields {
      Fields::Named(fields) => &fields.named,
      _ => return Err(crate::Error::InvalidStruct),
    },
    _ => return Err(crate::Error::InvalidStruct),
  };
  let mut field_names = Vec::new();
  for el in fields {
    field_names.push(el.ident.as_ref().ok_or(crate::Error::InvalidStruct)?);
  }
  let field_strings: Vec<_> = field_names.iter().map(ToString::to_string).collect();
  let field_vars: Vec<_> = field_names.iter().map(|el| quote::format_ident!("__{el}")).collect();
  let var_declarations = field_vars.iter().map(|var| {
    quote::quote! { let mut #var = None; }
  });
  let match_arms = field_strings.iter().zip(field_vars.iter()).map(|(ident, var)| {
    let upper = ident.to_uppercase();
    quote::quote! { #upper => #var = Some(value), }
  });
  let field_assignments = field_names.iter().zip(field_vars.iter()).zip(field_strings.iter()).map(
    |((name, var), name_str)| {
      quote::quote! { #name: #var.ok_or_else(|| wtx::Error::MissingVar(#name_str.into()))? }
    },
  );
  let expanded = quote::quote! {
    impl wtx::misc::FromVars for #struct_name {
      fn from_vars(vars: impl IntoIterator<Item = (String, String)>) -> wtx::Result<Self> {
        #(#var_declarations)*
        for (key, value) in vars {
          match key.as_str() {
            #(#match_arms)*
            _ => {}
          }
        }
        Ok(Self { #(#field_assignments),* })
      }
    }
  };
  Ok(proc_macro::TokenStream::from(expanded))
}
