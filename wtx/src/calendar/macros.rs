macro_rules! manage_out_of_bounds {
  (@one, $min:expr, $max:expr, $elem:expr, $to:expr $(,)?) => {
    manage_out_of_bounds!($min, $max, $elem, $to = $to.wrapping_add(1), $to = $to.wrapping_sub(1))
  };
  ($min:expr, $max:expr, $elem:expr, $greater:expr, $lesser:expr $(,)?) => {
    if $elem >= $max {
      $elem = $elem.wrapping_sub($max.wrapping_sub($min));
      $greater
    } else if $elem < $min {
      $elem = $elem.wrapping_add($max.wrapping_sub($min));
      $lesser
    }
  };
}
