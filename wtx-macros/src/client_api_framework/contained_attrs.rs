use syn::Attribute;

pub(crate) trait ContainedAttrs {
  fn contained_attrs(&mut self) -> Option<&mut Vec<Attribute>>;
}

impl ContainedAttrs for syn::Item {
  fn contained_attrs(&mut self) -> Option<&mut Vec<Attribute>> {
    Some(match self {
      Self::Const(item) => item.attrs.as_mut(),
      Self::Enum(item) => item.attrs.as_mut(),
      Self::ExternCrate(item) => item.attrs.as_mut(),
      Self::Fn(item) => item.attrs.as_mut(),
      Self::ForeignMod(item) => item.attrs.as_mut(),
      Self::Impl(item) => item.attrs.as_mut(),
      Self::Macro(item) => item.attrs.as_mut(),
      Self::Mod(item) => item.attrs.as_mut(),
      Self::Static(item) => item.attrs.as_mut(),
      Self::Struct(item) => item.attrs.as_mut(),
      Self::Trait(item) => item.attrs.as_mut(),
      Self::TraitAlias(item) => item.attrs.as_mut(),
      Self::Type(item) => item.attrs.as_mut(),
      Self::Union(item) => item.attrs.as_mut(),
      Self::Use(item) => item.attrs.as_mut(),
      Self::Verbatim(_) | _ => return None,
    })
  }
}
