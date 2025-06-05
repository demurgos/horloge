use crate::{impl_clock, impl_sleep};
use chrono04::TimeDelta;
use ::chrono04::{DateTime, Utc};
use tokio1::time::sleep;

/// Tokio scheduler with chrono types using the system time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTokio1Chrono04Clock;

impl_clock! {
  impl Clock for SystemTokio1Chrono04Clock {
    type Instant = DateTime<Utc>;

    fn now(&this)-> Self::Instant {
      Utc::now()
    }
  }
}

/// Type alias for the timer type used by `SystemTokio1Chrono04Clock` for
/// its `Scheduler` implementation.
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
  use crate::{ChronoClock, Clock, ErasedChronoScheduler};
  use ::chrono04::TimeDelta;
  const ONE_YEAR: TimeDelta = TimeDelta::new(365 * 24 * 60 * 60, 0).expect("constant time delta is valid");

  #[test]
  fn test_now() {
    let clock = SystemTokio1Chrono04Clock;
    use_clock(&clock);
    use_chrono_clock(&clock);
  }

  fn use_clock<TyClock>(clock: &TyClock)
  where
    TyClock: Clock<Instant = DateTime<Utc>>,
  {
    let one_year_ago = Utc::now() - ONE_YEAR;
    let now: DateTime<Utc> = clock.now();
    assert!(now > one_year_ago);
    use_chrono_clock(clock);
  }

  fn use_chrono_clock<TyClock>(clock: &TyClock)
  where
    TyClock: ChronoClock,
  {
    let one_year_ago = Utc::now() - ONE_YEAR;
    let now: DateTime<Utc> = clock.now_chrono();
    assert!(now > one_year_ago);
  }

  fn use_dyn(scheduler: Box<dyn ErasedChronoScheduler>)
  {
    scheduler.now_chrono();
  }
}
