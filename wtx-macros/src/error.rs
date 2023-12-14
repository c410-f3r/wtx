use proc_macro2::Span;

#[allow(
  // `syn::Error` is composed by a single vector
  variant_size_differences
)]
#[derive(Debug)]
pub(crate) enum Error {
  AbsentFieldInUnnamedStruct(Span),
  AbsentReqOrRes(Span),
  BadAfterSending(Span),
  BadAux(Span),
  BadAuxData(Span, String),
  BadBeforeSending(Span),
  BadField(Span),
  BadParams(Span),
  BadReq(Span),
  BadRes(Span),
  DuplicatedGlobalPkgAttr(Span),
  DuplicatedLocalPkgAttr(Span),
  IncorrectJsonRpcDataFormat,
  MandatoryOuterAttrsAreNotPresent,
  NoEnumStructOrType(Span),
  ResponsesCanNotHaveTypeParams(Span),
  Syn(syn::Error),
  UnknownDataFormat,
  UnknownTransport(Span),
}

impl From<syn::Error> for Error {
  fn from(from: syn::Error) -> Self {
    Self::Syn(from)
  }
}

impl From<Error> for syn::Error {
  fn from(from: Error) -> Self {
    match from {
      Error::AbsentFieldInUnnamedStruct(span) => syn::Error::new(
        span,
        "Unnamed structures must have a `#[pkg::field]` attribute on each field.",
      ),
      Error::AbsentReqOrRes(span) => syn::Error::new(
        span,
        "The `#[pkg]` module must have an inner `#[pkg::req_data]` element and an inner \
          `#[pkg::res_data]` element.",
      ),
      Error::BadAfterSending(span) => syn::Error::new(
        span,
        "`#[pkg::after_sending]` must be an async function named `after_sending` containing any \
        combination of `api: &mut SomeApi`, `params: &mut SomePackageParams`, `req_bytes: &[u8]`, \
        and `req_params: &mut SomeRequestParams`.",
      ),
      Error::BadAux(span) => syn::Error::new(
        span,
        "#[pkg::aux] must be an item implementation with none, one `#[pkg::aux_data]`, one \
          `#[pkg::aux_params]` or both `#[pkg::aux_data]` and `#[pkg::aux_params]`",
      ),
      Error::BadAuxData(span, name) => {
        syn::Error::new(span, format!("This method must be named `{name}`"))
      }
      Error::BadBeforeSending(span) => syn::Error::new(
        span,
        "`#[pkg::before_sending]` must be an async function named `before_sending` containing any \
        combination of `api: &mut SomeApi`, `params: &mut SomePackageParams` and `res_params: &mut \
        SomeResponseParams`.",
      ),
      Error::BadField(span) => syn::Error::new(
        span,
        "Field attributes must be annotated as `#[pkg::field(name = \"SomeName\")]`",
      ),
      Error::BadParams(span) => {
        syn::Error::new(span, "Parameters must end with the `Params` suffix.")
      }
      Error::BadReq(span) => syn::Error::new(span, "Request data must end with the `Req` suffix."),
      Error::BadRes(span) => syn::Error::new(span, "Response data must end with the `Res` suffix."),
      Error::DuplicatedGlobalPkgAttr(span) => syn::Error::new(
        span,
        "It is not possible to have more than one declaration of this `pkg` attribute in the \
          same package.",
      ),
      Error::DuplicatedLocalPkgAttr(span) => syn::Error::new(
        span,
        "It is not possible to have more than one `pkg` attribute in the same element.",
      ),
      Error::IncorrectJsonRpcDataFormat => syn::Error::new(
        Span::call_site(),
        "JSON-RPC expects the name of its method. For example, \
          `#[pkg(data_format(json_rpc(\"method\")))]`",
      ),
      Error::MandatoryOuterAttrsAreNotPresent => syn::Error::new(
        Span::call_site(),
        "All packages must have an `api` and a `data_format` attribute. For example, \
          #[pkg(api(SomeApi), data_format(json))]",
      ),
      Error::NoEnumStructOrType(span) => {
        syn::Error::new(span, "Invalid item. Expected enum, struct or type.")
      }
      Error::ResponsesCanNotHaveTypeParams(span) => {
        syn::Error::new(span, "Responses can not have type parameters")
      }
      Error::Syn(error) => error,
      Error::UnknownDataFormat => syn::Error::new(Span::call_site(), "Unknown data format."),
      Error::UnknownTransport(span) => syn::Error::new(span, "Unknown transport."),
    }
  }
}
