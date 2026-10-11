#[derive(Clone, Copy, Debug)]
pub(crate) struct Metadata<L, M> {
  pub(crate) len: L,
  pub(crate) misc: M,
  pub(crate) offset: L,
}
