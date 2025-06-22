# 0.2.0 (2025-06-22)

- **[Breaking change]** Fully refactor the crate to model the `now` and `sleep`
  operations based on the `Fn*` family of traits. Assume that the version 0.1
  from earlier this month was merely a draft.
- **[Breaking change]** Require the `PartialOrd` bound on `Clock::Instant`.
- Add extension trait for clocks, to compute durations between now and an another
  instant.
- **[Feature]** Add more traits to better handle multi-threading.
- **[Fix]** Fix erased traits missing `Send` bounds or not returning a boxed future.

# 0.1.1 (2025-06-04)

- **[Feature]** Add `ErasedScheduler` trait, a dyn-compatible version of `Scheduler`.
- **[Feature]** Add alias trait for scheduler based on `StdClock` and `ChronoClock`.

# 0.1.0 (2025-06-04)

- **[Feature]** First release.
