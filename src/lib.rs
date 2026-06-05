//!High performance random number generators for use in `no_std` environments.
//!
//!## Basic Usage
//!```rust
//!use femtorand::{CoreRNG, DefaultRNG};
//!const SEED: u64 = 0xDEADBEEF;
//!let mut prng = DefaultRNG::new(SEED);
//!// Generate integers, where all values are equally likely.
//!// The integer generating functions are generic for:
//!// u8, u16, u32, u64, usize, i8, i16, i32, i64, isize
//!assert_eq!(0x8FC99FBD, prng.generate_int::<u32>());
//!// Generate int between zero (inclusive) and an upper bound (exclusive).
//!// Roll a D6.
//!assert_eq!(6, prng.generate_int_lim::<u8>(6) + 1);
//!// Generate int between a lower bound (inclusive) and an upper bound (exclusive).
//!// Roll another D6.
//!assert_eq!(5, prng.generate_int_range::<u16>(1, 7));
//!// Fill an array.
//!let mut array = [0_u16; 3];
//!prng.fill::<u16>(&mut array);
//!assert_eq!([0xD85D, 0x5DE6, 0x78DE], array);
//!// Choose from a slice.
//!let selection: [&str; 4] = ["a", "b", "c", "d"];
//!assert_eq!("a", *prng.choice(&selection));
//!// Flip a coin.
//!assert_eq!(false, prng.generate_bool());
//!```
//!
//!## Seeds
//!Initializing a generator requires a seed. Two generators initalized with the same seed
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
//!In many applications having a generator produce the same
//!output for every program invocation is not desirable, but since this crate is intended 
//!to work in a `no_std` environment it can't know what sources of seed randomness are available to the caller.  
//!In systems utilizing a full desktop operating system,
//!the OS will generally provide some source of entropy that can be utilized as a seed. 
//!See the [getrandom](https://crates.io/crates/getrandom) crate for OS randomness sources.  
//!The `osseed` crate feature will utilize getrandom to seed a generator with os randomness. 
//!
//!## Generators
//!By default this crate uses the `WyRand` PRNG, it is invoked when creating a 
//!generator without specifying a type. `Lehmer64` is also available.
//!```rust
//!use femtorand::{CoreRNG, DefaultRNG, WyRand, Lehmer64};
//!const SEED: u64 = 0xDEADBEEF;
//!let mut wyrand = WyRand::new(SEED);
//!let mut default_rng = DefaultRNG::new(SEED);
//!// WyRand is currently configured as the default, but this not a guarantee and may change.
//!let default_rng_value = default_rng.generate_u128();
//!assert_eq!(wyrand.generate_u128(), default_rng_value);
//!// A different generator produces different output, even with the same seed.
//!let mut lehmer = Lehmer64::new(SEED);
//!assert_ne!(lehmer.generate_u128(), default_rng_value);
//!```
//!
//!## Optional features
//!
//!### Support for floating point
//!Using the `float` crate feature adds support for generation of 
//!floating point values and booleans with adjustable distribution.  
//!It is enabled by default.
//!```rust
//!#[cfg(feature = "float")]
//!{
//!use femtorand::{CoreRNG, WyRand, FloatRNG};
//!let mut prng = WyRand::new(0xDEADBEEF);
//!assert_eq!(0.6274890549391671, prng.generate_f64());
//!assert_eq!(true, prng.generate_weighted_bool(0.95));
//!}
//!```
//!
//!### Support for seeding from OS random
//!The `osseed` crate feature allows a generator to be automaticaly seeded from 
//!the OS entropy source.
//!```rust
//!#[cfg(feature = "osseed")]
//!{
//!use femtorand::{CoreRNG, WyRand};
//!// The `seeded` function will automatically seed the generator.
//!// This means the output will be different for each program invocation.
//!let mut prng_one = WyRand::seeded().unwrap();
//!let mut prng_two = WyRand::seeded().unwrap();
//!assert_ne!(prng_one.generate_int::<u64>(), prng_two.generate_int::<u64>());
//!}
//!```

#![no_std]

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(auto_cfg))]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;



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
