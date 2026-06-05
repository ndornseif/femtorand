//! Implements the Lehmer64 pseudorandom number generator.

/// The unused imports are to guarantee working links in `rustdoc`.
#[allow(clippy::unused_trait_names)]
#[allow(unused_imports)]
use crate::traits::{CoreRNG, FloatRNG, SeekableRNG};
#[allow(unused_imports)]
use crate::wyrand::WyRand;

/// Fast general purpose PRNG based on a linear congruential generator.
///
/// [`Lehmer64`] is a Lehmer RNG, a subtype of linear congruential generators (LCG)
/// with an increment of zero.  
/// The state is 128 bits wide, the modulus is `2**128` and the multiplier `15750249268501108917`.  
/// This generator passes Big Crush [^1].  
/// Whether [`WyRand`] or [`Lehmer64`] are faster is use case and architecture dependent,
/// if in doubt, benchmark.  
/// Unlike [`WyRand`] this generator does not allow freely seeking in the output stream
/// and as a consequence does not implement the [`SeekableRNG`] trait.  
/// The seed `0` is weak and will result in only zeroes being generated,
/// this is a consequence of the increment being zero.  
/// <div class="warning">Lehmer64 is NOT cryptographically secure.</div>
///
/// # Usage
/// ```
/// use femtorand::{CoreRNG, Lehmer64};
/// let mut prng = Lehmer64::new(0xDEADBEEF);
/// assert_eq!(0x487D_D3D8, prng.generate_int::<u32>());
/// prng.reseed(0); // Notice the zero only output with a seed of zero.
/// assert_eq!(0, prng.generate_int::<u32>());
/// ```
///
/// [^1]: [The fastest conventional random number generator that can pass Big Crush?;
/// Lemire D. (2019)](https://lemire.me/blog/2019/03/19/the-fastest-conventional-random-number-generator-that-can-pass-big-crush/)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Lehmer64 {
    /// The generators internal state.
    /// The upper 64 bits of the state are the last output returned by [`Self::next()`].
    state: u128,
}

impl Lehmer64 {
    /// Multiplier used in the LCG.
    const MULTIPLIER: u128 = 0xDA94_2042_E4DD_58B5;
    /// Sets the number of values discarded after instantiating a generator.
    pub const WARMUP_ITERATIONS: usize = 4;
    /// Sets the seed used when instantiating a generator using default.  
    /// This particular value was randomly chosen, it only needs to be nonzero.
    pub const DEFAULT_SEED: u64 = 0x2FB6_A490_3F74_5A36;

    /// Behaves like [`Self::new`] but without discarding the first set of values.
    ///
    /// If the seed is small the first couple values produced by the generator
    /// are of low quality and normally discarded, this function disables this behaviour.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{CoreRNG, Lehmer64};
    /// let mut cold_prng = Lehmer64::new_without_warmup(1);
    /// let mut slice = [0_u16; 4];
    /// cold_prng.fill::<u16>(&mut slice);
    /// // Notice the first output value is zero.
    /// assert_eq!([0x0, 0x65B4, 0x58D0, 0xB969], slice);
    /// // A generator instantiated normally with warmup is four steps ahead.
    /// let mut warmed_prng = Lehmer64::new(1);
    /// assert_eq!(cold_prng.generate_int::<u64>(), warmed_prng.generate_int::<u64>())
    /// ```
    #[inline]
    #[must_use]
    pub fn new_without_warmup(seed: u64) -> Self {
        Self {
            state: u128::from(seed),
        }
    }
}

impl CoreRNG for Lehmer64 {
    /// Initialize new generator with specified seed.
    ///
    /// Zero is a weak seed for [`Lehmer64`] and will only produce zeros
    /// as output.  
    /// If the seed is small, the first couple values are of low quality.  
    /// Because of this the first couple values are discarded, the number is
    /// set by the [`Self::WARMUP_ITERATIONS`] constant.
    #[inline]
    fn new(seed: u64) -> Self {
        let mut generator = Self::new_without_warmup(seed);
        // The first couple values are of low quality if a
        // small seed is used. So we discard the first set of outputs.
        for _ in 0..Self::WARMUP_ITERATIONS {
            let _ = generator.next();
        }
        generator
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(Self::MULTIPLIER);
        (self.state >> 64) as u64
    }
}

#[cfg(feature = "float")]
impl FloatRNG for Lehmer64 {}

impl Default for Lehmer64 {
    /// Initialize with default nonzero seed [`Self::DEFAULT_SEED`].
    ///
    /// This is done to prevent the generator from only generating
    /// zero as output with a zero seed.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{CoreRNG, Lehmer64};
    /// let mut prng = Lehmer64::default();
    /// assert_ne!(0, prng.generate_u128());
    /// ```
    fn default() -> Self {
        Self::new(Self::DEFAULT_SEED)
    }
}
