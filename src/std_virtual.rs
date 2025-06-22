use crate::{impl_now, Monotonic};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Monotic virtual clock using the standard [`Instant`] type.
///
/// The clock is monotonic. It can cover a period of `u64::MAX`
/// nanoseconds (about 584 years).
///
/// If trying to exceed the maximum supported time, the clock
/// will silently saturate.
///
/// The main use case of this clock is to mock time during
/// during tests.
///
/// This clock is internally backed by an atomic u64. References
/// can be shared between threads.
pub struct VirtualStdClock {
  /// Start time of the clock
  start: Instant,
  /// Offset in nanoseconds
  offset: AtomicU64,
}

impl VirtualStdClock {
  /// Create a new virtual clock starting at the provided time.
  pub fn new(start: Instant) -> Self {
    Self {
      start,
      offset: AtomicU64::new(0),
    }
  }

  /// Add the provided duration to the clock.
  ///
  /// Returns the time after the update.
  pub fn advance_by(&self, d: Duration) -> Instant {
    let d: u128 = d.as_nanos();
    let d: u64 = u64::try_from(d).unwrap_or(u64::MAX);
    let prev: u64 = self.offset.load(Ordering::SeqCst);
    let wanted_offset = prev.max(d);
    let prev: u64 = self.offset.fetch_max(wanted_offset, Ordering::SeqCst);
    let current_offset = prev.max(wanted_offset);
    self.start + Duration::from_nanos(current_offset)
  }

  /// Updates the clock to the provided time, if it more recent than the
  /// current time.
  ///
  /// Returns the time after the update.
  pub fn advance_to(&self, t: Instant) -> Instant {
    let wanted_offset: u128 = match t.checked_duration_since(self.start) {
      Some(d) => d.as_nanos(),
      None => 0u128,
    };
    let wanted_offset: u64 = u64::try_from(wanted_offset).unwrap_or(u64::MAX);
    let prev: u64 = self.offset.fetch_max(wanted_offset, Ordering::SeqCst);
    let current_offset = prev.max(wanted_offset);
    self.start + Duration::from_nanos(current_offset)
  }
}

impl_now! {
  impl Now for VirtualStdClock {
    type Instant = Instant;

    fn now(&this) -> Self::Instant {
      let current_offset = this.offset.load(Ordering::SeqCst);
      this.start + Duration::from_nanos(current_offset)
    }
  }
}

// SAFETY:
// The monotonicity of `VirtualStdClock::now` depends on the monotonicity
// of `VirtualStdClock::offset`. `VirtualStdClock::offset` is only updated by
// using `u64::fetch_max` to ensure monotonicity.
unsafe impl Monotonic for VirtualStdClock {}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{Now, StdNow};
  const ONE_YEAR: Duration = Duration::new(365 * 24 * 60 * 60, 0);

  #[test]
  fn test_now() {
    let start = Instant::now();
    let clock = VirtualStdClock::new(start);
    use_clock(&clock);
    use_std_clock(&clock);
  }

  fn use_clock<TyNow>(clock: &TyNow)
  where
    TyNow: Now<Instant = Instant> + Monotonic,
  {
    let one_year_ago = Instant::now() - ONE_YEAR;
    let now = clock.now();
    assert!(now > one_year_ago);
    use_std_clock(clock);
  }

  fn use_std_clock<TyNow>(clock: &TyNow)
  where
    TyNow: StdNow + Monotonic,
  {
    let one_year_ago = Instant::now() - ONE_YEAR;
    let now = clock.now_std();
    assert!(now > one_year_ago);
  }

  #[test]
  fn test_advance_by() {
    let start = Instant::now();
    let clock = VirtualStdClock::new(start);
    assert_eq!(clock.now(), start);
    clock.advance_by(ONE_YEAR);
    assert_eq!(clock.now(), start + ONE_YEAR);
    assert_eq!(clock.now(), start + ONE_YEAR);
  }

  #[test]
  fn test_advance_to() {
    let start = Instant::now();
    let clock = VirtualStdClock::new(start);
    assert_eq!(clock.now(), start);
    clock.advance_to(start + 2 * ONE_YEAR);
    assert_eq!(clock.now(), start + 2 * ONE_YEAR);
    assert_eq!(clock.now(), start + 2 * ONE_YEAR);
  }

  #[test]
  fn test_monotonic() {
    let start = Instant::now();
    let clock = VirtualStdClock::new(start);
    assert_eq!(clock.now(), start);
    clock.advance_to(start + 2 * ONE_YEAR);
    assert_eq!(clock.now(), start + 2 * ONE_YEAR);
    clock.advance_to(start + ONE_YEAR);
    assert_eq!(clock.now(), start + 2 * ONE_YEAR);
  }
}
