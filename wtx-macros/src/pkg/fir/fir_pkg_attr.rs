use syn::{punctuated::Punctuated, Meta, MetaList, NestedMeta, Path, Token};

const EMPTY_NESTED_META: &Punctuated<NestedMeta, Token![,]> = &Punctuated::new();

#[derive(Debug)]
pub(crate) struct FirPkgAttr<'attrs> {
  pub(crate) api: &'attrs Path,
  pub(crate) data_formats: &'attrs Punctuated<NestedMeta, Token![,]>,
  pub(crate) transports: &'attrs Punctuated<NestedMeta, Token![,]>,
}

impl<'attrs> TryFrom<&'attrs [NestedMeta]> for FirPkgAttr<'attrs> {
  type Error = crate::Error;

  fn try_from(from: &'attrs [NestedMeta]) -> Result<Self, Self::Error> {
    let mut api = None;
    let mut data_formats = None;
    let mut transports = None;
    for nested_meta in from {
      let NestedMeta::Meta(Meta::List(meta_list)) = nested_meta else {
        continue;
      };
      let Some(first_meta_list_path_seg) = meta_list.path.segments.first() else {
        continue;
      };
      match first_meta_list_path_seg.ident.to_string().as_str() {
        "api" => {
          api = first_nested_meta_path(meta_list);
        }
        "data_format" => {
          data_formats = Some(&meta_list.nested);
        }
        "transport" => {
          transports = Some(&meta_list.nested);
        }
        _ => {}
      }
    }
    Ok(Self {
      api: api.ok_or(crate::Error::MandatoryOuterAttrsAreNotPresent)?,
      data_formats: data_formats.unwrap_or(EMPTY_NESTED_META),
      transports: transports.unwrap_or(EMPTY_NESTED_META),
    })
  }
}

fn first_nested_meta_path(meta_list: &MetaList) -> Option<&Path> {
  let Some(NestedMeta::Meta(meta)) = meta_list.nested.first() else {
    return None;
  };
  if let Meta::Path(elem) = meta {
    Some(elem)
  } else {
    None
  }
}
