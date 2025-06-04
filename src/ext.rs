// //! Extension traits
// //!
// //! This module defines extension traits. These trait are intended to provide
// //! more ergonomic usage of the base traits.
// 
// use crate::{private, Sleep};
// use std::ops::AddAssign;
// 
// pub trait SleepExt<D>: Sleep<D> + private::SchedulerExtSealed {
//   fn sleep_until(&self, duration: D) -> Self::Timer
//   where
//     Self::Instant: AddAssign<D>;
// }
// 
// impl<T> private::SchedulerExtSealed for T where T: Sleep {}
// 
// impl<T> SchedulerExt for T
// where
//   T: Sleep,
// {
//   fn sleep<D>(&self, duration: D) -> Self::Timer
//   where
//     Self::Instant: AddAssign<D>,
//   {
//     let mut deadline: Self::Instant = self.now();
//     deadline += duration;
//     self.sleep(deadline)
//   }
// }
