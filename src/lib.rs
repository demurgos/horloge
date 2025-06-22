//! Library for clock and clock abstractions.
//!
//! This library provides traits representing an abstract clock.
//! - `NowOnce`, `NowMut`, `Now`, these are specialized versions of the
//!   `Fn*` traits. They represent the operation to retrieve the current time.
//! - `SleepOnce`, `SleepMut`, `Sleep`, these are specialized versions of the
//!   `Fn*` traits. They represent the operation to create a timer resolved
//!   at a later time.
//!
//! This crate also provides aliases and utility traits.
//! - `Clock: Now + Sleep`
//! - `ErasedSleep`: `Sleep` variant returning an opaque boxed future.
//! - `StdClock`, `Chrono04Clock`: high-level traits
//! - `NowExt`: helper methods

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

pub trait Duration: PartialOrd {
  fn zero() -> Self;
}

#[cfg(feature = "std")]
impl Duration for ::std::time::Duration {
  fn zero() -> Self {
    Self::ZERO
  }
}

#[cfg(feature = "chrono04")]
impl Duration for ::chrono04::TimeDelta {
  fn zero() -> Self {
    ::chrono04::TimeDelta::zero()
  }
}

#[macro_export]
macro_rules! impl_now {
  {
    impl Now for $clock:ty {
      type Instant = $instant:ty;
      fn now(&$this:ident) -> Self::Instant $body:block
    }
  } => {
    impl $crate::NowOnce for $clock {
      type Instant = $instant;

      fn now_once(mut self) -> Self::Instant {
        $crate::NowMut::now_mut(&mut self)
      }
    }

    impl $crate::NowMut for $clock {
      fn now_mut(&mut self) -> Self::Instant {
        $crate::Now::now(&*self)
      }
    }

    impl $crate::Now for $clock {
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
/// such as [`StdNow`].
///
/// # Primitive trait
///
/// Thia is a primitive trait of this library. It may be implemented on types
/// from other crates. This crate provides implementations integrating with
/// popular ecosystem crates.
pub trait NowOnce {
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
  /// the clock state. For a type `MyNow`. An implementation depending on
  /// exclusive state access would use `impl Now for &'_ mut MyNow {}`. An
  /// implementation without this requirement would use
  /// `impl Now for &'_ MyNow`.
  ///
  /// This design also enables one-shot implementations, akin to
  /// `FnOnce() -> Self::Instant`, but most actual clock implementations should
  /// be implemented on references instead.
  ///
  /// The method is infallible to keep the API simple so it may integrate with
  /// most libraries. If implementations have to deal with failure internally,
  /// they are encouraged to provide a custom separate fallible API. If they
  /// wish to implement this trait, they should return the last known valid
  /// time. If the need for better fallible clock support in this crate arises,
  /// a new trait may be added in a future version.
  fn now_once(self) -> Self::Instant;
}

pub trait NowMut: NowOnce {
  fn now_mut(&mut self) -> Self::Instant;
}

pub trait Now: NowMut {
  fn now(&self) -> Self::Instant;
}

impl<'a, T> NowOnce for &'a T
where
  T: Now,
{
  type Instant = T::Instant;

  fn now_once(self) -> Self::Instant {
    self.now()
  }
}

impl<'a, T> NowOnce for &'a mut T
where
  T: NowMut,
{
  type Instant = T::Instant;

  fn now_once(self) -> Self::Instant {
    self.now_mut()
  }
}

/// Marker trait indicating that a clock is monotonic.
///
/// A monotonic clock is a clock where calls to [`NowOnce::now`] always return
/// a value equal or larger than all previously returned values.
///
/// If this [`NowOnce`] implementation also implements [`Sleep`], then
/// both `now` and `sleep` must be consistent.
///
/// # Safety
///
/// For any `clock` value implementing this trait, the following must hold.
///
/// 1. If `let first = clock.now();` can be ordered with a "happens before"
///    relationship relative to `let second = clock.now();`, then
///    `first <= second`.
///
/// 2. If `clock` also implements [`Sleep<Duration>`], the implementation must
///    ensure that if
///    `let start = <clock as Now>::Instant = ...; let duration: Duration = ...; clock.sleep(duration).await;`
///    can be ordered with a "happens before" relationship relative to
///    `let time = Clock::now();`, then `start + duration <= time`.
///
/// # Primitive trait
///
/// This is a primitive trait of this library. It may be implemented on types
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
/// it must be verified manually. To support a strong correctness guarantee
/// for consumers, which can't be verified automatically by the compiler, this
/// trait is `unsafe` to implement.
pub unsafe trait Monotonic: NowOnce {}

/// Trait for types providing timers, enabling to "sleep".
///
/// The [`sleep`] method returns a timer implementing the `Future` trait. The
/// future resolves when two `Now::now()` calls separated by a sleep have a
/// difference equal or larger than `duration`.
///
/// The timer can be canceled by dropping the value.
///
/// # Primitive trait
///
/// Thia is a primitive trait of this library. It may be implemented on types
/// from other crates. This crate provides implementations integrating with
/// popular ecosystem crates.
pub trait SleepOnce<D> {
  /// Output type for [`Self::sleep_once`].
  ///
  /// This timer is a future that resolves once the current time of this
  /// clock is equal or larger than the deadline provided to
  /// [`Self::sleep_once`].
  ///
  /// Dropping the value cancels the timer.
  ///
  /// # Design
  ///
  /// Using a future to represent timers enables integration with both
  /// non-blocking asynchronous and blocking synchronous code (through in-place
  /// blocking). However, the focus is on asynchronous code as it is more
  /// general. An important use-case of this crate is to help test code
  /// depending on time. In this scenario, there is often a job task and
  /// a control task. A future-based approach allows them to run concurrently
  /// on a single thread which then unlocks easier determinism. Multithreaded
  /// asynchronous or synchronous code is still supported. If the need to better
  /// support a blocking synchronous clock in this crate is raised, then
  /// it may be added as a separate trait without breaking backwards
  /// compatibility.
  type Timer: Future<Output = ()> + Send;

  /// Create a timer future that will resolve once `duration` has elapsed.
  ///
  /// # Design
  ///
  /// The function receives a moved `self` for the same reason as [`NowOnce`].
  /// Most implementations are expected to be implemented on reference types.
  // TODO: update design section, the part below is outdated
  // The signature for this method accepts a point in time in the future, this
  // is the "sleep_until" API style. An alternative approach would have been
  // to accept a duration, this is the "sleep" API style. The "sleep" style
  // has the benefit of supporting the relative timers independent of the
  // current time. As documented at the trait level, the choice was made,
  // however, to couple `Clock` with `Now`. This choice benefits the
  // "sleep_until" approach as it allows specifying a contract between the
  // current time and when timers resolve. Expressing this property with
  // independent `Clock` and `Now` traits would not be possible. The
  // "sleep_until" API style also has the benefit of sharing its argument type
  // with the output type of `Now::now`. A "sleep" style API would require
  // defining a "duration" type and specifying how it relates to
  // `Now::Instant`.
  //
  // This API is infallible. This choice was made for similar reasons as
  // described in [`NowOnce::now`]. Supporting fallibility in a library providing
  // integration across ecosystem crates would make the API more complex and
  // potentially too hard to use, defeating its goal of reducing fragmentation.
  // Fallibility for the `schedule` methods brings a second challenge: there
  // are two ways for this method to fail. Creating the timer can fail, or
  // polling the timer can fail. Most implementation should be fine with an
  // infallible API. If the need for better fallible clock support
  // in this crate arises, a new trait may be added in a future version.
  fn sleep_once(self, duration: D) -> Self::Timer;
}

pub trait SleepMut<D>: SleepOnce<D> {
  fn sleep_mut(&mut self, duration: D) -> Self::Timer;
}

pub trait Sleep<D>: SleepMut<D> {
  fn sleep(&self, duration: D) -> Self::Timer;
}

// /// Trait alias for `Now + Send + Sync`.
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
// pub trait SyncNow: NowOnce + Send + Sync + private::SyncNowSealed {}
//
// impl<T> private::SyncNowSealed for T where T: NowOnce + Send + Sync {}
//
// impl<T> SyncNow for T where T: NowOnce + Send + Sync {}

/// Dyn-compatible version of [`Sleep`].
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "std")]
pub trait ErasedSleepOnce<D>: private::ErasedSleepSealed<D> {
  fn erased_sleep_once(self, duration: D) -> ::std::pin::Pin<Box<dyn Future<Output = ()> + Send>>;
}

#[cfg(feature = "std")]
impl<D, T> private::ErasedSleepSealed<D> for T
where
  T: SleepOnce<D>,
  T::Timer: Send + 'static,
{
}

#[cfg(feature = "std")]
impl<D, T> ErasedSleepOnce<D> for T
where
  T: SleepOnce<D>,
  T::Timer: Send + 'static,
{
  fn erased_sleep_once(self, duration: D) -> ::std::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(self.sleep_once(duration))
  }
}

#[cfg(feature = "std")]
pub trait ErasedSleepMut<D>: ErasedSleepOnce<D> {
  fn erased_sleep_mut(&mut self, duration: D) -> ::std::pin::Pin<Box<dyn Future<Output = ()> + Send>>;
}

#[cfg(feature = "std")]
impl<D, T> ErasedSleepMut<D> for T
where
  T: ErasedSleepOnce<D> + SleepMut<D>,
  T::Timer: Send + 'static,
{
  fn erased_sleep_mut(&mut self, duration: D) -> ::std::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(self.sleep_mut(duration))
  }
}

#[cfg(feature = "std")]
pub trait ErasedSleep<D>: ErasedSleepMut<D> {
  fn erased_sleep(&self, duration: D) -> ::std::pin::Pin<Box<dyn Future<Output = ()> + Send>>;
}

#[cfg(feature = "std")]
impl<D, T> ErasedSleep<D> for T
where
  T: ErasedSleepMut<D> + Sleep<D>,
  T::Timer: Send + 'static,
{
  fn erased_sleep(&self, duration: D) -> ::std::pin::Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(self.sleep(duration))
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
pub trait StdNow: Now<Instant = ::std::time::Instant> + private::StdNowSealed {
  /// Retrieve the current time, as a standard [`Instant`](::std::time::Instant).
  fn now_std(&self) -> ::std::time::Instant;
}

#[cfg(feature = "std")]
impl<T> private::StdNowSealed for T where T: Now<Instant = ::std::time::Instant> {}

#[cfg(feature = "std")]
impl<T> StdNow for T
where
  T: Now<Instant = ::std::time::Instant>,
{
  fn now_std(&self) -> ::std::time::Instant {
    self.now()
  }
}

/// Alias trait for `StdNow + Clock`
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "std")]
pub trait StdClock: StdNow + Sleep<::std::time::Duration> {}

#[cfg(feature = "std")]
impl<T> StdClock for T where T: StdNow + Sleep<::std::time::Duration> {}

/// Alias trait for `StdNow + ErasedClock`
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "std")]
pub trait ErasedStdClock: StdNow + Sleep<::std::time::Duration> {}

#[cfg(feature = "std")]
impl<T> ErasedStdClock for T where T: StdNow + Sleep<::std::time::Duration> {}

/// Trait for a source of the current time, as a [`chrono::DateTime<Utc>`](::chrono04::DateTime).
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "chrono04")]
pub trait Chrono04Now: Now<Instant = ::chrono04::DateTime<::chrono04::Utc>> + private::ChronoNowSealed {
  /// Retrieve the current time, as a [`chrono::DateTime<Utc>`](::chrono04::DateTime).
  fn now_chrono(&self) -> ::chrono04::DateTime<::chrono04::Utc>;
}

#[cfg(feature = "chrono04")]
impl<T> private::ChronoNowSealed for T where T: Now<Instant = ::chrono04::DateTime<::chrono04::Utc>> {}

#[cfg(feature = "chrono04")]
impl<T> Chrono04Now for T
where
  T: Now<Instant = ::chrono04::DateTime<::chrono04::Utc>>,
{
  fn now_chrono(&self) -> ::chrono04::DateTime<::chrono04::Utc> {
    self.now()
  }
}

/// Alias trait for `ChronoNow + Clock`
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "chrono04")]
pub trait Chrono04Clock: Chrono04Now + Sleep<::chrono04::TimeDelta> + Send + Sync {}

#[cfg(feature = "chrono04")]
impl<T> Chrono04Clock for T where T: Chrono04Now + Sleep<::chrono04::TimeDelta> + Send + Sync {}

/// Alias trait for `ChronoNow + ErasedClock`
///
/// # Blanket implementation
///
/// This trait is implemented using a blanket implementation for all types
/// implementing the super traits. If you wish to use types implementing this
/// trait, you can use it as a trait bound. If you wish to implement this trait,
/// you should instead implement the super traits.
#[cfg(feature = "chrono04")]
pub trait ErasedChrono04Clock: Chrono04Now + ErasedSleep<::chrono04::TimeDelta> + Send + Sync {}

#[cfg(feature = "chrono04")]
impl<T> ErasedChrono04Clock for T where T: Chrono04Now + ErasedSleep<::chrono04::TimeDelta> + Send + Sync {}
