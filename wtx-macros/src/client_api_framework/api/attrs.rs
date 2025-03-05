use crate::client_api_framework::{api::mode::Mode, transport_group::TransportGroup};
use syn::{Meta, MetaList, NestedMeta, Path};

#[derive(Debug)]
pub(crate) struct Attrs<'attrs> {
  pub(crate) error: &'attrs Path,
  pub(crate) mode: Mode,
  pub(crate) pkgs_aux: Option<&'attrs Path>,
  pub(crate) transports: Vec<TransportGroup>,
}

impl<'attrs> TryFrom<&'attrs [NestedMeta]> for Attrs<'attrs> {
  type Error = crate::Error;

  fn try_from(from: &'attrs [NestedMeta]) -> Result<Self, Self::Error> {
    let mut error = None;
    let mut pkgs_aux = None;
    let mut transports = Vec::new();
    let mut mode = None;
    for nested_meta in from {
      let NestedMeta::Meta(Meta::List(meta_list)) = nested_meta else {
        continue;
      };
      let Some(first_meta_list_path_seg) = meta_list.path.segments.first() else {
        continue;
      };
      match first_meta_list_path_seg.ident.to_string().as_str() {
        "error" => {
          error = first_nested_meta_path(meta_list);
        }
        "mode" => 'block: {
          if let Some(path) = first_nested_meta_path(meta_list) {
            match path.get_ident().map(ToString::to_string).as_deref() {
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
          pkgs_aux = first_nested_meta_path(meta_list);
        }
        "transport" => {
          transports =
            meta_list.nested.iter().map(TryInto::try_into).collect::<crate::Result<_>>()?;
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

fn first_nested_meta_path(meta_list: &MetaList) -> Option<&Path> {
  let Some(NestedMeta::Meta(meta)) = meta_list.nested.first() else {
    return None;
  };
  if let Meta::Path(elem) = meta { Some(elem) } else { None }
}
