use crate::misc::Deque;
use alloc::collection::VecDeque;

#[kani::proof]
fn queue() {
  let bytes = kani::vec::any_vec::<u8, 128>();
  let mut queue = Deque::with_capacity(bytes.len()).unwrap();
  let mut vec_deque = VecDeque::with_capacity(bytes.len());

  for byte in bytes.iter().copied() {
    queue.push_front(byte).unwrap();
    vec_deque.push_front(byte);
  }
  assert_eq!((queue.capacity(), queue.len()), (vec_deque.capacity(), vec_deque.len()));
  for _ in 0..(bytes.len() / 2) {
    assert_eq!(queue.as_slices(), vec_deque.as_slices());
    assert_eq!(queue.get(0), vec_deque.get(0));
    assert_eq!(queue.get_mut(0), vec_deque.get_mut(0));
    assert_eq!(queue.pop_back(), vec_deque.pop_back());
    assert_eq!(queue.as_slices(), vec_deque.as_slices());
    assert_eq!(queue.get(0), vec_deque.get(0));
    assert_eq!(queue.get_mut(0), vec_deque.get_mut(0));
    assert_eq!(queue.pop_front(), vec_deque.pop_front());
  }
  loop {
    if queue.len() == 0 {
      break;
    }
    assert_eq!(queue.as_slices(), vec_deque.as_slices());
    assert_eq!(queue.get(0), vec_deque.get(0));
    assert_eq!(queue.get_mut(0), vec_deque.get_mut(0));
    assert_eq!(queue.pop_back(), vec_deque.pop_back());
    if queue.len() == 0 {
      break;
    }
    assert_eq!(queue.as_slices(), vec_deque.as_slices());
    assert_eq!(queue.get(0), vec_deque.get(0));
    assert_eq!(queue.get_mut(0), vec_deque.get_mut(0));
    assert_eq!(queue.pop_front(), vec_deque.pop_front());
  }
  assert_eq!((queue.capacity(), queue.len()), (vec_deque.capacity(), vec_deque.len()));
  assert_eq!((queue.len(), vec_deque.len()), (0, 0));
}
