//! Private helpers

/// Private trait to implement sealing for [`SyncClock`]
#[cfg(feature = "std")]
pub trait SyncClockSealed {}

/// Private trait to implement sealing for [`ErasedScheduler`]
#[cfg(feature = "std")]
pub trait ErasedSchedulerSealed {}

/// Private trait to implement sealing for [`StdClock`]
#[cfg(feature = "std")]
pub trait StdClockSealed {}

/// Private trait to implement sealing for [`ChronoClock`]
#[cfg(feature = "chrono04")]
pub trait ChronoClockSealed {}

/// Private trait to implement sealing for [`ClockExt`]
#[cfg(feature = "chrono04")]
pub trait SchedulerExtSealed {}

#[cfg(feature = "chrono04")]
pub mod chrono;
