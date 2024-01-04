#[cfg(feature = "std")]
macro_rules! loop_files {
  ($buffer:expr, $iter:expr, $n:expr, $cb:expr) => {{
    loop {
      for el in $iter.by_ref().take($n) {
        $buffer.push(el?);
      }
      if $buffer.is_empty() {
        break;
      }
      $cb;
      $buffer.clear();
    }
  }};
}
