use crate::{Clock, Scheduler};
use ::chrono04::{DateTime, Utc};
use ::tokio1::time::{sleep_until, Instant};
use crate::private::chrono::chrono_to_std;

/// Tokio scheduler with chrono types using the system time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SystemTokio1Chrono04Clock;

impl Clock for SystemTokio1Chrono04Clock {
  type Instant = DateTime<Utc>;

  fn now(&self) -> Self::Instant {
    Utc::now()
  }
}

/// Type alias for the timer type used by `SystemTokio1Chrono04Clock` for
/// its `Scheduler` implementation.
pub type SystemTokio1Chrono04Timer = ::tokio1::time::Sleep;

impl Scheduler for SystemTokio1Chrono04Clock {
  type Timer = SystemTokio1Chrono04Timer;

  fn schedule(&self, deadline: DateTime<Utc>) -> Self::Timer {
    sleep_until(Instant::from_std(chrono_to_std(deadline)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ChronoClock;
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
}
