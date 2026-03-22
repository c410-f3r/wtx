mod keywords {
  syn::custom_keyword!(from_records);
}

use crate::misc::parts_from_generics;
use core::option::IntoIter;
use syn::{
  Data, DeriveInput, Fields, GenericParam, Ident, Path, Type, WherePredicate,
  parse::{Parse, ParseStream},
  punctuated::Punctuated,
  spanned::Spanned as _,
  token::Comma,
};

pub(crate) fn from_records(
  item: proc_macro::TokenStream,
) -> crate::Result<proc_macro::TokenStream> {
  let input = syn::parse::<DeriveInput>(item)?;
  let name = &input.ident;

  let mut custom_bounds = Vec::<WherePredicate>::new();
  let mut database_opt = None;
  let mut modifier_opt = None;
  for input_attr in &input.attrs {
    if let Some(first) = input_attr.path().segments.first()
      && first.ident == "from_records"
    {
      let attrs = input_attr.parse_args::<ContainerAttrs>()?;
      database_opt = Some(attrs.database);
      modifier_opt = attrs.modifier;
      custom_bounds = attrs.bounds;
    }
  }

  let (params, where_predicates) = parts_from_generics(&input.generics);
  let database = database_opt.ok_or_else(|| crate::Error::MissingDatabase(name.span()))?;

  let DatabaseGenericParams {
    database_err_ty,
    database_generic_err_param,
    database_generic_err_where_predicate,
    modifier_call,
  } = database_generic_params(&database, modifier_opt, params)?;
  let mut pdi = ProcessDataInput::default();
  process_data(&input, name, &mut pdi)?;
  let ProcessDataInput {
    decodes_after_id,
    decodes_after_id_method,
    decodes_before_id,
    decodes_before_id_method,
    fields_num,
    id_opt,
    ignores,
    ignores_tys,
    manys,
    manys_tys,
    ones,
    ones_tys,
    ones_opts,
    ones_opts_tys,
  } = &pdi;

  let additional_bounds = params.iter().filter_map(|el| {
    let GenericParam::Type(type_param) = el else {
      return None;
    };
    let ident = &type_param.ident;
    if struct_type_is_field_ty(ones_tys, ident) || struct_type_is_field_ty(ones_opts_tys, ident) {
      return Some(quote::quote!(#ident: wtx::database::FromRecords<'exec, #database>));
    }
    if !struct_type_is_field_ty(ignores_tys, ident) && !struct_type_is_field_ty(manys_tys, ident) {
      return Some(quote::quote!(#ident: wtx::codec::Decode<'exec, #database>));
    }
    None
  });
  let (id_idx, id_ident, id_ty) = match (manys.is_empty(), id_opt) {
    (false, None) => return Err(crate::Error::MissingId(name.span())),
    (true, None) => (quote::quote!(None), None, quote::quote!(())),
    (_, Some((id_idx, id_ident, id_ty))) => {
      (quote::quote!(Some(#id_idx)), *id_ident, quote::quote!(#id_ty))
    }
  };

  let id_ident_iter0 = id_ident.iter();
  let id_ident_iter1 = id_ident.iter();

  let expanded = quote::quote!(
    impl<'exec, #(#database_generic_err_param,)* #params> wtx::database::FromRecords<'exec, #database> for #name<#params>
    where
      #(#database_generic_err_where_predicate: From<wtx::Error>,)*
      #(#custom_bounds,)*
      #(#additional_bounds,)*
      #where_predicates
    {
      const FIELDS: u16 = #fields_num;
      const ID_IDX: Option<usize> = #id_idx;

      type IdTy = #id_ty;

      #[inline]
      fn from_records(
        _curr_params: &mut wtx::database::FromRecordsParams<<#database as wtx::database::Database>::Record<'exec>>,
        _records: &<#database as wtx::database::Database>::Records<'exec>,
      ) -> Result<Self, <#database as wtx::codec::CodecController>::Error> {
        use wtx::database::Record as _;

        let is_in_one_relationship = _curr_params.is_in_one_relationship;

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
          let #ones = {
            _curr_params.is_in_one_relationship = true;
            let rslt = <_ as wtx::database::FromRecords::<#database>>::from_records(_curr_params, _records);
            _curr_params.is_in_one_relationship = false;
            rslt?
          };
        )*
        #(
          let #ones_opts = {
            _curr_params.is_in_one_relationship = true;
            let prev_curr_field_idx = _curr_params.curr_field_idx;
            let rslt = <#ones_opts_tys as wtx::database::FromRecords::<#database>>::from_records(_curr_params, _records);
            _curr_params.is_in_one_relationship = false;
            if rslt.is_err() {
              let curr_field_idx = prev_curr_field_idx.wrapping_add(
                <#ones_opts_tys as wtx::database::FromRecords::<#database>>::FIELDS.into()
              );
              _curr_params.curr_field_idx = curr_field_idx;
              None
            } else {
              rslt?
            }
          };
        )*

        let prev_consumed_records = _curr_params.consumed_records;
        #(
          let mut #manys: #manys_tys = Default::default();
          wtx::database::seek_related_entities(
            _curr_params,
            (_parent_id_iter0, _parent_id_column_idx),
            _records,
            |elem| Ok(wtx::collection::TryExtend::try_extend(&mut #manys, [elem]).map_err(#database_err_ty::from)?)
          )?;
        )*
        if prev_consumed_records == _curr_params.consumed_records && !is_in_one_relationship {
          _curr_params.inc_consumed_records(1);
        }

        let mut _instance = Self {
          #(#decodes_before_id,)*
          #(#id_ident_iter1,)*
          #(#decodes_after_id,)*
          #(#ignores: Default::default(),)*
          #(#manys,)*
          #(#ones,)*
          #(#ones_opts,)*
        };
        #modifier_call
        Ok(_instance)
      }
    }
  );
  Ok(proc_macro::TokenStream::from(expanded))
}

#[derive(Debug)]
enum FieldTy {
  Decode,
  Id,
  Ignore,
  Many,
  One,
}

#[derive(Debug)]
struct ContainerAttrs {
  bounds: Vec<WherePredicate>,
  database: Path,
  modifier: Option<Path>,
}

impl Parse for ContainerAttrs {
  fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
    let database = input.parse::<Path>()?;
    let mut bounds = Vec::new();
    let mut modifier = None;
    while input.peek(syn::Token![,]) {
      let _ = input.parse::<syn::Token![,]>()?;
      if input.is_empty() {
        break;
      }
      let key = input.parse::<Ident>()?;
      let _ = input.parse::<syn::Token![=]>()?;
      if key == "bound" {
        bounds.push(input.parse::<syn::LitStr>()?.parse()?);
      } else if key == "modifier" {
        if modifier.is_some() {
          return Err(crate::Error::DuplicatedContainerTy(key.span()).into());
        }
        modifier = Some(input.parse::<Path>()?);
      } else {
        return Err(crate::Error::UnknownContainerTy.into());
      }
    }

    Ok(Self { bounds, database, modifier })
  }
}

#[derive(Debug)]
struct DatabaseGenericParams<'any> {
  database_err_ty: proc_macro2::TokenStream,
  database_generic_err_param: IntoIter<&'any Ident>,
  database_generic_err_where_predicate: IntoIter<&'any Ident>,
  modifier_call: Option<proc_macro2::TokenStream>,
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
      "ignore" => FieldTy::Ignore,
      "many" => FieldTy::Many,
      "one" => FieldTy::One,
      _ => return Err(crate::Error::UnknownFieldTy(path.span()).into()),
    };
    Ok(Self { ty: Some(ty) })
  }
}

#[derive(Debug, Default)]
struct ProcessDataInput<'any> {
  decodes_after_id: Vec<&'any Option<Ident>>,
  decodes_after_id_method: Vec<Ident>,
  decodes_before_id: Vec<&'any Option<Ident>>,
  decodes_before_id_method: Vec<Ident>,
  fields_num: u16,
  id_opt: Option<(usize, Option<&'any Ident>, &'any Type)>,
  ignores: Vec<&'any Option<Ident>>,
  ignores_tys: Vec<&'any Type>,
  manys: Vec<&'any Option<Ident>>,
  manys_tys: Vec<&'any Type>,
  ones: Vec<&'any Option<Ident>>,
  ones_tys: Vec<&'any Type>,
  ones_opts: Vec<&'any Option<Ident>>,
  ones_opts_tys: Vec<&'any Type>,
}

fn database_generic_params<'any>(
  database: &'any Path,
  modifier_opt: Option<Path>,
  params: &Punctuated<GenericParam, Comma>,
) -> crate::Result<DatabaseGenericParams<'any>> {
  let mut database_generic_err = None;
  'outer: for segment in &database.segments {
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
      continue;
    };
    for arg in &args.args {
      if let syn::GenericArgument::Type(Type::Path(type_path)) = arg
        && type_path.qself.is_none()
        && type_path.path.leading_colon.is_none()
        && type_path.path.segments.len() == 1
        && let Some(seg) = &type_path.path.segments.first()
        && matches!(seg.arguments, syn::PathArguments::None)
      {
        database_generic_err = Some(&seg.ident);
        break 'outer;
      }
    }
  }
  if let Some(elem) = database_generic_err
    && elem != "E"
  {
    return Err(crate::Error::ReservedTypeNameE);
  }
  let mut has_struct_type_named_e = false;
  if database_generic_err.is_some() {
    for el in params {
      let GenericParam::Type(type_param) = el else {
        continue;
      };
      if &type_param.ident == "E" {
        has_struct_type_named_e = true;
      }
    }
  }
  let database_err_ty = if let Some(elem) = database_generic_err {
    quote::quote!(#elem)
  } else {
    quote::quote!(wtx::Error)
  };
  let database_generic_err_param =
    if has_struct_type_named_e { None.into_iter() } else { database_generic_err.into_iter() };
  let database_generic_err_where_predicate = database_generic_err.into_iter();
  let modifier_call = modifier_opt.map(|elem| quote::quote!(#elem(&mut _instance);));
  Ok(DatabaseGenericParams {
    database_err_ty,
    database_generic_err_param,
    database_generic_err_where_predicate,
    modifier_call,
  })
}

fn extract_decode_method(ty: &Type) -> Ident {
  if is_opt(ty) {
    return Ident::new("decode_opt", ty.span());
  }
  Ident::new("decode", ty.span())
}

fn is_opt(ty: &Type) -> bool {
  if let Type::Path(path) = ty
    && let Some(first) = path.path.segments.first()
    && first.ident == "Option"
  {
    true
  } else {
    false
  }
}

fn process_data<'any>(
  input: &'any DeriveInput,
  name: &Ident,
  quote_params: &mut ProcessDataInput<'any>,
) -> Result<(), crate::Error> {
  match &input.data {
    Data::Struct(data) => match &data.fields {
      Fields::Named(fields) => {
        for (idx, elem) in fields.named.iter().enumerate() {
          let mut ty_opt = None;
          for attr in &elem.attrs {
            if let Some(first) = attr.path().segments.first()
              && first.ident == "from_records"
            {
              ty_opt = attr.parse_args::<FieldAttrs>()?.ty;
              break;
            }
          }
          match ty_opt.unwrap_or(FieldTy::Decode) {
            FieldTy::Decode => {
              quote_params.fields_num = quote_params.fields_num.wrapping_add(1);
              if quote_params.id_opt.is_none() {
                quote_params.decodes_before_id.push(&elem.ident);
                quote_params.decodes_before_id_method.push(extract_decode_method(&elem.ty));
              } else {
                quote_params.decodes_after_id.push(&elem.ident);
                quote_params.decodes_after_id_method.push(extract_decode_method(&elem.ty));
              }
            }
            FieldTy::Id => {
              quote_params.fields_num = quote_params.fields_num.wrapping_add(1);
              if quote_params.id_opt.is_none() {
                quote_params.id_opt = Some((idx, elem.ident.as_ref(), &elem.ty));
              } else {
                return Err(crate::Error::DuplicatedId(name.span()));
              }
            }
            FieldTy::Ignore => {
              quote_params.ignores.push(&elem.ident);
              quote_params.ignores_tys.push(&elem.ty);
            }
            FieldTy::Many => {
              quote_params.fields_num = quote_params.fields_num.wrapping_add(1);
              quote_params.manys.push(&elem.ident);
              quote_params.manys_tys.push(&elem.ty);
            }
            FieldTy::One => {
              quote_params.fields_num = quote_params.fields_num.wrapping_add(1);
              if is_opt(&elem.ty) {
                quote_params.ones_opts.push(&elem.ident);
                quote_params.ones_opts_tys.push(&elem.ty);
              } else {
                quote_params.ones.push(&elem.ident);
                quote_params.ones_tys.push(&elem.ty);
              }
            }
          }
        }
      }
      _ => return Err(crate::Error::UnsupportedStructure),
    },
    _ => return Err(crate::Error::UnsupportedStructure),
  }
  Ok(())
}

fn struct_type_is_field_ty(fields_tys: &[&Type], struct_ty_ident: &Ident) -> bool {
  fields_tys.iter().any(|ty| {
    if let Type::Path(type_path) = ty
      && type_path.path.segments.len() == 1
      && let Some(first) = type_path.path.segments.first()
      && &first.ident == struct_ty_ident
    {
      true
    } else {
      false
    }
  })
}
