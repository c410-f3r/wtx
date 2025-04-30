// H = Head (Inclusive)
// LF = Left Free
// LO = Left Occupied
// RF = Right Free
// RO = Right Occupied
// T = Tail (Exclusive)

use crate::collection::{BlocksDeque, blocks_deque::BlockRef};

// [. . . . . . . .]: Empty - (LF=8, LO=0,RF=0, RO=0) - (H=0, T=0)
// [. . . . . . . H]: Push front - (LF=7, LO=0, RF=0, RO=1) - (H=7, T=8)
// [. . . . . H * *]: Push front - (LF=5, LO=0, RF=0, RO=3) - (H=5, T=8)
// [. . H * * * * *]: Push front - (LF=2, LO=0, RF=0, RO=6) - (H=2, T=8)
// [H * * * * * * *]: Push front - (LF=0, LO=0, RF=0, RO=8) - (H=0, T=8)
// [H * * * * * * .]: Pop back - (LF=0, LO=0, RF=2, RO=6) - (H=0, T=7)
// [* * * * * * * H]: Push front - (LF=0, LO=7, RF=0, RO=0) - (H=7, T=7)
// [* * * * * . . H]: Pop back - (LF=2, LO=5, RF=0, RO=1) - (H=7, T=5)
// [* * . . . . . H]: Pop back - (LF=5, LO=2, RF=0, RO=1) - (H=7, T=2)
// [* * . . H * * *]: Push front - (LF=2, LO=2, RF=0, RO=4) - (H=4, T=2)
// [* * H * * * * *]: Push front - (LF=0, LO=2, RF=0, RO=6) - (H=2, T=2)
// [. . H * * * * *]: Pop back - (LF=2, LO=0, RF=0, RO=6) - (H=2, T=8)
// [. . H * * * * .]: Pop back - (LF=2, LO=0, RF=1, RO=5) - (H=2, T=7)
// [. . H * . . . .]: Pop back - (LF=1, LO=0, RF=4, RO=3) - (H=2, T=4)
// [. H * * . . . .]: Push front - (LF=1, LO=0, RF=4, RO=3) - (H=2, T=4)
// [H * * * . . . .]: Push front - (LF=0, LO=0, RF=4, RO=4) - (H=0, T=4)
// [H . . . . . . .]: Pop back - (LF=0, LO=0, RF=7, RO=1) - (H=0, T=1)
// [. . . . . . . .]: Pop back - (LF=8, LO=0, RF=0, RO=0) - (H=0, T=0)
#[test]
fn pop_back() {
  let mut bq = BlocksDeque::with_exact_capacity(4, 8).unwrap();
  check_state(&bq, 0, 0, &[], &[]);

  bq.push_front_from_copyable_data([&[1][..]], ()).unwrap();
  check_state(&bq, 1, 1, &[1], &[]);

  bq.push_front_from_copyable_data([&[2, 3][..]], ()).unwrap();
  check_state(&bq, 2, 3, &[2, 3, 1], &[]);

  bq.push_front_from_copyable_data([&[4, 5], &[6][..]], ()).unwrap();
  check_state(&bq, 3, 6, &[4, 5, 6, 2, 3, 1], &[]);

  bq.push_front_from_copyable_data([&[7, 8][..]], ()).unwrap();
  check_state(&bq, 4, 8, &[7, 8, 4, 5, 6, 2, 3, 1], &[]);

  let _ = bq.pop_back();
  check_state(&bq, 3, 7, &[7, 8, 4, 5, 6, 2, 3], &[]);

  bq.push_front_from_copyable_data([&[9][..]], ()).unwrap();
  check_state(&bq, 4, 8, &[9], &[7, 8, 4, 5, 6, 2, 3]);

  let _ = bq.pop_back();
  check_state(&bq, 3, 6, &[9], &[7, 8, 4, 5, 6]);

  let _ = bq.pop_back();
  check_state(&bq, 2, 3, &[9], &[7, 8]);

  bq.push_front_from_copyable_data([&[10], &[11, 12][..]], ()).unwrap();
  check_state(&bq, 3, 6, &[10, 11, 12, 9], &[7, 8]);

  bq.push_front_from_copyable_data([&[13, 14][..]], ()).unwrap();
  check_state(&bq, 4, 8, &[13, 14, 10, 11, 12, 9], &[7, 8]);

  let _ = bq.pop_back();
  check_state(&bq, 3, 6, &[13, 14, 10, 11, 12, 9], &[]);

  let _ = bq.pop_back();
  check_state(&bq, 2, 5, &[13, 14, 10, 11, 12], &[]);

  let _ = bq.pop_back();
  check_state(&bq, 1, 2, &[13, 14], &[]);

  bq.push_front_from_copyable_data([&[15][..]], ()).unwrap();
  check_state(&bq, 2, 3, &[15, 13, 14], &[]);

  bq.push_front_from_copyable_data([&[16][..]], ()).unwrap();
  check_state(&bq, 3, 4, &[16, 15, 13, 14], &[]);

  let _ = bq.pop_back();
  check_state(&bq, 2, 2, &[16, 15], &[]);

  let _ = bq.pop_back();
  check_state(&bq, 1, 1, &[16], &[]);

  let _ = bq.pop_back();
  check_state(&bq, 0, 0, &[], &[]);
}

// [. . . . . . . .]: Empty - (LF=8, LO=0,RF=0, RO=0) - (H=0, T=0)
// [. . . . . H * *]: Push front - (LF=5, LO=0, RF=0, RO=3) - (H=5, T=8)
// [H * * * * * * *]: Push front - (LF=0, LO=0, RF=0, RO=8) - (H=0, T=8)
// [. . . . . H * *]: Push front - (LF=5, LO=0, RF=0, RO=3) - (H=5, T=8)
// [. . . . . . . .]: Pop back - (LF=8, LO=0, RF=0, RO=0) - (H=0, T=0)
#[test]
fn pop_front() {
  let mut bq = BlocksDeque::with_exact_capacity(2, 8).unwrap();
  check_state(&bq, 0, 0, &[], &[]);

  bq.push_front_from_copyable_data([&[1, 2, 3][..]], ()).unwrap();
  check_state(&bq, 1, 3, &[1, 2, 3], &[]);

  bq.push_front_from_copyable_data([&[4, 5], &[6, 7, 8][..]], ()).unwrap();
  check_state(&bq, 2, 8, &[4, 5, 6, 7, 8, 1, 2, 3], &[]);

  let _ = bq.pop_front();
  check_state(&bq, 1, 3, &[1, 2, 3], &[]);

  let _ = bq.pop_front();
  check_state(&bq, 0, 0, &[], &[]);
}

// []: Empty - (LF=0, LO=0,RF=0, RO=0) - (H=0, T=0)
// [H * * *]: Push front - (LF=0, LO=0, RF=0, RO=4) - (H=0, T=4)
#[test]
fn push_reserve_and_push() {
  let mut bq = BlocksDeque::new();
  bq.reserve_front(1, 4).unwrap();
  bq.push_front_from_copyable_data([&[0, 1, 2, 3][..]], ()).unwrap();
  check_state(&bq, 1, 4, &[0, 1, 2, 3], &[]);
  assert_eq!(bq.get(0), Some(BlockRef { data: &[0, 1, 2, 3], misc: &(), range: 0..4 }));
  assert_eq!(bq.get(1), None);
  bq.reserve_front(1, 6).unwrap();
  bq.push_front_from_copyable_data([&[4, 5, 6, 7, 8, 9][..]], ()).unwrap();
  check_state(&bq, 2, 10, &[4, 5, 6, 7, 8, 9, 0, 1, 2, 3], &[]);
  assert_eq!(bq.get(0), Some(BlockRef { data: &[4, 5, 6, 7, 8, 9], misc: &(), range: 0..6 }));
  assert_eq!(bq.get(1), Some(BlockRef { data: &[0, 1, 2, 3], misc: &(), range: 6..10 }));
  assert_eq!(bq.get(2), None);
}

// [. . . H * * . . ]: Pop back - (LF=5, LO=0, RF=0, RO=3) - (H=5, T=8)
// [. . . . . . . . ]: Pop back - (LF=8, LO=0, RF=0, RO=0) - (H=0, T=0)
#[test]
fn wrap_pop_back() {
  let mut bq = wrap_initial();
  let _ = bq.pop_back();
  let _ = bq.pop_back();
  check_state(&bq, 1, 3, &[1, 2, 3], &[]);
  assert_eq!(bq.get(0).unwrap().data, &[1, 2, 3]);
  let _ = bq.pop_back();
  check_state(&bq, 0, 0, &[], &[]);
}

// [. . . . . . H * ]: Pop front - (LF=2, LO=0, RF=4, RO=0) - (H=2, T=4)
// [. . . . . . . . ]: Pop front - (LF=8, LO=0, RF=0, RO=0) - (H=0, T=0)
#[test]
fn wrap_pop_front() {
  let mut bq = wrap_initial();
  let _ = bq.pop_front();
  check_state(&bq, 2, 2, &[0, 0], &[]);
  assert_eq!(bq.get(0).unwrap().data, &[0]);
  assert_eq!(bq.get(1).unwrap().data, &[0]);
  let _ = bq.pop_front();
  let _ = bq.pop_front();
  check_state(&bq, 0, 0, &[], &[]);
}

// [. . . . . . . . ]: Empty - (LF=8, LO=0, RF=0, RO=0)
// [. . H * * * * * ]: Push front - (LF=2, LO=0, RF=0, RO=6)
// [. . H * . . . . ]: Pop back - (LF=2, LO=0, RF=4, RO=0)
// [. . . H * * * * ]: Push front - (LF=3, LO=0, RF=0, RO=5)
fn wrap_initial() -> BlocksDeque<i32, ()> {
  let mut bq = BlocksDeque::with_exact_capacity(6, 8).unwrap();
  check_state(&bq, 0, 0, &[], &[]);
  for _ in 0..6 {
    bq.push_front_from_copyable_data([&[0][..]], ()).unwrap();
  }
  check_state(&bq, 6, 6, &[0, 0, 0, 0, 0, 0], &[]);
  for idx in 0..6 {
    assert_eq!(bq.get(idx).unwrap().data, &[0]);
  }
  let _ = bq.pop_back();
  let _ = bq.pop_back();
  let _ = bq.pop_back();
  let _ = bq.pop_back();
  check_state(&bq, 2, 2, &[0, 0], &[]);
  assert_eq!(bq.get(0).unwrap().data, &[0]);
  assert_eq!(bq.get(1).unwrap().data, &[0]);
  bq.push_front_from_copyable_data([&[1, 2, 3][..]], ()).unwrap();
  check_state(&bq, 3, 5, &[1, 2, 3, 0, 0], &[]);
  assert_eq!(bq.get(0).unwrap().data, &[1, 2, 3]);
  assert_eq!(bq.get(1).unwrap().data, &[0]);
  assert_eq!(bq.get(2).unwrap().data, &[0]);
  bq
}

#[track_caller]
fn check_state(
  bq: &BlocksDeque<i32, ()>,
  blocks_len: usize,
  elements_len: usize,
  front: &[i32],
  back: &[i32],
) {
  let (local_front, local_back) = bq.as_slices();
  assert_eq!(bq.blocks_len(), blocks_len);
  assert_eq!(bq.elements_len(), elements_len);
  assert_eq!(front, local_front);
  assert_eq!(back, local_back);
}
