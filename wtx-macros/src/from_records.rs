mod keywords {
  syn::custom_keyword!(from_records);
}

use crate::misc::parts_from_generics;
use syn::{
  Data, DeriveInput, Fields, GenericParam, Ident, Path, Type,
  parse::{Parse, ParseStream},
  spanned::Spanned as _,
};

pub(crate) fn from_records(
  item: proc_macro::TokenStream,
) -> crate::Result<proc_macro::TokenStream> {
  let input = syn::parse::<DeriveInput>(item)?;
  let name = input.ident;

  let mut database_opt = None;
  for input_attr in &input.attrs {
    if let Some(first) = input_attr.path().segments.first() {
      if first.ident == "from_records" {
        database_opt = Some(input_attr.parse_args::<ContainerAttrs>()?.database);
      }
    }
  }

  let database = database_opt.ok_or_else(|| crate::Error::MissingDatabase(name.span()))?;
  let (params, where_predicates) = parts_from_generics(&input.generics);
  let additional_where_predicates = params.iter().filter_map(|el| {
    if let GenericParam::Type(type_param) = el {
      let ident = &type_param.ident;
      Some(quote::quote! { #ident: wtx::misc::Decode<'exec, #database>, })
    } else {
      None
    }
  });
  let mut decodes_after_id = Vec::new();
  let mut decodes_after_id_method = Vec::new();
  let mut decodes_before_id = Vec::new();
  let mut decodes_before_id_method = Vec::new();
  let mut id_opt = None;
  let mut manys = Vec::new();
  let mut manys_ty = Vec::new();
  let mut ones = Vec::new();

  match &input.data {
    Data::Struct(data) => match &data.fields {
      Fields::Named(fields) => {
        for (idx, elem) in fields.named.iter().enumerate() {
          let mut ty_opt = None;
          for attr in &elem.attrs {
            if let Some(first) = attr.path().segments.first() {
              if first.ident == "from_records" {
                ty_opt = attr.parse_args::<FieldAttrs>()?.ty;
                break;
              }
            }
          }
          let ty = ty_opt.unwrap_or(FieldTy::Decode);
          match ty {
            FieldTy::Decode => {
              if id_opt.is_none() {
                decodes_before_id.push(&elem.ident);
                decodes_before_id_method.push(extract_decode_method(&elem.ty));
              } else {
                decodes_after_id.push(&elem.ident);
                decodes_after_id_method.push(extract_decode_method(&elem.ty));
              }
            }
            FieldTy::Id => {
              if id_opt.is_none() {
                id_opt = Some((idx, elem.ident.as_ref(), &elem.ty));
              } else {
                return Err(crate::Error::DuplicatedId(name.span()));
              }
            }
            FieldTy::Many => {
              manys.push(&elem.ident);
              manys_ty.push(&elem.ty);
            }
            FieldTy::One => {
              ones.push(&elem.ident);
            }
          }
        }
      }
      _ => return Err(crate::Error::UnsupportedStructure),
    },
    _ => return Err(crate::Error::UnsupportedStructure),
  }

  let (id_idx, id_ident, id_ty) = match (manys.is_empty(), id_opt) {
    (false, None) => return Err(crate::Error::MissingId(name.span())),
    (true, None) => (quote::quote!(None), None, quote::quote!(())),
    (_, Some((id_idx, id_ident, id_ty))) => {
      (quote::quote!(Some(#id_idx)), id_ident, quote::quote!(#id_ty))
    }
  };

  let id_ident_iter0 = id_ident.iter();
  let id_ident_iter1 = id_ident.iter();

  let expanded = quote::quote! {
    impl<'exec, #params> wtx::database::FromRecords<'exec, #database> for #name<#params>
    where
      #(#additional_where_predicates)*
      #where_predicates
    {
      const ID_IDX: Option<usize> = #id_idx;

      type IdTy = #id_ty;

      #[inline]
      fn from_records(
        _curr_params: &mut wtx::database::FromRecordsParams<<#database as wtx::database::Database>::Record<'exec>>,
        _records: &<#database as wtx::database::Database>::Records<'exec>,
      ) -> Result<Self, <#database as wtx::misc::DEController>::Error> {
        use wtx::database::Record as _;

        #(
          let #decodes_before_id = _curr_params.curr_record.#decodes_before_id_method(_curr_params.curr_field_idx)?;
          _curr_params.inc_field_idx();
        )*

        #(
          let _parent_id_column_idx = _curr_params.curr_field_idx;
          let #id_ident_iter0: #id_ty = _curr_params.curr_record.decode(_curr_params.curr_field_idx)?;
          _curr_params.inc_field_idx();
          let _parent_id_iter0 = #id_ident_iter0;
        )*

        #(
          let #decodes_after_id = _curr_params.curr_record.#decodes_after_id_method(_curr_params.curr_field_idx)?;
          _curr_params.inc_field_idx();
        )*

        #(
          let #ones = <_>::from_records(_curr_params, _records)?;
        )*

        let prev_consumed_records = _curr_params.consumed_records;
        #(
          let mut #manys: #manys_ty = <_>::default();
          wtx::database::seek_related_entities(
            _curr_params,
            (_parent_id_iter0, _parent_id_column_idx),
            _records,
            |elem| Ok(#manys.push(elem).map_err(wtx::Error::from)?)
          )?;
        )*

        if prev_consumed_records == _curr_params.consumed_records {
          _curr_params.inc_consumed_records(1);
        }

        Ok(Self {
          #(#decodes_before_id,)*
          #(#id_ident_iter1,)*
          #(#decodes_after_id,)*
          #(#manys,)*
          #(#ones,)*
        })
      }
    }
  };
  Ok(proc_macro::TokenStream::from(expanded))
}

#[derive(Debug)]
enum FieldTy {
  Decode,
  Id,
  Many,
  One,
}

#[derive(Debug)]
struct ContainerAttrs {
  database: Path,
}

impl Parse for ContainerAttrs {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    let database = input.parse::<Path>()?;
    Ok(Self { database })
  }
}

#[derive(Debug)]
struct FieldAttrs {
  ty: Option<FieldTy>,
}

impl Parse for FieldAttrs {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    let path = input.parse::<Path>()?;
    let Some(first) = path.segments.first() else {
      return Err(crate::Error::UnknownFieldTy(path.span()).into());
    };
    let ty = match first.ident.to_string().as_str() {
      "decode" => FieldTy::Decode,
      "id" => FieldTy::Id,
      "many" => FieldTy::Many,
      "one" => FieldTy::One,
      _ => return Err(crate::Error::UnknownFieldTy(path.span()).into()),
    };
    Ok(Self { ty: Some(ty) })
  }
}

fn extract_decode_method(ty: &Type) -> Ident {
  if let Type::Path(path) = ty {
    if let Some(first) = path.path.segments.first() {
      if first.ident == "Option" {
        return Ident::new("decode_opt", ty.span());
      }
    }
  }
  Ident::new("decode", ty.span())
}
