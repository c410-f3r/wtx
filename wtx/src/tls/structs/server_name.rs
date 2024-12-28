use crate::tls::structs::name_type::NameType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerName<'any> {
  pub name_type: NameType,
  pub name: &'any str,
}
