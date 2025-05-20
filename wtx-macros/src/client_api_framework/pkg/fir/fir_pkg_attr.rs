use crate::misc::fist_meta_path_from_meta_list;
use syn::{Meta, Path, Token, punctuated::Punctuated};

const EMPTY_NESTED_META: Punctuated<Meta, Token![,]> = Punctuated::new();

#[derive(Debug)]
pub(crate) struct FirPkgAttr {
  pub(crate) data_formats: Punctuated<Meta, Token![,]>,
  pub(crate) id: Path,
  pub(crate) transports: Punctuated<Meta, Token![,]>,
}

impl TryFrom<&Punctuated<Meta, Token![,]>> for FirPkgAttr {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &Punctuated<Meta, Token![,]>) -> Result<Self, Self::Error> {
    let mut api = None;
    let mut data_formats = None;
    let mut transports = None;
    for nested_meta in from {
      let Meta::List(meta_list) = nested_meta else {
        continue;
      };
      let Some(first_meta_list_path_seg) = meta_list.path.segments.first() else {
        continue;
      };
      match first_meta_list_path_seg.ident.to_string().as_str() {
        "data_format" => {
          data_formats = Some(meta_list.parse_args_with(Punctuated::parse_terminated)?);
        }
        "id" => {
          api = fist_meta_path_from_meta_list(meta_list);
        }
        "transport" => {
          transports = Some(meta_list.parse_args_with(Punctuated::parse_terminated)?);
        }
        _ => {}
      }
    }
    Ok(Self {
      id: api.ok_or(crate::Error::MandatoryOuterAttrsAreNotPresent)?,
      data_formats: data_formats.unwrap_or(EMPTY_NESTED_META),
      transports: transports.unwrap_or(EMPTY_NESTED_META),
    })
  }
}
