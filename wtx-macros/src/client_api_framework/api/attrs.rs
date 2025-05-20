use crate::client_api_framework::{api::mode::Mode, transport_group::TransportGroup};
use syn::{Meta, MetaList, Path, Token, punctuated::Punctuated};

#[derive(Debug)]
pub(crate) struct Attrs {
  pub(crate) error: Path,
  pub(crate) mode: Mode,
  pub(crate) pkgs_aux: Option<Path>,
  pub(crate) transports: Vec<TransportGroup>,
}

impl TryFrom<&Punctuated<Meta, Token![,]>> for Attrs {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &Punctuated<Meta, Token![,]>) -> Result<Self, Self::Error> {
    let mut error: Option<Path> = None;
    let mut pkgs_aux = None;
    let mut transports = Vec::new();
    let mut mode = None;
    for meta in from {
      let Meta::List(meta_list) = meta else {
        continue;
      };
      let Some(first_meta_list_path_seg) = meta_list.path.segments.first() else {
        continue;
      };
      match first_meta_list_path_seg.ident.to_string().as_str() {
        "error" => {
          error = fist_meta_from_meta_list(meta_list).map(|el| match el {
            Meta::Path(elem) => elem,
            Meta::List(elem) => elem.path,
            Meta::NameValue(elem) => elem.path,
          });
        }
        "mode" => 'block: {
          if let Some(path) = fist_meta_from_meta_list(meta_list) {
            match path.path().get_ident().map(ToString::to_string).as_deref() {
              Some("auto") => {
                mode = Some(Mode::Auto);
                break 'block;
              }
              Some("manual") => {
                mode = Some(Mode::Manual);
                break 'block;
              }
              _ => {}
            }
          }
          return Err(crate::Error::UnknownApiMode(first_meta_list_path_seg.ident.span()));
        }
        "pkgs_aux" => {
          pkgs_aux = fist_meta_from_meta_list(meta_list).map(|el| match el {
            Meta::Path(elem) => elem,
            Meta::List(elem) => elem.path,
            Meta::NameValue(elem) => elem.path,
          });
        }
        "transport" => {
          let metas = meta_list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
          transports = metas.iter().map(TryInto::try_into).collect::<crate::Result<_>>()?;
        }
        _ => {}
      }
    }
    Ok(Self {
      error: error.ok_or(crate::Error::AbsentApi)?,
      mode: mode.unwrap_or(Mode::Manual),
      pkgs_aux,
      transports,
    })
  }
}

fn fist_meta_from_meta_list(meta_list: &MetaList) -> Option<Meta> {
  meta_list
    .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
    .ok()?
    .into_iter()
    .next()
}
