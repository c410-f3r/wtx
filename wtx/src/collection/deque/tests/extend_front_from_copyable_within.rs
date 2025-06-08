use crate::collection::{
  Deque,
  deque::tests::{
    no_wrapping_00, no_wrapping_01, no_wrapping_02, no_wrapping_11, no_wrapping_36, wrapping_00,
    wrapping_22, wrapping_60, wrapping_64, wrapping_70, wrapping_71, wrapping_77,
  },
};

#[test]
fn extend_front_from_copyable_within() {
  test(no_wrapping_00, (&[], &[]), (&[], &[]), (&[], &[]), (&[], &[]), (&[], &[]), (&[], &[]));
  test(no_wrapping_11, (&[], &[]), (&[], &[]), (&[], &[]), (&[], &[]), (&[], &[]), (&[], &[]));

  test(
    no_wrapping_01,
    (&[1], &[1]),
    (&[1], &[]),
    (&[1], &[]),
    (&[1], &[]),
    (&[1], &[]),
    (&[1], &[]),
  );
  test(
    no_wrapping_02,
    (&[1], &[1, 2]),
    (&[1, 2], &[1, 2]),
    (&[1, 2], &[]),
    (&[2], &[1, 2]),
    (&[1, 2], &[]),
    (&[1, 2], &[]),
  );
  test(
    no_wrapping_36,
    (&[4, 4, 5, 6], &[]),
    (&[4, 5, 4, 5, 6], &[]),
    (&[4, 5, 6], &[]),
    (&[5, 4, 5, 6], &[]),
    (&[5, 6, 4, 5, 6], &[]),
    (&[4, 5, 6], &[]),
  );

  test(
    wrapping_00,
    (&[1, 1, 2, 3, 4, 5, 6, 7, 8], &[]),
    (&[1, 2, 1, 2, 3, 4, 5, 6, 7, 8], &[]),
    (&[1, 2, 3, 4, 5, 6, 7, 8], &[]),
    (&[2, 1, 2, 3, 4, 5, 6, 7, 8], &[]),
    (&[2, 3, 1, 2, 3, 4, 5, 6, 7, 8], &[]),
    (&[1, 2, 3, 4, 5, 6, 7, 8], &[]),
  );
  test(
    wrapping_22,
    (&[1, 1, 2, 3, 4, 5, 6], &[7, 8]),
    (&[1, 2, 1, 2, 3, 4, 5, 6], &[7, 8]),
    (&[1, 2, 3, 4, 5, 6], &[7, 8]),
    (&[2, 1, 2, 3, 4, 5, 6], &[7, 8]),
    (&[2, 3, 1, 2, 3, 4, 5, 6], &[7, 8]),
    (&[1, 2, 3, 4, 5, 6], &[7, 8]),
  );
  test(
    wrapping_77,
    (&[1, 1], &[2, 3, 4, 5, 6, 7, 8]),
    (&[1], &[2, 3, 4, 5, 6, 7, 8]),
    (&[1], &[2, 3, 4, 5, 6, 7, 8]),
    (&[2, 1], &[2, 3, 4, 5, 6, 7, 8]),
    (&[2, 3, 1], &[2, 3, 4, 5, 6, 7, 8]),
    (&[1], &[2, 3, 4, 5, 6, 7, 8]),
  );

  test(
    wrapping_60,
    (&[7, 7, 8], &[]),
    (&[7, 8, 7, 8], &[]),
    (&[7, 8], &[]),
    (&[8, 7, 8], &[]),
    (&[7, 8], &[]),
    (&[7, 8], &[]),
  );
  test(
    wrapping_64,
    (&[1, 1, 2], &[3, 4, 5, 6]),
    (&[1, 2, 1, 2], &[3, 4, 5, 6]),
    (&[1, 2], &[3, 4, 5, 6]),
    (&[2, 1, 2], &[3, 4, 5, 6]),
    (&[1, 2], &[3, 4, 5, 6]),
    (&[1, 2], &[3, 4, 5, 6]),
  );
  test(
    wrapping_70,
    (&[1, 1], &[]),
    (&[1], &[]),
    (&[1], &[]),
    (&[1], &[]),
    (&[1], &[]),
    (&[1], &[]),
  );
  test(
    wrapping_71,
    (&[1, 1], &[2]),
    (&[1], &[2]),
    (&[1], &[2]),
    (&[2, 1], &[2]),
    (&[1], &[2]),
    (&[1], &[2]),
  );
}

fn test(
  deque_cb: impl Fn() -> Deque<i32>,
  skip0_copy1: (&[i32], &[i32]),
  skip0_copy2: (&[i32], &[i32]),
  skip0_copy10: (&[i32], &[i32]),
  skip1_copy1: (&[i32], &[i32]),
  skip1_copy2: (&[i32], &[i32]),
  skip1_copy10: (&[i32], &[i32]),
) {
  {
    let mut deque = deque_cb();
    let _rslt = deque.extend_front_from_copyable_within(0..1);
    assert_eq!(deque.as_slices(), skip0_copy1);
  }
  {
    let mut deque = deque_cb();
    let _rslt = deque.extend_front_from_copyable_within(0..2);
    assert_eq!(deque.as_slices(), skip0_copy2);
  }
  {
    let mut deque = deque_cb();
    let _rslt = deque.extend_front_from_copyable_within(0..10);
    assert_eq!(deque.as_slices(), skip0_copy10);
  }
  {
    let mut deque = deque_cb();
    let _rslt = deque.extend_front_from_copyable_within(1..2);
    assert_eq!(deque.as_slices(), skip1_copy1);
  }
  {
    let mut deque = deque_cb();
    let _rslt = deque.extend_front_from_copyable_within(1..3);
    assert_eq!(deque.as_slices(), skip1_copy2);
  }
  {
    let mut deque = deque_cb();
    let _rslt = deque.extend_front_from_copyable_within(1..11);
    assert_eq!(deque.as_slices(), skip1_copy10);
  }
}
