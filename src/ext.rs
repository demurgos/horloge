//! Extension traits
//!
//! This module defines extension traits. These trait are intended to provide
//! more ergonomic usage of the base traits.

use crate::{private, Scheduler};
use std::ops::AddAssign;

pub trait SchedulerExt: Scheduler + private::SchedulerExtSealed {
  fn sleep<D>(&self, duration: D) -> Self::Timer
  where
    Self::Instant: AddAssign<D>;
}

impl<T> private::SchedulerExtSealed for T where T: Scheduler {}

impl<T> SchedulerExt for T
where
  T: Scheduler,
{
  fn sleep<D>(&self, duration: D) -> Self::Timer
  where
    Self::Instant: AddAssign<D>,
  {
    let mut deadline: Self::Instant = self.now();
    deadline += duration;
    self.schedule(deadline)
  }
}
