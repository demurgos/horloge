use crate::{impl_now, impl_sleep};
use chrono04::TimeDelta;
use ::chrono04::{DateTime, Utc};
use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll, Waker};

/// Tokio clock with chrono types using virtual time.
pub struct VirtualTokio1Chrono04Clock {
  state: Arc<VirtualClockState>,
}

impl VirtualTokio1Chrono04Clock {
  pub fn new(start: DateTime<Utc>) -> Self {
    Self {
      state: Arc::new(VirtualClockState::new(start)),
    }
  }

  pub fn advance_by(&self, d: TimeDelta) -> DateTime<Utc> {
    self.state.advance_by(d)
  }

  /// Updates the clock to the supplied time, if it more recent
  ///
  /// Returns the time after the update.
  pub fn advance_to(&self, t: DateTime<Utc>) -> DateTime<Utc> {
    self.state.advance_to(t)
  }
}

pub struct VirtualClockState {
  start: DateTime<Utc>,
  offset: AtomicU64,
  timer_id: AtomicUsize,
  timers: RwLock<BTreeMap<(DateTime<Utc>, usize), Waker>>,
}

impl VirtualClockState {
  pub fn new(start: DateTime<Utc>) -> Self {
    Self {
      start,
      offset: AtomicU64::new(0),
      timer_id: AtomicUsize::new(0),
      timers: RwLock::new(BTreeMap::new()),
    }
  }

  /// Returns the time after the update
  pub fn advance_by(&self, d: TimeDelta) -> DateTime<Utc> {
    let d: i64 = match d.num_nanoseconds() {
      Some(d) => d,
      None => {
        // chrono returns `None` if there's an overflow in either direction
        // we need to compare with zero to recover the sign
        if d < TimeDelta::zero() { i64::MIN } else { i64::MAX }
      }
    };
    let d: u64 = u64::try_from(d).unwrap_or(0u64);
    let prev: u64 = self.offset.load(Ordering::SeqCst);
    let wanted_offset = prev.max(d);
    let prev: u64 = self.offset.fetch_max(wanted_offset, Ordering::SeqCst);
    let current_offset: u64 = prev.max(wanted_offset);
    let current_offset: i64 = i64::try_from(current_offset).unwrap_or(i64::MAX);
    let now = self.start + TimeDelta::nanoseconds(current_offset);
    self.signal_ready(now);
    now
  }

  /// Updates the clock to the supplied time, if it more recent
  ///
  /// Returns the time after the update.
  pub fn advance_to(&self, t: DateTime<Utc>) -> DateTime<Utc> {
    self.advance_by(t - self.start)
  }

  /// Mark all timers with a deadline equal to or earlier than `now` as ready.
  fn signal_ready(&self, now: DateTime<Utc>) {
    let timers = self.timers.read().expect("failed to lock virtual clock timers");
    let mut ready: Vec<Waker> = Vec::new();
    for ((deadline, _id), waker) in timers.iter() {
      if *deadline > now {
        break;
      }
      ready.push(waker.clone());
    }
    drop(timers);
    for waker in ready {
      waker.wake()
    }
  }

  fn now(&self) -> DateTime<Utc> {
    let offset = self.offset.load(Ordering::SeqCst);
    let offset: i64 = i64::try_from(offset).unwrap_or(i64::MAX);
    self.start + TimeDelta::nanoseconds(offset)
  }

  /// Create a new [`VirtualTimer`] key for the provided deadline.
  fn sleep(&self, duration: TimeDelta) -> (DateTime<Utc>, usize) {
    let deadline: DateTime<Utc> = self.now() + duration;
    let id = self.timer_id.fetch_add(1, Ordering::SeqCst);
    (deadline, id)
  }

  fn clear(&self, key: (DateTime<Utc>, usize)) -> bool {
    let mut timers = self.timers.write().expect("failed to lock virtual clock timers");
    timers.remove(&key).is_some()
  }

  fn clear_expired(&self, key: (DateTime<Utc>, usize), waker: &Waker) -> bool {
    let now = self.now();
    if now < key.0 {
      let mut timers = self.timers.write().expect("failed to lock virtual clock timers");
      timers.entry(key).or_insert_with(|| waker.clone());
      false
    } else {
      self.clear(key);
      true
    }
  }
}

impl_now! {
  impl Now for VirtualTokio1Chrono04Clock {
    type Instant = DateTime<Utc>;

    fn now(&this)-> Self::Instant {
      this.state.now()
    }
  }
}

pub struct VirtualTimer {
  key: (DateTime<Utc>, usize),
  state: Arc<VirtualClockState>,
}

impl Drop for VirtualTimer {
  fn drop(&mut self) {
    self.state.clear(self.key);
  }
}

impl Future for VirtualTimer {
  type Output = ();

  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let state = &*self.state;
    let key = self.key;
    if state.clear_expired(key, cx.waker()) {
      Poll::Ready(())
    } else {
      Poll::Pending
    }
  }
}

impl_sleep! {
  impl Sleep<TimeDelta> for VirtualTokio1Chrono04Clock {
    type Timer = VirtualTimer;

    fn sleep(&this, duration: TimeDelta) -> Self::Timer {
      let state = Arc::clone(&this.state);
      let key = state.sleep(duration);
      VirtualTimer { key, state }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{Chrono04Now, Now};
  use ::chrono04::TimeDelta;
  use chrono04::TimeZone;
  use std::sync::LazyLock;

  const START: LazyLock<DateTime<Utc>> = LazyLock::new(|| {
    Utc
      .with_ymd_and_hms(2025, 1, 1, 0, 0, 0)
      .earliest()
      .expect("start time for tests is valid")
  });
  const ONE_YEAR: TimeDelta = TimeDelta::new(365 * 24 * 60 * 60, 0).expect("constant time delta is valid");

  #[test]
  fn test_now() {
    let start = *START;
    let clock = VirtualTokio1Chrono04Clock::new(start);
    use_clock(&clock);
    use_chrono_clock(&clock);
  }

  fn use_clock<TyNow>(clock: &TyNow)
  where
    TyNow: Now<Instant = DateTime<Utc>>,
  {
    let one_year_ago = Utc::now() - ONE_YEAR;
    let now: DateTime<Utc> = clock.now();
    assert!(now > one_year_ago);
    use_chrono_clock(clock);
  }

  fn use_chrono_clock<TyNow>(clock: &TyNow)
  where
    TyNow: Chrono04Now,
  {
    let one_year_ago = Utc::now() - ONE_YEAR;
    let now: DateTime<Utc> = clock.now_chrono();
    assert!(now > one_year_ago);
  }
}
