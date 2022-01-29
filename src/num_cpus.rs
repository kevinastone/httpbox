//! `num_cpus` wrapper to return NonZeroUsize
//!
//! It was deemed a breaking change for the `num_cpus` package and not worth the
//! trouble, so this shim addresses that:
//! https://github.com/seanmonstar/num_cpus/issues/105
use std::num::NonZeroUsize;

pub fn num_cpus() -> NonZeroUsize {
    unsafe {
        // SAFETY: [`num_cpus::get`][num_cpus_get] always returns at least 1
        // https://github.com/seanmonstar/num_cpus/pull/106
        //
        // [num_cpus_get]: https://docs.rs/num_cpus/latest/num_cpus/fn.get.html
        NonZeroUsize::new_unchecked(num_cpus::get())
    }
}
