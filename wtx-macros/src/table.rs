use core::fmt::Write as _;
use syn::{Data, DeriveInput, Fields};

pub(crate) fn table(item: proc_macro::TokenStream) -> crate::Result<proc_macro::TokenStream> {
  let input = syn::parse::<DeriveInput>(item)?;
  let struct_name = &input.ident;
  let struct_name_string = &input.ident.to_string().to_lowercase();

  let fields = match input.data {
    Data::Struct(data) => match data.fields {
      Fields::Named(fields) => {
        let mut string = String::new();
        let mut iter = fields.named.iter();
        if let Some(elem) = iter.next() {
          let field_name = elem.ident.as_ref().ok_or(crate::Error::InvalidStruct)?;
          let _rslt = write!(string, r#""{struct_name_string}".{field_name}"#);
        }
        for elem in iter {
          let field_name = elem.ident.as_ref().ok_or(crate::Error::InvalidStruct)?;
          let _rslt = write!(string, r#","{struct_name_string}".{field_name}"#);
        }
        string
      }
      _ => return Err(crate::Error::InvalidStruct),
    },
    _ => return Err(crate::Error::InvalidStruct),
  };

  let expanded = quote::quote! {
    impl #struct_name {
      /// Fields separated by commas
      pub(crate) fn fields() -> &'static str {
        #fields
      }
    }
  };

  Ok(proc_macro::TokenStream::from(expanded))
}
