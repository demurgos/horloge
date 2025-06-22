//! Private helpers

// /// Private trait to implement sealing for [`SyncNow`]
// #[cfg(feature = "std")]
// pub trait SyncNowSealed {}

/// Private trait to implement sealing for [`ErasedSleep`]
pub trait ErasedSleepSealed<D> {}

/// Private trait to implement sealing for [`StdNow`]
#[cfg(feature = "std")]
pub trait StdNowSealed {}

/// Private trait to implement sealing for [`ChronoNow`]
#[cfg(feature = "chrono04")]
pub trait ChronoNowSealed {}

/// Private trait to implement sealing for [`NowExt`]
pub trait NowExtSealed {}
