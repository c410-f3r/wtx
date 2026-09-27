use crate::collections::VectorUsize;

#[allow(unreachable_pub, reason = "tests")]
#[test]
fn compiles() {
  create_packages_aux_wrapper!();
  let _pkg = PkgsAux::from_minimum((), (), ());
  let _pkg = PkgsAux::new((), 0, VectorUsize::new(), (), false, false, ());
}
