use proc_macro2::{Ident, TokenStream};

#[derive(Debug)]
pub(crate) struct DataFormatElems {
  pub(crate) dfe_data_format_builder_fn: Ident,
  pub(crate) dfe_ext_req_ctnt_wrapper: Ident,
  pub(crate) dfe_ext_res_ctnt_wrapper: Ident,
  pub(crate) dfe_pkgs_aux_call: TokenStream,
}
