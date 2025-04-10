use crate::misc::{Deque, collections::deque::is_wrapping};

#[test]
fn as_slices() {
  let mut queue = Deque::with_capacity(4).unwrap();
  queue.push_front(1).unwrap();
  queue.push_back(5).unwrap();
  assert_eq!(queue.as_slices(), (&[1][..], &[5][..]));
}

#[test]
fn clear() {
  let mut queue = Deque::with_capacity(1).unwrap();
  assert_eq!(queue.len(), 0);
  queue.push_front(1).unwrap();
  assert_eq!(queue.len(), 1);
  queue.clear();
  assert_eq!(queue.len(), 0);
}

#[test]
fn get() {
  let mut queue = Deque::with_capacity(1).unwrap();
  assert_eq!(queue.get(0), None);
  assert_eq!(queue.get_mut(0), None);
  queue.push_front(1).unwrap();
  assert_eq!(queue.get(0), Some(&1i32));
  assert_eq!(queue.get_mut(0), Some(&mut 1i32));
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
    let mut queue = Deque::with_exact_capacity(5).unwrap();
    queue.push_back(1).unwrap();
    queue.push_back(1).unwrap();
    queue.push_back(1).unwrap();
    queue.push_back(1).unwrap();
    queue.push_back(1).unwrap();
    let _ = queue.pop_front().unwrap();
    let _ = queue.pop_front().unwrap();
    let _ = queue.pop_front().unwrap();
    let _ = queue.pop_front().unwrap();
    assert_eq!((queue.head, queue.tail, queue.as_slices()), (4, 0, (&[1][..], &[][..])));
  }
  // H * * * T (0-5)
  {
    let mut queue = Deque::with_exact_capacity(5).unwrap();
    queue.push_back(1).unwrap();
    queue.push_back(2).unwrap();
    queue.push_back(3).unwrap();
    queue.push_back(4).unwrap();
    queue.push_back(5).unwrap();
    assert_eq!(
      (queue.head, queue.tail, queue.as_slices()),
      (0, 0, (&[1, 2, 3, 4, 5][..], &[][..]))
    );
  }
}

#[test]
fn pop_back() {
  instances(
    |queue| {
      let _ = queue.pop_back().unwrap();
      (0, 0, &[], &[])
    },
    |queue| {
      let _ = queue.pop_back().unwrap();
      (4, 4, &[], &[])
    },
    |queue| {
      let _ = queue.pop_back().unwrap();
      (4, 0, &[1], &[])
    },
    |queue| {
      let _ = queue.pop_back().unwrap();
      (0, 4, &[1, 2, 3, 4], &[])
    },
    |queue| {
      let _ = queue.pop_back().unwrap();
      (4, 3, &[1], &[2, 3, 4])
    },
  );
}

#[test]
fn pop_front() {
  instances(
    |queue| {
      let _ = queue.pop_front().unwrap();
      (1, 1, &[], &[])
    },
    |queue| {
      let _ = queue.pop_front().unwrap();
      (0, 0, &[], &[])
    },
    |queue| {
      let _ = queue.pop_front().unwrap();
      (0, 1, &[2], &[])
    },
    |queue| {
      let _ = queue.pop_front().unwrap();
      (1, 0, &[2, 3, 4, 5], &[])
    },
    |queue| {
      let _ = queue.pop_front().unwrap();
      (0, 4, &[2, 3, 4, 5], &[])
    },
  );
}

#[test]
fn push_front() {
  let mut queue = Deque::with_capacity(1).unwrap();
  assert_eq!(queue.len(), 0);
  queue.push_front(1).unwrap();
  assert_eq!(queue.len(), 1);
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
  let mut queue = Deque::<u8>::new();
  assert_eq!(queue.capacity(), 0);
  let _ = queue.reserve_back(10).unwrap();
  assert!(queue.capacity() >= 10);
  let _ = queue.reserve_front(20).unwrap();
  assert!(queue.capacity() >= 20);
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
    let mut queue = Deque::with_exact_capacity(5).unwrap();
    queue.push_back(1).unwrap();
    let (head, tail, front, back) = single_begin(&mut queue);
    verify_instance(&queue, head, tail, front, back);
  }
  // . . . . H (4-0)
  {
    let mut queue = Deque::with_exact_capacity(5).unwrap();
    queue.push_front(1).unwrap();
    let (head, tail, front, back) = single_end(&mut queue);
    verify_instance(&queue, head, tail, front, back);
  }
  // T . . . H (4-1)
  {
    let mut queue = Deque::with_exact_capacity(5).unwrap();
    queue.push_back(2).unwrap();
    queue.push_front(1).unwrap();
    let (head, tail, front, back) = single_both_sides(&mut queue);
    verify_instance(&queue, head, tail, front, back);
    // |_| (4, 1, &[1][..], &[2][..]),
  }
  // H * * * T (0-0)
  {
    let mut queue = Deque::with_exact_capacity(5).unwrap();
    queue.push_front(5).unwrap();
    queue.push_front(4).unwrap();
    queue.push_front(3).unwrap();
    queue.push_front(2).unwrap();
    queue.push_front(1).unwrap();
    let _ = queue.pop_front().unwrap();
    queue.push_back(1).unwrap();
    let _ = queue.pop_back().unwrap();
    queue.push_front(1).unwrap();
    let (head, tail, front, back) = full_begin(&mut queue);
    verify_instance(&queue, head, tail, front, back);
  }
  // * * * T H (4-4)
  {
    let mut queue = Deque::with_exact_capacity(5).unwrap();
    queue.push_front(1).unwrap();
    queue.push_back(2).unwrap();
    queue.push_back(3).unwrap();
    queue.push_back(4).unwrap();
    queue.push_back(5).unwrap();
    let (head, tail, front, back) = full_end(&mut queue);
    verify_instance(&queue, head, tail, front, back);
  }
}

#[track_caller]
fn verify_instance(queue: &Deque<i32>, head: usize, tail: usize, front: &[i32], back: &[i32]) {
  assert_eq!((queue.head, queue.tail, queue.as_slices()), (head, tail, (front, back)));
  assert_eq!(queue.len(), front.len() + back.len());
  if is_wrapping(queue.head, queue.data.len(), queue.tail) {
    assert!(!front.is_empty());
  } else {
    assert!(back.is_empty());
  }
}
