use crate::{
  collections::{DequeUsize, backward_deque_idx},
  misc::waiter::Waiter,
};
use core::{mem, task::Waker};

#[derive(Debug)]
pub(crate) struct Waiters {
  deque: DequeUsize<Waiter>,
  newest_idx: usize,
  waiting_count: usize,
  woken_count: usize,
}

impl Waiters {
  #[inline(always)]
  pub(crate) const fn new() -> Self {
    Self { deque: DequeUsize::new(), newest_idx: 0, waiting_count: 0, woken_count: 0 }
  }

  #[cfg(test)]
  #[inline(always)]
  pub(crate) fn len(&self) -> usize {
    self.deque.len()
  }

  /// Number of elements that are waiting to be awakened.
  #[inline(always)]
  pub(crate) const fn waiting_count(&self) -> usize {
    self.waiting_count
  }

  /// Registers `waker`, re-using the entry referenced by `idx_opt` if any.
  ///
  /// Returns `true` if the number of waiting elements went from zero to one.
  #[inline(always)]
  pub(crate) fn register(&mut self, idx_opt: &mut Option<usize>, waker: &Waker) -> bool {
    let prev_waiting_count = self.waiting_count;
    if let Some(idx) = *idx_opt
      && let local_idx = backward_deque_idx(idx, self.newest_idx)
      && let Some(waiter) = self.deque.get_mut(local_idx)
    {
      waiter.register(&mut self.waiting_count, &mut self.woken_count, waker);
      return prev_waiting_count == 0 && self.waiting_count > 0;
    }
    let _rslt = self.deque.push_front(Waiter::Waiting(waker.clone()));
    self.newest_idx = self.newest_idx.wrapping_add(1);
    *idx_opt = Some(self.newest_idx);
    self.waiting_count = self.waiting_count.wrapping_add(1);
    prev_waiting_count == 0
  }

  /// Removes the entry referenced by `idx_opt`, returning its previous state.
  ///
  /// `idx_opt` is always set to `None`.
  #[inline(always)]
  pub(crate) fn remove(&mut self, idx_opt: &mut Option<usize>) -> Option<Waiter> {
    let idx = idx_opt.take()?;
    let actual_idx = backward_deque_idx(idx, self.newest_idx);
    let waiter = self.deque.get_mut(actual_idx)?;
    let prev = mem::replace(waiter, Waiter::Removed);
    match prev {
      Waiter::Removed => {}
      Waiter::Waiting(_) => self.waiting_count = self.waiting_count.wrapping_sub(1),
      Waiter::Woken => self.woken_count = self.woken_count.wrapping_sub(1),
    }
    if self.waiting_count == 0 && self.woken_count == 0 {
      self.deque.clear();
    } else {
      while matches!(self.deque.last(), Some(Waiter::Removed)) {
        drop(self.deque.pop_back());
      }
    }
    Some(prev)
  }

  /// Marks the oldest waiting element as woken and returns its waker, as long as the number of
  /// woken elements that weren't yet removed or re-registered is less than `max_woken`.
  ///
  /// `max_woken` should be the number of resources that are currently available, which avoids
  /// spurious wake-ups.
  #[inline(always)]
  pub(crate) fn wake(&mut self, max_woken: usize) -> Option<Waker> {
    if self.waiting_count == 0 || self.woken_count >= max_woken {
      return None;
    }
    let mut actual_idx = self.deque.len();
    loop {
      actual_idx = actual_idx.checked_sub(1)?;
      let is_last = actual_idx.wrapping_add(1) == self.deque.len();
      let waiter = self.deque.get_mut(actual_idx)?;
      match mem::replace(waiter, Waiter::Woken) {
        Waiter::Removed => {
          if is_last {
            drop(self.deque.pop_back());
          } else {
            *waiter = Waiter::Removed;
          }
        }
        Waiter::Waiting(waker) => {
          self.waiting_count = self.waiting_count.wrapping_sub(1);
          self.woken_count = self.woken_count.wrapping_add(1);
          return Some(waker);
        }
        Waiter::Woken => {}
      }
    }
  }
}
