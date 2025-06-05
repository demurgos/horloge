//! Private helpers for Chrono.

use ::chrono04::{DateTime, Utc};

/// Convert a chrono [`DateTime<Utc>`](DateTime) into a standard [`Instant`](::std::time::Instant)
#[cfg(feature = "tokio1")]
pub(crate) fn chrono_to_std(time: DateTime<Utc>) -> ::std::time::Instant {
  let chrono_now = Utc::now();
  let chrono_duration = time.signed_duration_since(chrono_now);
  let (is_neg, std_duration) = if chrono_duration < ::chrono04::Duration::zero() {
    (true, (-chrono_duration).to_std().unwrap_or(::std::time::Duration::MAX))
  } else {
    (false, chrono_duration.to_std().unwrap_or(::std::time::Duration::MAX))
  };
  let std_now = ::std::time::Instant::now();
  if is_neg {
    std_now - std_duration
  } else {
    std_now + std_duration
  }
}
