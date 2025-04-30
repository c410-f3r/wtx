use crate::collection::Vector;

#[allow(unreachable_pub, reason = "tests")]
#[test]
fn compiles() {
  create_packages_aux_wrapper!();
  let _pkg = PkgsAux::from_minimum((), (), ());
  let _pkg = PkgsAux::new((), Vector::new(), (), false, ());
}
