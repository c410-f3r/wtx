use syn::Attribute;

pub(crate) trait ContainedAttrs {
  fn contained_attrs(&mut self) -> Option<&mut Vec<Attribute>>;
}

impl ContainedAttrs for syn::Item {
  fn contained_attrs(&mut self) -> Option<&mut Vec<Attribute>> {
    Some(match *self {
      Self::Const(ref mut item) => item.attrs.as_mut(),
      Self::Enum(ref mut item) => item.attrs.as_mut(),
      Self::ExternCrate(ref mut item) => item.attrs.as_mut(),
      Self::Fn(ref mut item) => item.attrs.as_mut(),
      Self::ForeignMod(ref mut item) => item.attrs.as_mut(),
      Self::Impl(ref mut item) => item.attrs.as_mut(),
      Self::Macro(ref mut item) => item.attrs.as_mut(),
      Self::Macro2(ref mut item) => item.attrs.as_mut(),
      Self::Mod(ref mut item) => item.attrs.as_mut(),
      Self::Static(ref mut item) => item.attrs.as_mut(),
      Self::Struct(ref mut item) => item.attrs.as_mut(),
      Self::Trait(ref mut item) => item.attrs.as_mut(),
      Self::TraitAlias(ref mut item) => item.attrs.as_mut(),
      Self::Type(ref mut item) => item.attrs.as_mut(),
      Self::Union(ref mut item) => item.attrs.as_mut(),
      Self::Use(ref mut item) => item.attrs.as_mut(),
      _ => return None,
    })
  }
}
