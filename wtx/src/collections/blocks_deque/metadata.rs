#[derive(Clone, Copy, Debug)]
pub(crate) struct Metadata<M> {
  pub(crate) len: usize,
  pub(crate) misc: M,
  pub(crate) offset: usize,
}
