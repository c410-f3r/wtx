use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub(crate) fn conn_aux(item: proc_macro::TokenStream) -> crate::Result<proc_macro::TokenStream> {
  let input = syn::parse::<DeriveInput>(item)?;
  let name = input.ident;
  let mut field_names = Vec::new();
  let mut field_tys = Vec::new();
  match input.data {
    Data::Struct(data) => match data.fields {
      Fields::Named(fields) => {
        for elem in fields.named {
          field_names.push(elem.ident);
          field_tys.push(elem.ty);
        }
      }
      _ => return Err(crate::Error::UnsupportedStructure),
    },
    _ => return Err(crate::Error::UnsupportedStructure),
  }
  let expanded = quote! {
    impl wtx::http::server_framework::ConnAux for #name {
      type Init = Self;

      #[inline]
      fn conn_aux(init: Self::Init) -> wtx::Result<Self> {
        Ok(init)
      }
    }

    #(
      impl wtx::misc::Lease<#field_tys> for #name {
        #[inline]
        fn lease(&self) -> &#field_tys {
          &self.#field_names
        }
      }

      impl wtx::misc::LeaseMut<#field_tys> for #name {
          #[inline]
          fn lease_mut(&mut self) -> &mut #field_tys {
            &mut self.#field_names
          }
        }
    )*
  };
  Ok(proc_macro::TokenStream::from(expanded))
}
