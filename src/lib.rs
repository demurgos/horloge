#[cfg(feature = "std")]
mod std_system;
#[cfg(feature = "std")]
mod std_virtual;
#[cfg(all(feature = "std", feature = "chrono04", feature = "tokio1"))]
mod tokio1_chrono04_system;
#[cfg(all(feature = "std", feature = "chrono04", feature = "tokio1"))]
mod tokio1_chrono04_virtual;

mod private;

#[cfg(feature = "std")]
pub use std_system::*;
#[cfg(feature = "std")]
pub use std_virtual::*;
#[cfg(all(feature = "std", feature = "chrono04", feature = "tokio1"))]
pub use tokio1_chrono04_system::*;
#[cfg(all(feature = "std", feature = "chrono04", feature = "tokio1"))]
pub use tokio1_chrono04_virtual::*;

/// Trait for a source of the current time, as an abstract point in time.
///
/// This trait exposes a single method returning the current point in time.
/// This is a very generic trait. If you can make additional assumptions about
/// the time library you use, it is recommended to use a more specific trait
/// such as `StdClock`.
pub trait Clock {
  /// Type used to represent a point in time.
  type Instant: PartialOrd;

  /// Returns a value representing the current point in time.
  fn now(&self) -> Self::Instant;
}

/// Marker trait indicating that a clock is monotonic.
///
/// A monotonic clock is a clock where calls to [`Clock::now`] always returns
/// a value equal or strictly larger than all previously returned values.
///
/// # Safety
///
/// This trait should only be implemented if the [`Clock::now`] verifies
/// the monotonic property. If two calls `let first = Clock::now()` and
/// `let second = Clock::now()` can be ordered with a "happens before"
/// relationship and `Clock::Instant` defines a partial order, then
/// `first <= second`.
pub unsafe trait Monotonic: Clock {}

/// Trait for a scheduler.
///
/// The scheduler returns a timer implementing the `Future` trait. The future
/// resolves when the `Scheduler::now()` is larger than `deadline`.
///
/// The timer can be canceled by dropping the value.
pub trait Scheduler: Clock {
  type Timer: Future<Output = ()> + Send;

  fn schedule(&self, deadline: <Self as Clock>::Instant) -> Self::Timer;
}

/// Dyn-compatible version of [`Scheduler`].
pub trait ErasedScheduler: Clock {
  fn erased_schedule(&self, deadline: <Self as Clock>::Instant) -> Box<dyn Future<Output = ()>>;
}

impl<T> ErasedScheduler for T
where
  T: Scheduler,
  T::Timer: Send + Sync + 'static,
{
  fn erased_schedule(&self, deadline: <Self as Clock>::Instant) -> Box<dyn Future<Output = ()>> {
    Box::new(self.schedule(deadline))
  }
}

/// Trait for a source of the current time, as a standard [`Instant`](::std::time::Instant).
///
/// This trait is automatically implemented for values implementing
/// [`Clock<Instant = ::std::time::Instant>`](Clock).
#[cfg(feature = "std")]
pub trait StdClock {
  fn now_std(&self) -> ::std::time::Instant;
}

#[cfg(feature = "std")]
impl<T> StdClock for T
where
  T: Clock<Instant = ::std::time::Instant>,
{
  fn now_std(&self) -> ::std::time::Instant {
    self.now()
  }
}

/// Alias trait for `ChronoClock + Scheduler`
#[cfg(feature = "std")]
pub trait StdScheduler: StdClock + Scheduler {}

#[cfg(feature = "std")]
impl<T> StdScheduler for T where T: StdClock + Scheduler {}

/// Alias trait for `ChronoClock + ErasedScheduler`
#[cfg(feature = "std")]
pub trait ErasedStdScheduler: StdClock + ErasedScheduler {}

#[cfg(feature = "std")]
impl<T> ErasedStdScheduler for T where T: StdClock + ErasedScheduler {}

/// Trait for a source of the current time, as a [`chrono::DateTime<Utc>`](::chrono04::DateTime).
///
/// This trait is automatically implemented for values implementing
/// [`Clock<Instant = chrono::DateTime<Utc>`](Clock).
#[cfg(feature = "chrono04")]
pub trait ChronoClock {
  fn now_chrono(&self) -> ::chrono04::DateTime<::chrono04::Utc>;
}

#[cfg(feature = "chrono04")]
impl<T> ChronoClock for T
where
  T: Clock<Instant = ::chrono04::DateTime<::chrono04::Utc>>,
{
  fn now_chrono(&self) -> ::chrono04::DateTime<::chrono04::Utc> {
    self.now()
  }
}

/// Alias trait for `ChronoClock + Scheduler`
#[cfg(feature = "chrono04")]
pub trait ChronoScheduler: ChronoClock + Scheduler {}

#[cfg(feature = "chrono04")]
impl<T> ChronoScheduler for T where T: ChronoClock + Scheduler {}

/// Alias trait for `ChronoClock + ErasedScheduler`
#[cfg(feature = "chrono04")]
pub trait ErasedChronoScheduler: ChronoClock + ErasedScheduler {}

#[cfg(feature = "chrono04")]
impl<T> ErasedChronoScheduler for T where T: ChronoClock + ErasedScheduler {}
