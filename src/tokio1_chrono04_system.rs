use crate::{impl_now, impl_sleep};
use chrono04::TimeDelta;
use ::chrono04::{DateTime, Utc};
use tokio1::time::sleep;

/// Tokio clock with chrono types using the system time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTokio1Chrono04Clock;

impl_now! {
  impl Now for SystemTokio1Chrono04Clock {
    type Instant = DateTime<Utc>;

    fn now(&this)-> Self::Instant {
      Utc::now()
    }
  }
}

/// Type alias for the timer type used by `SystemTokio1Chrono04Clock` for
/// its `Clock` implementation.
pub type SystemTokio1Chrono04Timer = ::tokio1::time::Sleep;

impl_sleep! {
  impl Sleep<TimeDelta> for SystemTokio1Chrono04Clock {
    type Timer = SystemTokio1Chrono04Timer;

    fn sleep(&this, duration: TimeDelta) -> Self::Timer {
      sleep(duration.to_std().unwrap())
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{Chrono04Clock, Chrono04Now, ErasedChrono04Clock, Now, NowExt};
  use ::chrono04::TimeDelta;
  const ONE_YEAR: TimeDelta = TimeDelta::new(365 * 24 * 60 * 60, 0).expect("constant time delta is valid");

  #[test]
  fn test_now() {
    let clock = SystemTokio1Chrono04Clock;
    use_now(&clock);
    use_chrono04_now(&clock);
    use_chrono04_clock(&clock);
    use_dyn_now(Box::new(clock));
  }

  fn use_now<TyNow>(clock: &TyNow)
  where
    TyNow: Now<Instant = DateTime<Utc>>,
  {
    let one_year_ago = Utc::now() - ONE_YEAR;
    let now: DateTime<Utc> = clock.now();
    assert!(now > one_year_ago);
    use_chrono04_now(clock);
  }

  fn use_chrono04_now<TyNow>(clock: &TyNow)
  where
    TyNow: Chrono04Now,
  {
    let one_year_ago = Utc::now() - ONE_YEAR;
    let now: DateTime<Utc> = clock.now_chrono();
    assert!(now > one_year_ago);
  }

  fn use_dyn_now(clock: Box<dyn ErasedChrono04Clock>) {
    let now = clock.now_chrono();
    let elapsed = clock.saturating_duration_since(now);
    assert!(elapsed >= TimeDelta::zero());
  }

  fn use_chrono04_clock<TyClock>(clock: &TyClock)
  where
    TyClock: Chrono04Clock,
  {
    let one_year_ago = Utc::now() - ONE_YEAR;
    let now: DateTime<Utc> = clock.now_chrono();
    assert!(now > one_year_ago);
  }
}
