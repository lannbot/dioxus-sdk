//! # Dioxus Time Utilities
//!
//! Cross-platform timing utilities for your Dioxus apps.
//!
//! We currently offer:
//! - [`use_timeout`]
//! - [`use_debounce`]
//! - [`use_interval`]
//! - and [`sleep`]
//!
//! ## Backends
//!
//! Every hook in this crate waits through [`sleep`], which is implemented by one
//! of three backends:
//!
//! | target | backend |
//! | --- | --- |
//! | non-wasm | `tokio::time` |
//! | wasm | `gloo-timers` (browser `setTimeout`) |
//! | `wasm32` with the `wasip3` feature | [`wasi:clocks/monotonic-clock`] |
//!
//! The `wasip3` feature exists for WebAssembly components, which have no
//! JavaScript host to call `setTimeout` on: `gloo-timers` reaches it through
//! `wasm-bindgen`, whose imports are only resolvable on `wasm32-unknown-unknown`
//! with the `wasm-bindgen` CLI in the pipeline. Waiting on the monotonic clock
//! instead keeps the timing path inside the component model.
//!
//! Note that awaiting a WASIp3 import requires a `wit-bindgen` async task to be
//! current, which is the case for code reached from a component's async exports.
//!
//! [`wasi:clocks/monotonic-clock`]: https://github.com/WebAssembly/wasi-clocks
#![warn(missing_docs)]

use std::time::Duration;

mod interval;
pub use interval::{UseInterval, use_interval};

mod debounce;
pub use debounce::{UseDebounce, use_debounce};

mod timeout;
pub use timeout::{TimeoutHandle, UseTimeout, use_timeout};

/// Pause the current task for the specified duration.
///
/// # Examples
/// ```rust
/// use std::time::Duration;
/// use dioxus::prelude::*;
///
/// #[component]
/// pub fn App() -> Element {
///     let mut has_slept = use_signal(|| false);
///
///     use_effect(move || {
///         spawn(async move {
///             dioxus_sdk_time::sleep(Duration::from_secs(3)).await;
///             has_slept.set(true);
///         });
///     });
///
///     rsx! {
///         "I have slept: {has_slept}"
///     }
/// }
/// ```
pub async fn sleep(duration: Duration) {
    #[cfg(not(target_family = "wasm"))]
    tokio::time::sleep(duration).await;

    // `wasi:clocks` durations are nanoseconds in a `u64`, which caps out around
    // 584 years; saturate rather than wrapping a longer request into a short one.
    #[cfg(all(target_arch = "wasm32", feature = "wasip3"))]
    wasip3::clocks::monotonic_clock::wait_for(
        u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX),
    )
    .await;

    #[cfg(all(
        target_family = "wasm",
        not(all(target_arch = "wasm32", feature = "wasip3"))
    ))]
    gloo_timers::future::sleep(duration).await;
}
