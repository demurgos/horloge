//! Library for clock and scheduler abstractions.
//!
//! This library provides traits representing an abstract clock

#[cfg(feature = "std")]
mod std_system;
#[cfg(feature = "std")]
mod std_virtual;
#[cfg(all(feature = "std", feature = "chrono04", feature = "tokio1"))]
mod tokio1_chrono04_system;
#[cfg(all(feature = "std", feature = "chrono04", feature = "tokio1"))]
mod tokio1_chrono04_virtual;

mod ext;
mod private;

pub use ext::*;
#[cfg(feature = "std")]
pub use std_system::*;
#[cfg(feature = "std")]
pub use std_virtual::*;
#[cfg(all(feature = "std", feature = "chrono04", feature = "tokio1"))]
pub use tokio1_chrono04_system::*;
#[cfg(all(feature = "std", feature = "chrono04", feature = "tokio1"))]
pub use tokio1_chrono04_virtual::*;

#[macro_export]
macro_rules! impl_clock {
  {
    impl Clock for $clock:ty {
      type Instant = $instant:ty;
      fn now(&$this:ident) -> Self::Instant $body:block
    }
  } => {
    impl $crate::ClockOnce for $clock {
      type Instant = $instant;

      fn now_once(mut self) -> Self::Instant {
        $crate::ClockMut::now_mut(&mut self)
      }
    }

    impl $crate::ClockMut for $clock {
      fn now_mut(&mut self) -> Self::Instant {
        $crate::Clock::now(&*self)
      }
    }

    impl $crate::Clock for $clock {
      #[allow(unused_variables)]
      fn now(&self) -> Self::Instant {
        let $this = self;
        $body
      }
    }
  };
}

#[macro_export]
macro_rules! impl_sleep {
  {
    impl Sleep<$duration:ty> for $sleep:ty {
      type Timer = $timer:ty;
      fn sleep(&$this:ident, $d:ident: $d_ty:ty) -> Self::Timer $body:block
    }
  } => {
    impl $crate::SleepOnce<$duration> for $sleep {
      type Timer = $timer;

      fn sleep_once(mut self, duration: $duration) -> Self::Timer {
        $crate::SleepMut::sleep_mut(&mut self, duration)
      }
    }

    impl $crate::SleepMut<$duration> for $sleep {
      fn sleep_mut(&mut self, duration: $duration) -> Self::Timer {
        $crate::Sleep::sleep(&*self, duration)
      }
    }

    impl $crate::Sleep<$duration> for $sleep {
      #[allow(unused_variables)]
      fn sleep(&self, $d: $d_ty) -> Self::Timer {
        let $this = self;
        $body
      }
    }
  };
}

/// Trait for a source of the current time, as an abstract point in time.
///
/// This trait exposes a single method returning the current point in time.
/// This is a very generic trait. If you can make additional assumptions about
/// the time library you use, it is recommended to use a more specific trait
/// such as `StdClock`.
///
/// # Primitive trait
///
/// Thia is a primitive trait of this library. It may be implemented on types
/// from other crates. This crate provides implementations integrating with
/// popular ecosystem crates.
pub trait ClockOnce {
  /// Type used to represent a point in time.
  ///
  /// No requirements are placed on this type regarding the time resolution,
  /// precision, or other properties / traits implementations. Consumer code
  /// should use more specific sub-traits or bounds to enforce its contract.
  type Instant: PartialOrd;

  /// Returns a value representing the current point in time.
  ///
  /// # Design
  ///
  /// This method receives `self`. This choice was picked as it allows
  /// implementations to control if they need exclusive or shared access to
  /// the clock state. For a type `MyClock`. An implementation depending on
  /// exclusive state would use `impl Clock for &'_ mut MyClock {}`. An
  /// implementation without this requirement would use
  /// `impl Clock for &'_ MyClock`.
  ///
  /// This design also enables one-shot clocks, akin to
  /// `FnOnce() -> Self::Instant`, but most actual clock implementation should
  /// be implemented on references instead.
  ///
  /// The method is infallible to keep the API simple so it may integrate with
  /// most libraries. If implementations have to deal with failure internally,
  /// they should return a dedicated fallible API, and use the latest known
  /// valid time for this method's result. They are also encouraged to provide
  /// a separate fallible API. If the need for better fallible clock support
  /// in this crate arises, a new trait may be added in a future version.
  fn now_once(self) -> Self::Instant;
}

pub trait ClockMut: ClockOnce {
  fn now_mut(&mut self) -> Self::Instant;
}

pub trait Clock: ClockMut {
  fn now(&self) -> Self::Instant;
}

impl<'a, T> ClockOnce for &'a T
where
  T: Clock,
{
  type Instant = T::Instant;

  fn now_once(self) -> Self::Instant {
    self.now()
  }
}

impl<'a, T> ClockOnce for &'a mut T
where
  T: ClockMut,
{
  type Instant = T::Instant;

  fn now_once(self) -> Self::Instant {
    self.now_mut()
  }
}

/// Marker trait indicating that a clock is monotonic.
///
/// A monotonic clock is a clock where calls to [`ClockOnce::now`] always return
/// a value equal or larger than all previously returned values.
///
/// If this [`ClockOnce`] implementation also implements [`Sleep`], then
/// both `now` and `schedule` are consistent.
///
/// # Safety
///
/// For any `clock` value implementing this trait, the following must hold.
///
/// 1. If `let first = clock.now();` can be ordered with a "happens before"
///    relationship relative to `let second = clock.now();`, then
///    `first <= second`.
///
/// 2. If `clock` also implements [`Sleep`], the implementation should
///    ensure that if
///    `let deadline: <clock as Clock>::Instant = ...; clock.schedule(deadline).await;`
///    can be ordered with a "happens before" relationship relative to
///    `let time = Scheduler::now();`, then `deadline <= time`.
///
/// # Primitive trait
///
/// Thia is a primitive trait of this library. It may be implemented on types
/// from other crates. This crate provides implementations integrating with
/// popular ecosystem crates.
///
/// # Design
///
/// Most real clock implementations can't guarantee a monotonic behavior, either
/// due to a lack of guarantees at the API level or because of hardware bugs.
///
/// See for example [the standard library disclaimer about monotonicity](https://doc.rust-lang.org/std/time/struct.Instant.html#monotonicity).
///
/// Since monotonicity is a behavior that depends on specific implementations,
/// it must be verified by manually. To support a strong correctness guarantee
/// for consumers, which can't be verified automatically by the compiler, this
/// trait is `unsafe` to implement.
pub unsafe trait Monotonic: ClockOnce {}

/// Trait for types providing timers, enabling to "sleep".
///
/// The [`schedule`] method returns a timer implementing the `Future` trait. The
/// future resolves when `Scheduler::now()` is equal or larger than `deadline`.
///
/// The timer can be canceled by dropping the value.
///
/// # Primitive trait
///
/// Thia is a primitive trait of this library. It may be implemented on types
/// from other crates. This crate provides implementations integrating with
/// popular ecosystem crates.
///
/// # Design
///
/// This trait extends `Clock`. This coupling is intended. Is is based on
/// two assumptions:
/// 1. Consumer code with the need for timers woken at a later time often also
///    needs to access the current time. The `Clock` super trait enables
///    consumers to use a single bound `Scheduler` instead of `Clock + Scheduler`.
/// 2. `Scheduler` implementations are often able to also provide the current
///    time with much difficulty. In case there happens to be a practical
///    limitation where this assumption is violated, the implementer can still
///    return a sentinel `Instant` value lower than all supported `deadline`
///    values (as if `Clock` was frozen at a point in time).
///
/// The implication of this choice is that implementations can't provide
/// sleeping behavior without a way to retrieve the current time.
pub trait SleepOnce<D> {
  /// Output type for [`Self::sleep_once`].
  ///
  /// This timer is a future that resolves once the current time of this
  /// scheduler is equal or larger than the deadline provided to
  /// [`Self::sleep_once`].
  ///
  /// Dropping the value cancels the timer.
  ///
  /// # Design
  ///
  /// Using a future to represent timers enables integration with both
  /// non-blocking asynchronous and blocking synchronous code (through in-place
  /// blocking). The focus is however on asynchronous code as it is more
  /// general. An important use-case of this crate is to help testing code
  /// depending on the time. In this scenario, there is often a job task and
  /// a control task. A future-based approach allows them to run concurrently
  /// on a single thread which then unlocks easier determinism. Multi-threaded
  /// asynchronous or synchronous code is still supported. If the need to better
  /// support a blocking synchronous scheduler in this crate is raised, then
  /// it may be added as a separate trait without breaking backwards
  /// compatibility.
  type Timer: Future<Output = ()> + Send;

  /// Create a timer future that will resolve once `Self::now() >= deadline`.
  ///
  /// Another name for this operation is `sleep_until`.
  ///
  /// # Design
  ///
  /// The function receives a moved `self` for the same reason as [`ClockOnce`].
  /// Most implementations are expected to be implemented on reference types.
  ///
  /// The signature for this method accepts a point in time in the future, this
  /// is the "sleep_until" API style. An alternative approach would have been
  /// to accept a duration, this is the "sleep" API style. The "sleep" style
  /// has the benefit of supporting the relative timers independent of the
  /// current time. As documented at the trait level, the choice was made,
  /// however, to couple `Scheduler` with `Clock`. This choice benefits the
  /// "sleep_until" approach as it allows specifying a contract between the
  /// current time and when timers resolve. Expressing this property with
  /// independent `Scheduler` and `Clock` traits would not be possible. The
  /// "sleep_until" API style also has the benefit of sharing its argument type
  /// with the output type of `Clock::now`. A "sleep" style API would require
  /// defining a "duration" type and specifying how it relates to
  /// `Clock::Instant`.
  ///
  /// This API is infallible. This choice was made for similar reasons as
  /// described in [`ClockOnce::now`]. Supporting fallibility in a library providing
  /// integration across ecosystem crates would make the API more complex and
  /// potentially too hard to use, defeating its goal of reducing fragmentation.
  /// Fallibility for the `schedule` methods brings a second challenge: there
  /// are two ways for this method to fail. Creating the timer can fail, or
  /// polling the timer can fail. Most implementation should be fine with an
  /// infallible API. If the need for better fallible scheduler support
  /// in this crate arises, a new trait may be added in a future version.
  fn sleep_once(self, duration: D) -> Self::Timer;
}

pub trait SleepMut<D>: SleepOnce<D> {
  fn sleep_mut(&mut self, duration: D) -> Self::Timer;
}

pub trait Sleep<D>: SleepMut<D> {
  fn sleep(&self, duration: D) -> Self::Timer;
}

// /// Trait alias for `Clock + Send + Sync`.
// ///
// /// This trait is implemented automatically if the type implements
// /// the super traits.
// ///
// /// # Blanket implementation
// ///
// /// This trait is implemented using a blanket implementation for all types
// /// implementing the super traits. If you wish to use types implementing this
// /// trait, you can use it as a trait bound. If you wish to implement this trait,
// /// you should instead implement the super traits.
// pub trait SyncClock: ClockOnce + Send + Sync + private::SyncClockSealed {}
//
// impl<T> private::SyncClockSealed for T where T: ClockOnce + Send + Sync {}
//
// impl<T> SyncClock for T where T: ClockOnce + Send + Sync {}

/// Dyn-compatible version of [`Sleep`].
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "std")]
pub trait ErasedSchedulerOnce<D>: private::ErasedSchedulerSealed<D> {
  fn erased_schedule_once(self, duration: D) -> Box<dyn Future<Output = ()>>;
}

#[cfg(feature = "std")]
impl<D, T> private::ErasedSchedulerSealed<D> for T
where
  T: SleepOnce<D>,
  T::Timer: Send + 'static,
{
}

#[cfg(feature = "std")]
impl<D, T> ErasedSchedulerOnce<D> for T
where
  T: SleepOnce<D>,
  T::Timer: Send + 'static,
{
  fn erased_schedule_once(self, duration: D) -> Box<dyn Future<Output = ()>> {
    Box::new(self.sleep_once(duration))
  }
}

#[cfg(feature = "std")]
pub trait ErasedSchedulerMut<D>: ErasedSchedulerOnce<D> {
  fn erased_schedule_mut(&mut self, duration: D) -> Box<dyn Future<Output = ()>>;
}

#[cfg(feature = "std")]
impl<D, T> ErasedSchedulerMut<D> for T
where
  T: ErasedSchedulerOnce<D> + SleepMut<D>,
  T::Timer: Send + 'static,
{
  fn erased_schedule_mut(&mut self, duration: D) -> Box<dyn Future<Output = ()>> {
    Box::new(self.sleep_mut(duration))
  }
}

#[cfg(feature = "std")]
pub trait ErasedScheduler<D>: ErasedSchedulerMut<D> {
  fn erased_schedule(&self, duration: D) -> Box<dyn Future<Output = ()>>;
}

#[cfg(feature = "std")]
impl<D, T> ErasedScheduler<D> for T
where
  T: ErasedSchedulerMut<D> + Sleep<D>,
  T::Timer: Send + 'static,
{
  fn erased_schedule(&self, duration: D) -> Box<dyn Future<Output = ()>> {
    Box::new(self.sleep(duration))
  }
}

/// Trait for a source of the current time, as a standard [`Instant`](::std::time::Instant).
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "std")]
pub trait StdClock: Clock<Instant = ::std::time::Instant> + private::StdClockSealed {
  /// Retrieve the current time, as a standard [`Instant`](::std::time::Instant).
  fn now_std(&self) -> ::std::time::Instant;
}

#[cfg(feature = "std")]
impl<T> private::StdClockSealed for T where T: Clock<Instant = ::std::time::Instant> {}

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
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "std")]
pub trait StdScheduler: StdClock + Sleep<::std::time::Duration> {}

#[cfg(feature = "std")]
impl<T> StdScheduler for T where T: StdClock + Sleep<::std::time::Duration> {}

/// Alias trait for `ChronoClock + ErasedScheduler`
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "std")]
pub trait ErasedStdScheduler: StdClock + Sleep<::std::time::Duration> {}

#[cfg(feature = "std")]
impl<T> ErasedStdScheduler for T where T: StdClock + Sleep<::std::time::Duration> {}

/// Trait for a source of the current time, as a [`chrono::DateTime<Utc>`](::chrono04::DateTime).
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "chrono04")]
pub trait ChronoClock: Clock<Instant = ::chrono04::DateTime<::chrono04::Utc>> + private::ChronoClockSealed {
  /// Retrieve the current time, as a [`chrono::DateTime<Utc>`](::chrono04::DateTime).
  fn now_chrono(&self) -> ::chrono04::DateTime<::chrono04::Utc>;
}

#[cfg(feature = "chrono04")]
impl<T> private::ChronoClockSealed for T where T: Clock<Instant = ::chrono04::DateTime<::chrono04::Utc>> {}

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
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "chrono04")]
pub trait ChronoScheduler: ChronoClock + Sleep<::chrono04::TimeDelta> + Send + Sync {}

#[cfg(feature = "chrono04")]
impl<T> ChronoScheduler for T where T: ChronoClock + Sleep<::chrono04::TimeDelta> + Send + Sync {}

/// Alias trait for `ChronoClock + ErasedScheduler`
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "chrono04")]
pub trait ErasedChronoScheduler: ChronoClock + ErasedScheduler<::chrono04::TimeDelta> + Send + Sync {}

#[cfg(feature = "chrono04")]
impl<T> ErasedChronoScheduler for T where T: ChronoClock + ErasedScheduler<::chrono04::TimeDelta> + Send + Sync {}
