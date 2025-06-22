use crate::impl_now;
use ::std::time::Instant;

/// Standard library clock using the system time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemStdClock;

impl_now! {
  impl Now for SystemStdClock {
    type Instant = Instant;

    fn now(&this)-> Self::Instant {
      Instant::now()
    }
  }
}

// TODO: Rust guarantees monotonicity, for tier 1 platforms. There should be
//       conditional implementation of `Monotonic` when detecting such platform.

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{Now, NowOnce, StdNow};
  use ::std::time::Duration;
  const ONE_YEAR: Duration = Duration::new(365 * 24 * 60 * 60, 0);

  #[test]
  fn test_now() {
    let clock = SystemStdClock;
    use_clock(&clock);
    use_std_clock(&clock);
  }

  fn use_clock<TyNow>(clock: &TyNow)
  where
    TyNow: Now<Instant = Instant>,
  {
    let one_year_ago = Instant::now() - ONE_YEAR;
    let now = clock.now_once();
    assert!(now > one_year_ago);
    use_std_clock(clock);
  }

  fn use_std_clock<TyClock>(clock: &TyClock)
  where
    TyClock: StdNow,
  {
    let one_year_ago = Instant::now() - ONE_YEAR;
    let now = clock.now_std();
    assert!(now > one_year_ago);
  }
}
