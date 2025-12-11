use syn::{Attribute, Data, DeriveInput, Fields, Type};

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

  let mut field_infos = Vec::new();
  for field in fields {
    field_infos.push(FieldInfo {
      custom_fn: custom_function(&field.attrs)?,
      ident: field.ident.as_ref().ok_or(crate::Error::InvalidStruct)?,
      is_optional: is_option_type(&field.ty),
    });
  }

  let field_strings: Vec<_> = field_infos.iter().map(|f| f.ident.to_string()).collect();
  let field_vars: Vec<_> =
    field_infos.iter().map(|field_info| quote::format_ident!("_{}", field_info.ident)).collect();
  let var_declarations = field_vars.iter().map(|var| {
    quote::quote! { let mut #var = None; }
  });

  let match_arms =
    field_strings.iter().zip(&field_vars).zip(&field_infos).map(|((name_str, var), field_info)| {
      let upper = name_str.to_uppercase();
      if let Some(custom_fn) = &field_info.custom_fn {
        quote::quote! {
          #upper => #var = Some(#custom_fn(value)?),
        }
      } else {
        quote::quote! {
          #upper => #var = Some(value),
        }
      }
    });

  let field_assignments =
    field_infos.iter().zip(&field_vars).zip(&field_strings).map(|((info, var), name_str)| {
      let name = info.ident;
      if info.is_optional {
        quote::quote! { #name: #var }
      } else {
        quote::quote! {
          #name: #var.ok_or_else(|| wtx::Error::MissingVar(#name_str.into()))?
        }
      }
    });

  let string_path = if cfg!(feature = "std") {
    quote::quote!(std::string::)
  } else {
    quote::quote!(alloc::string::)
  };

  let expanded = quote::quote! {
    impl wtx::misc::FromVars for #struct_name {
      fn from_vars(vars: impl IntoIterator<Item = (#string_path String, #string_path String)>) -> wtx::Result<Self> {
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

fn custom_function(attrs: &[Attribute]) -> crate::Result<Option<syn::Path>> {
  for attr in attrs {
    if attr.path().is_ident("from_vars") {
      return Ok(Some(attr.parse_args()?));
    }
  }
  Ok(None)
}

fn is_option_type(ty: &Type) -> bool {
  if let Type::Path(type_path) = ty
    && let Some(path_segment) = type_path.path.segments.last()
    && path_segment.ident == "Option"
  {
    true
  } else {
    false
  }
}

struct FieldInfo<'ident> {
  custom_fn: Option<syn::Path>,
  ident: &'ident syn::Ident,
  is_optional: bool,
}
