mod extend_front_from_copyable_within;

use crate::collection::{Deque, deque::is_wrapping};

#[test]
fn as_slices() {
  assert_eq!(no_wrapping_00().as_slices(), (&[][..], &[][..]));
  assert_eq!(no_wrapping_11().as_slices(), (&[][..], &[][..]));

  assert_eq!(no_wrapping_01().as_slices(), (&[1][..], &[][..]));
  assert_eq!(no_wrapping_02().as_slices(), (&[1, 2][..], &[][..]));
  assert_eq!(no_wrapping_36().as_slices(), (&[4, 5, 6][..], &[][..]));

  assert_eq!(wrapping_00().as_slices(), (&[1, 2, 3, 4, 5, 6, 7, 8][..], &[][..]));
  assert_eq!(wrapping_22().as_slices(), (&[1, 2, 3, 4, 5, 6][..], &[7, 8][..]));
  assert_eq!(wrapping_77().as_slices(), (&[1][..], &[2, 3, 4, 5, 6, 7, 8][..]));

  assert_eq!(wrapping_60().as_slices(), (&[7, 8][..], &[][..]));
  assert_eq!(wrapping_64().as_slices(), (&[1, 2][..], &[3, 4, 5, 6][..]));
  assert_eq!(wrapping_70().as_slices(), (&[1][..], &[][..]));
  assert_eq!(wrapping_71().as_slices(), (&[1][..], &[2][..]));
}

#[test]
fn clear() {
  let mut deque = Deque::with_capacity(1).unwrap();
  assert_eq!(deque.len(), 0);
  deque.push_front(1).unwrap();
  assert_eq!(deque.len(), 1);
  deque.clear();
  assert_eq!(deque.len(), 0);
}

#[test]
fn get() {
  let mut deque = Deque::with_capacity(1).unwrap();
  assert_eq!(deque.get(0), None);
  assert_eq!(deque.get_mut(0), None);
  deque.push_front(1).unwrap();
  assert_eq!(deque.get(0), Some(&1i32));
  assert_eq!(deque.get_mut(0), Some(&mut 1i32));
}

#[test]
fn heads_tails_and_slices() {
  instances(
    |_| (0, 1, &[1][..], &[][..]),
    |_| (4, 0, &[1][..], &[][..]),
    |_| (4, 1, &[1][..], &[2][..]),
    |_| (0, 0, &[1, 2, 3, 4, 5][..], &[][..]),
    |_| (4, 4, &[1][..], &[2, 3, 4, 5][..]),
  );
}

#[test]
fn impossible_instances() {
  // . . . . H (4-5)
  {
    let mut deque = Deque::with_exact_capacity(5).unwrap();
    deque.push_back(1).unwrap();
    deque.push_back(1).unwrap();
    deque.push_back(1).unwrap();
    deque.push_back(1).unwrap();
    deque.push_back(1).unwrap();
    let _ = deque.pop_front().unwrap();
    let _ = deque.pop_front().unwrap();
    let _ = deque.pop_front().unwrap();
    let _ = deque.pop_front().unwrap();
    assert_eq!((deque.head, deque.tail, deque.as_slices()), (4, 0, (&[1][..], &[][..])));
  }
  // H * * * T (0-5)
  {
    let mut deque = Deque::with_exact_capacity(5).unwrap();
    deque.push_back(1).unwrap();
    deque.push_back(2).unwrap();
    deque.push_back(3).unwrap();
    deque.push_back(4).unwrap();
    deque.push_back(5).unwrap();
    assert_eq!(
      (deque.head, deque.tail, deque.as_slices()),
      (0, 0, (&[1, 2, 3, 4, 5][..], &[][..]))
    );
  }
}

#[test]
fn pop_back() {
  instances(
    |deque| {
      let _ = deque.pop_back().unwrap();
      (0, 0, &[], &[])
    },
    |deque| {
      let _ = deque.pop_back().unwrap();
      (4, 4, &[], &[])
    },
    |deque| {
      let _ = deque.pop_back().unwrap();
      (4, 0, &[1], &[])
    },
    |deque| {
      let _ = deque.pop_back().unwrap();
      (0, 4, &[1, 2, 3, 4], &[])
    },
    |deque| {
      let _ = deque.pop_back().unwrap();
      (4, 3, &[1], &[2, 3, 4])
    },
  );
}

#[test]
fn pop_front() {
  instances(
    |deque| {
      let _ = deque.pop_front().unwrap();
      (1, 1, &[], &[])
    },
    |deque| {
      let _ = deque.pop_front().unwrap();
      (0, 0, &[], &[])
    },
    |deque| {
      let _ = deque.pop_front().unwrap();
      (0, 1, &[2], &[])
    },
    |deque| {
      let _ = deque.pop_front().unwrap();
      (1, 0, &[2, 3, 4, 5], &[])
    },
    |deque| {
      let _ = deque.pop_front().unwrap();
      (0, 4, &[2, 3, 4, 5], &[])
    },
  );
}

#[test]
fn push_front() {
  let mut deque = Deque::with_capacity(1).unwrap();
  assert_eq!(deque.len(), 0);
  deque.push_front(1).unwrap();
  assert_eq!(deque.len(), 1);
}

#[test]
fn push_when_full() {
  let mut bq = Deque::with_capacity(5).unwrap();
  bq.push_front(0).unwrap();
  bq.push_front(1).unwrap();
  bq.push_front(2).unwrap();
  bq.push_front(3).unwrap();
  bq.push_front(4).unwrap();
  let _ = bq.pop_back();
  let _ = bq.pop_back();
  bq.push_front(5).unwrap();
  bq.push_front(6).unwrap();
  assert_eq!(bq.as_slices(), (&[6, 5][..], &[4, 3, 2][..]));
}

#[test]
fn reserve() {
  let mut deque = Deque::<u8>::new();
  assert_eq!(deque.capacity(), 0);
  let _ = deque.reserve_back(10).unwrap();
  assert!(deque.capacity() >= 10);
  let _ = deque.reserve_front(20).unwrap();
  assert!(deque.capacity() >= 20);
}

fn instances(
  single_begin: impl FnOnce(&mut Deque<i32>) -> (usize, usize, &'static [i32], &'static [i32]),
  single_end: impl FnOnce(&mut Deque<i32>) -> (usize, usize, &'static [i32], &'static [i32]),
  single_both_sides: impl FnOnce(&mut Deque<i32>) -> (usize, usize, &'static [i32], &'static [i32]),
  full_begin: impl FnOnce(&mut Deque<i32>) -> (usize, usize, &'static [i32], &'static [i32]),
  full_end: impl FnOnce(&mut Deque<i32>) -> (usize, usize, &'static [i32], &'static [i32]),
) {
  // H . . . . (0-1)
  {
    let mut deque = Deque::with_exact_capacity(5).unwrap();
    deque.push_back(1).unwrap();
    let (head, tail, front, back) = single_begin(&mut deque);
    verify_instance(&deque, head, tail, front, back);
  }
  // . . . . H (4-0)
  {
    let mut deque = Deque::with_exact_capacity(5).unwrap();
    deque.push_front(1).unwrap();
    let (head, tail, front, back) = single_end(&mut deque);
    verify_instance(&deque, head, tail, front, back);
  }
  // T . . . H (4-1)
  {
    let mut deque = Deque::with_exact_capacity(5).unwrap();
    deque.push_back(2).unwrap();
    deque.push_front(1).unwrap();
    let (head, tail, front, back) = single_both_sides(&mut deque);
    verify_instance(&deque, head, tail, front, back);
    // |_| (4, 1, &[1][..], &[2][..]),
  }
  // H * * * T (0-0)
  {
    let mut deque = Deque::with_exact_capacity(5).unwrap();
    deque.push_front(5).unwrap();
    deque.push_front(4).unwrap();
    deque.push_front(3).unwrap();
    deque.push_front(2).unwrap();
    deque.push_front(1).unwrap();
    let _ = deque.pop_front().unwrap();
    deque.push_back(1).unwrap();
    let _ = deque.pop_back().unwrap();
    deque.push_front(1).unwrap();
    let (head, tail, front, back) = full_begin(&mut deque);
    verify_instance(&deque, head, tail, front, back);
  }
  // * * * T H (4-4)
  {
    let mut deque = Deque::with_exact_capacity(5).unwrap();
    deque.push_front(1).unwrap();
    deque.push_back(2).unwrap();
    deque.push_back(3).unwrap();
    deque.push_back(4).unwrap();
    deque.push_back(5).unwrap();
    let (head, tail, front, back) = full_end(&mut deque);
    verify_instance(&deque, head, tail, front, back);
  }
}

#[track_caller]
fn verify_instance(deque: &Deque<i32>, head: usize, tail: usize, front: &[i32], back: &[i32]) {
  assert_eq!((deque.head, deque.tail, deque.as_slices()), (head, tail, (front, back)));
  assert_eq!(deque.len(), front.len() + back.len());
  if is_wrapping(deque.head, deque.data.len(), deque.tail) {
    assert!(!front.is_empty());
  } else {
    assert!(back.is_empty());
  }
}

fn pop_front_n(deque: &mut Deque<i32>, n: i32) {
  for _ in 0..n {
    let _ = deque.pop_front().unwrap();
  }
}

fn push_back_n(deque: &mut Deque<i32>, n: i32) {
  for idx in 1..=n {
    deque.push_back(idx).unwrap();
  }
}

/// H(0) = T(0): . . . . . . . . (no wrapping)
fn no_wrapping_00() -> Deque<i32> {
  let deque = Deque::with_capacity(8).unwrap();
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (0, 0, 0, false));
  deque
}

/// H(0) < T(1): H . . . . . . . (no wrapping)
fn no_wrapping_01() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  push_back_n(&mut deque, 1);
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (0, 1, 1, false));
  deque
}

/// H(0) < T(2): H T . . . . . . (no wrapping)
fn no_wrapping_02() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  push_back_n(&mut deque, 2);
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (0, 2, 2, false));
  deque
}

/// H(1) = T(1): . . . . . . . . (no wrapping)
fn no_wrapping_11() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  deque.push_back(1).unwrap();
  let _ = deque.pop_front().unwrap();
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (1, 1, 0, false));
  deque
}

/// H(3) < T(6): . . . H * T . . (no wrapping)
fn no_wrapping_36() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  push_back_n(&mut deque, 6);
  pop_front_n(&mut deque, 3);
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (3, 6, 3, false));
  deque
}

#[test]
fn truncate_back() {
  let mut deque = Deque::with_exact_capacity(4).unwrap();
  deque.push_front(1).unwrap();
  deque.push_back(2).unwrap();
  deque.push_back(3).unwrap();
  deque.truncate_back(1);
  deque.push_back(99).unwrap();
  let (front, back) = deque.as_slices();
  assert_eq!(front, &[1]);
  assert_eq!(back, &[99]);
}

/// H(0) = T(0): H * * * * * * T (wrapping)
fn wrapping_00() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  push_back_n(&mut deque, 8);
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (0, 0, 8, true));
  deque
}

/// H(2) = T(2): * T H * * * * * (wrapping)
fn wrapping_22() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  deque.push_front(6).unwrap();
  deque.push_front(5).unwrap();
  deque.push_front(4).unwrap();
  deque.push_front(3).unwrap();
  deque.push_front(2).unwrap();
  deque.push_front(1).unwrap();
  deque.push_back(7).unwrap();
  deque.push_back(8).unwrap();
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (2, 2, 8, true));
  deque
}

/// H(6) < T(0): . . . . . H * T (wrapping)
fn wrapping_60() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  push_back_n(&mut deque, 8);
  pop_front_n(&mut deque, 6);
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (6, 0, 2, true));
  deque
}

/// H(6) > T(4): * * * T . . H * (wrapping)
fn wrapping_64() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  deque.push_front(2).unwrap();
  deque.push_front(1).unwrap();
  deque.push_back(3).unwrap();
  deque.push_back(4).unwrap();
  deque.push_back(5).unwrap();
  deque.push_back(6).unwrap();
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (6, 4, 6, true));
  deque
}

/// H(7) > T(0): . . . . . . . H (wrapping)
fn wrapping_70() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  deque.push_front(1).unwrap();
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (7, 0, 1, true));
  deque
}

/// H(7) > T(1): T . . . . . . H (wrapping)
fn wrapping_71() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  deque.push_front(1).unwrap();
  deque.push_back(2).unwrap();
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (7, 1, 2, true));
  deque
}

/// H(7) = T(7): * * * * * * T H (wrapping)
fn wrapping_77() -> Deque<i32> {
  let mut deque = Deque::with_capacity(8).unwrap();
  deque.push_front(1).unwrap();
  deque.push_back(2).unwrap();
  deque.push_back(3).unwrap();
  deque.push_back(4).unwrap();
  deque.push_back(5).unwrap();
  deque.push_back(6).unwrap();
  deque.push_back(7).unwrap();
  deque.push_back(8).unwrap();
  assert_eq!((deque.head, deque.tail, deque.len(), deque.is_wrapping()), (7, 7, 8, true));
  deque
}
