//! Extension traits
//!
//! This module defines extension traits. These trait are intended to provide
//! more ergonomic usage of the base traits.

use crate::{private, Clock, Duration};
use std::ops::Sub;

pub trait ClockExt: Clock + private::SchedulerExtSealed {
  /// Returns the amount of time elapsed from `earlier` to now, or zero duration
  /// if that instant is later than this one.
  fn saturating_duration_since(&self, earlier: Self::Instant) -> <Self::Instant as Sub<Self::Instant>>::Output
  where
    Self::Instant: Sub<Self::Instant, Output: Duration>;
}

impl<T> private::SchedulerExtSealed for T where T: ?Sized + Clock {}

impl<T> ClockExt for T
where
  T: ?Sized + Clock,
{
  fn saturating_duration_since(&self, earlier: Self::Instant) -> <Self::Instant as Sub<Self::Instant>>::Output
  where
    Self::Instant: Sub<Self::Instant, Output: Duration>,
  {
    let now = self.now();
    if now <= earlier {
      return <<Self::Instant as Sub<_>>::Output as Duration>::zero();
    }
    let duration = now - earlier;
    let zero = <<Self::Instant as Sub<_>>::Output as Duration>::zero();
    if duration <= zero {
      return zero;
    }
    duration
  }
}
