#![no_std]

//!High performace random number generators for use in `no_std` enviroments.
//!
//!
//!## Seeds
//!Initalizing a generator requires a seed. Two generators initalized with the same seed
//!will produce the same output. 
//!```rust
//!use femtorand::{CoreRNG, WyRand};
//!const SEED: u64 = 0xDEADBEEF;
//!let mut prng_one = WyRand::new(SEED);
//!let mut prng_two = WyRand::new(SEED);
//!// This holds for an arbitrary number of iterations.
//!for _ in 0..10 {
//!    assert_eq!(prng_one.generate_int::<u64>(), prng_two.generate_int::<u64>());
//!}
//!```
//!
//!In many applications havig a generator produce the same
//!output for every program invocation is not desirable, but since this crate is intended 
//!to work in a `no_std` enviroment it cant know what sources of seed randomness are available to the caller.
//!For embedded platforms the included hardware rng can be utilized if available, reading sources of
//!analog noise via some ADC or taking the value of the sytem clock or some counter is also common.
//!In systems utilizing a full desktop operating system,
//!the OS will generally provide some source of entropy that can be utilized as a seed. 
//!See the [getrandom](https://crates.io/crates/getrandom) crate for os randomness sources.
//!
//!## Generators
//!By default this crate uses the `WyRand` PRNG, it is invoked when creating a 
//!generator without specifying a type. `Lehmer64` is also available.
//!```rust
//!use femtorand::{CoreRNG, DefaultRNG, WyRand, Lehmer64};
//!let mut wyrand = WyRand::new(0xDEADBEEF);
//!let mut default_rng = DefaultRNG::new(0xDEADBEEF);
//!// WyRand is currently configured as the default, but this not a guarantee and may change.
//!assert_eq!(wyrand.generate_u128(), default_rng.generate_u128());
//!let mut lehmer = Lehmer64::new(0xDEADBEEF);
//!assert_ne!(lehmer.generate_u128(), default_rng.generate_u128());
//!```
//!
//!## Optional features
//!
//!### Support for floating point
//!Using the `float` crate feature adds support for generation of 
//!floating point values and booleans with adjustabe distribution.
//!```rust
//!use femtorand::{CoreRNG, WyRand, FloatRNG};
//!let mut prng = WyRand::new(0xDEADBEEF);
//!assert_eq!(0.6274890549391671, prng.generate_f64());
//!assert_eq!(true, prng.generate_weighted_bool(0.95));
//!```
#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;


// Compiling docs with optional annotation
//  RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --features float 
mod lehmer;
mod traits;
mod wyrand;

pub use crate::lehmer::*;
pub use crate::traits::*;
pub use crate::wyrand::*;

/// The default RNG is currently [`WyRand`], this may change in the future.
///
/// # Usage
/// ```
/// use femtorand::{WyRand, CoreRNG, DefaultRNG};
/// let mut wyrand_rng = WyRand::new(0xDEADBEEF);
/// let mut default_rng = DefaultRNG::new(0xDEADBEEF);
/// // Identical to WyRand.
/// assert_eq!(wyrand_rng.generate_int::<u64>(), default_rng.generate_int::<u64>());
/// ```
pub type DefaultRNG = WyRand;
