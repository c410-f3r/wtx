use syn::{ItemStruct, ItemType};

#[derive(Clone, Copy, Debug)]
pub(crate) enum EnumStructOrType<'any> {
  Enum,
  Struct(&'any ItemStruct),
  Type(&'any ItemType),
}
