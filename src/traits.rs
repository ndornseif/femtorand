//! Define traits that a random number generator may implement.

/// One over 2 to the 24th power. Equivalent to 1.0 / (1u32 << 24) as f32.  
/// Exact float representation: 5.9604644775390625E-8
#[cfg(feature = "float")]
const INV_2POW24: f32 = f32::from_bits(0x3380_0000);

/// One over 2 to the 53rd power. Equivalent to 1.0 / (1u64 << 53) as f64.  
/// Exact double representation: 1.1102230246251565404236316680908203125E-16
#[cfg(feature = "float")]
const INV_2POW53: f64 = f64::from_bits(0x3ca0_0000_0000_0000);

/// General trait for pseudorandom number generators.
///
/// This trait defines core generator functionality.
pub trait CoreRNG: Sized {
    /// Initialize new generator with specified seed.
    fn new(seed: u64) -> Self;

    /// Initialize a new generator and seed from OS entropy source.
    ///
    /// This function requires the `osseed` crate feature to be enabled.
    /// The OS entropy source is read using the [`getrandom`] crate.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng_one = WyRand::seeded().unwrap();
    /// let mut prng_two = WyRand::seeded().unwrap();
    /// assert_ne!(prng_one.generate_int::<u64>(), prng_two.generate_int::<u64>());
    /// ```
    #[cfg(feature = "osseed")]
    #[cfg_attr(docsrs, doc(cfg(feature = "osseed")))]
    fn seeded() -> Result<Self, getrandom::Error> {
        let seed = getrandom::u64()?;
        Ok(Self::new(seed))
    }

    /// Generate a pseudorandom u64.
    ///
    /// Advances the generator one step.
    /// Using [`Self::generate_int::<u64>`] is generally preferred.
    /// `next` is the method used to actually define a PRNG.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng_one = WyRand::new(0xDEADBEEF);
    /// let mut prng_two = prng_one;
    /// assert_eq!(prng_one.generate_int::<u64>(), prng_two.next());
    /// ```
    #[doc(hidden)]
    fn next(&mut self) -> u64;

    /// Generate a pseudorandom integer in the interval `[0; max_out)`.
    ///
    /// If `max_out` is zero or one, the output will always be zero.
    /// This function is intended for library internal use.
    /// Prefer [`Self::generate_int_lim`].
    #[allow(clippy::cast_possible_truncation)]
    #[doc(hidden)]
    fn next_lim_u64(&mut self, max_out: u64) -> u64 {
        let cutoff = max_out.wrapping_neg().checked_rem(max_out).unwrap_or(0);
        let mut full;
        let mut low;
        loop {
            full = (u128::from(self.next())).wrapping_mul(u128::from(max_out));
            low = full as u64;
            if low >= cutoff {
                break;
            }
        }
        (full >> 64) as u64
    }

    /// Generate a pseudorandom integer in the interval `[0; max_out)`.
    ///
    /// If `max_out` is zero or one, the output will always be zero.
    /// This function is intended for library internal use.
    /// Prefer [`Self::generate_int_lim`].
    #[allow(clippy::cast_possible_truncation)]
    #[doc(hidden)]
    fn next_lim_u32(&mut self, max_out: u32) -> u32 {
        let cutoff = max_out.wrapping_neg().checked_rem(max_out).unwrap_or(0);
        let mut full;
        let mut low;
        loop {
            full = (self.next()).wrapping_mul(u64::from(max_out));
            low = full as u32;
            if low >= cutoff {
                break;
            }
        }
        (full >> 32) as u32
    }

    /// Reset to initial state with a given seed, equivalent to replacing with `Self::new(seed)`.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng_one = WyRand::new(0xDEADBEEF);
    /// prng_one.reseed(0x42);
    /// let mut prng_two = WyRand::new(0x42);
    /// assert_eq!(prng_one.generate_int::<u64>(), prng_two.generate_int::<u64>());
    /// ```
    #[inline]
    fn reseed(&mut self, seed: u64) {
        *self = Self::new(seed);
    }

    /// Generate a pseudorandom integer.
    ///
    /// Advances the generator one step.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(0xBD, prng.generate_int::<u8>());
    /// assert_eq!(0x8EC4, prng.generate_int::<u16>());
    /// assert_eq!(0xDD1D_B295, prng.generate_int::<u32>());
    /// assert_eq!(0x9370_9C7F_AF7F_D85D, prng.generate_int::<u64>());
    /// assert_eq!(0x759E_48DE_CED2_5DE6, prng.generate_int::<usize>());
    ///
    /// assert_eq!(-0x22, prng.generate_int::<i8>());
    /// assert_eq!(-0x1316, prng.generate_int::<i16>());
    /// assert_eq!(0x4C8B_75FE, prng.generate_int::<i32>());
    /// assert_eq!(-0x1E27_E3EB_99A3_EC8D, prng.generate_int::<i64>());
    /// assert_eq!(0x3AA3_C648_F809_F140, prng.generate_int::<isize>());
    /// ```
    #[inline]
    fn generate_int<T: PrimitiveInteger>(&mut self) -> T {
        T::truncate_from_u64(self.next())
    }

    /// Generate a pseudorandom integer in the interval `[0; max_out)`.
    ///
    /// Advances the generator one step.  
    /// Setting `max_out` to a value less than two will always return zero.  
    /// Based on lemires algorithm[^1].
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// // Roll a D6.
    /// assert_eq!(4, prng.generate_int_lim::<u8>(6) + 1);
    /// assert_eq!(6, prng.generate_int_lim::<u16>(6) + 1);
    /// assert_eq!(5, prng.generate_int_lim::<u32>(6) + 1);
    /// assert_eq!(4, prng.generate_int_lim::<u64>(6) + 1);
    /// assert_eq!(3, prng.generate_int_lim::<usize>(6) + 1);
    ///
    /// // Generating negative integers:
    /// // Wrong
    /// assert_eq!(0, prng.generate_int_lim::<i64>(-10));
    /// // Correct
    /// assert_eq!(-2, -prng.generate_int_lim::<i64>(10))
    /// ```
    ///
    /// [^1]: [Fast Random Integer Generation in an Interval; 
    /// ACM Transactions on Modeling and Computer Simulation 29 (1);
    /// Lemire D., (2018)](https://arxiv.org/abs/1805.10941)
    fn generate_int_lim<T: PrimitiveInteger>(&mut self, max_out: T) -> T {
        if max_out < T::TWO {
            return T::ZERO;
        }
        T::truncate_from_u64(self.next_lim_u64(T::cast_to_u64(max_out)))
    }

    /// Generate a pseudorandom integer in the interval `[min_out; max_out)`.
    ///
    /// Advances the generator one step.
    /// If `max_out - min_out` is less than two, this function will always return `min_out`.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(13, prng.generate_int_range::<u8>(10, 16));
    /// assert_eq!(15, prng.generate_int_range::<u16>(10, 16));
    /// assert_eq!(14, prng.generate_int_range::<u32>(10, 16));
    /// assert_eq!(13, prng.generate_int_range::<u64>(10, 16));
    /// assert_eq!(12, prng.generate_int_range::<usize>(10, 16));
    ///
    /// assert_eq!(-9, prng.generate_int_range::<i8>(-10, -6));
    /// assert_eq!(-8, prng.generate_int_range::<i16>(-10, 1));
    ///
    /// // Incorrect parameters result in `min_out` being returned.
    /// assert_eq!(10, prng.generate_int_range::<i64>(10, 5));
    /// assert_eq!(10, prng.generate_int_range::<i64>(10, 10));
    /// assert_eq!(10, prng.generate_int_range::<i64>(10, 11));  
    /// ```
    fn generate_int_range<T: PrimitiveInteger>(&mut self, min_out: T, max_out: T) -> T {
        let delta = max_out.saturating_sub(&min_out);
        if delta < T::TWO {
            return min_out;
        }
        min_out + self.generate_int_lim::<T>(delta)
    }

    /// Generate a pseudorandom u128.
    ///
    /// This is handled as a special case since the generator
    /// returns 64 bits per iteration, so this function
    /// advances the generator two steps.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, SeekableRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(0xA0A3_1F69_8FC9_9FBD_E083_7439_EEFC_8EC4, prng.generate_u128());
    /// // The generator was advanced two steps.
    /// prng.move_state_backwards(2);
    /// assert_eq!(0xA0A3_1F69_8FC9_9FBD_E083_7439_EEFC_8EC4, prng.generate_u128());
    /// // Moving back only one step causes makes the original lower half the new upper half.
    /// prng.move_state_backwards(1);
    /// assert_eq!(0xE083_7439_EEFC_8EC4_bE5E_8436_DD1D_B295, prng.generate_u128());
    /// ```
    #[inline]
    fn generate_u128(&mut self) -> u128 {
        let upper = u128::from(self.next());
        let lower = u128::from(self.next());
        (upper << 64) | lower
    }

    /// Generate a pseudorandom i128.
    ///
    /// This is handled as a special case since the generator
    /// returns 64 bits per iteration, so this function
    /// advances the generator two steps.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, SeekableRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(-0x5F5C_E096_7036_6042_1F7C_8BC6_1103_713C, prng.generate_i128());
    /// // The generator was advanced two steps.
    /// prng.move_state_backwards(2);
    /// assert_eq!(-0x5F5C_E096_7036_6042_1F7C_8BC6_1103_713C, prng.generate_i128());
    /// // Moving back only one step causes makes the original lower half the new upper half.
    /// prng.move_state_backwards(1);
    /// assert_eq!(-0x1F7C_8BC6_1103_713B_41A1_7BC9_22E2_4D6B, prng.generate_i128());
    /// ```
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    fn generate_i128(&mut self) -> i128 {
        let upper = u128::from(self.next());
        let lower = u128::from(self.next());
        ((upper << 64) | lower) as i128
    }

    /// Generate a pseudorandom boolean with 50% chance of being `true`.
    ///
    /// Advances the generator one step.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(true, prng.generate_bool());
    /// assert_eq!(false, prng.generate_bool());
    /// ```
    #[inline]
    fn generate_bool(&mut self) -> bool {
        (self.next() & 1) != 0
    }

    /// Fill a slice with pseudorandom bytes.
    ///
    /// Advances the generator by `ceil(destination.len() / 8.0)` steps.  
    /// This function is generally faster than [`Self::fill::<u8>`] for slices longer than eight bytes.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// let mut slice = [0_u8; 5];
    /// prng.fill_bytes(&mut slice);
    /// assert_eq!([0xBD, 0x9F, 0xC9, 0x8F, 0x69], slice);
    /// ```
    fn fill_bytes(&mut self, destination: &mut [u8]) {
        for block in destination.chunks_mut(8) {
            block.copy_from_slice(&self.next().to_le_bytes()[0..block.len()]);
        }
    }

    /// Fill a slice with pseudorandom integers.
    ///
    /// Advances the generator one step for each element in `destination`.  
    /// To generate [`u8`] values prefer [`Self::fill_bytes`],
    /// for slices longer than about eight bytes it is faster.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// let mut slice = [0_u16; 3];
    /// prng.fill::<u16>(&mut slice);
    /// assert_eq!([0x9FBD, 0x8EC4, 0xB295], slice);
    /// ```
    fn fill<T: PrimitiveInteger>(&mut self, destination: &mut [T]) {
        for element in destination {
            *element = self.generate_int::<T>();
        }
    }

    /// Randomly select element from slice.
    ///
    /// All elements have equal probability to be chosen.
    /// Advances the generator one step.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// let selection: [&str; 4] = ["a", "b", "c", "d"];
    /// assert_eq!("c", *prng.choice(&selection));
    /// assert_eq!("d", *prng.choice(&selection));
    /// ```
    fn choice<'a, T>(&mut self, selection: &'a [T]) -> &'a T {
        let index = self.generate_int_lim(selection.len());
        &selection[index]
    }
}

/// Trait to allow seeking in the output stream of a random number generator.
///
/// Implemented for RNGs that allow skipping to different positions in the output
/// without needing to generate all the values in between.
pub trait SeekableRNG: CoreRNG {
    /// Advance the generator state by `delta` steps.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, SeekableRNG};
    /// let mut prng_one = WyRand::new(0xDEADBEEF);
    /// let _ = prng_one.generate_int::<u64>();
    /// let mut prng_two = WyRand::new(0xDEADBEEF);
    /// prng_two.move_state_forwards(1);
    /// assert_eq!(prng_one.generate_int::<u64>(), prng_two.generate_int::<u64>());
    /// ```
    fn move_state_forwards(&mut self, delta: u64);

    /// Reverse the generator state by `delta` steps.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, SeekableRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// let result_one = prng.generate_int::<u64>();
    /// prng.move_state_backwards(1);
    /// assert_eq!(result_one, prng.generate_int::<u64>());
    /// ```
    fn move_state_backwards(&mut self, delta: u64);
}

/// Trait to allow a random number generator to produce floating point values.
///
/// Can be implemented by all generators that implement [`CoreRNG`] but is only
/// used when the `float` crate feature is enabled.
#[cfg(feature = "float")]
#[cfg_attr(docsrs, doc(cfg(feature = "float")))]
//#[doc(cfg(feature = "float"))]
pub trait FloatRNG: CoreRNG {
    /// Generate pseudorandom [`f32`] in the interval `[0; 1)`.
    ///
    /// Advances the generator one step.
    /// The resulting float has 24 bits of effective entropy.
    /// The distribution is uniform.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, FloatRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(0.56167024, prng.generate_f32());
    /// ```
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    fn generate_f32(&mut self) -> f32 {
        ((self.next() as u32) >> 8) as f32 * INV_2POW24
    }

    /// Generate pseudorandom [`f64`] in the interval `[0; 1)`.
    ///
    /// Advances the generator one step.
    /// The resulting double has 53 bits of effective entropy.
    /// The distribution is uniform.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, FloatRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(0.6274890549391671, prng.generate_f64());
    /// ```
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn generate_f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 * INV_2POW53
    }

    /// Generate pseudorandom [`f32`] that can take any possible value, including NaN, inf, etc.  
    ///
    /// Advances the generator one step.
    /// All possible [`f32`] values are equally likely, meaning the distribution is not uniform
    /// over the number line.  
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, FloatRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(-1.9881659e-29, prng.generate_any_f32());
    /// ```
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn generate_any_f32(&mut self) -> f32 {
        f32::from_bits(self.next() as u32)
    }

    /// Generate pseudorandom [`f64`] that can take any possible value, including NaN, inf, etc.  
    ///
    /// Advances the generator one step.
    /// All possible [`f64`] values are equally likely, meaning the distribution is not uniform
    /// over the number line.  
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, FloatRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(-1.8255826664036648e-151, prng.generate_any_f64());
    /// ```
    #[inline]
    fn generate_any_f64(&mut self) -> f64 {
        f64::from_bits(self.next())
    }

    /// Generate a pseudorandom boolean with specified chance of being `true`.
    ///
    /// Advances the generator one step.  
    /// `chance` is expressed as a fraction of one. E.g 0.75 is a 75% chance of returning `true`.
    /// Advances the generator one step.
    /// For values of `chance < 0` the output will always be `false`. For `chance > 1` always `true`.
    ///
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, FloatRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(true, prng.generate_weighted_bool(0.75));
    /// ```
    #[inline]
    fn generate_weighted_bool(&mut self, chance: f32) -> bool {
        self.generate_f32() < chance
    }
}

/// Implements casting to a type from [`u128`] and [`u64`].
/// This allows generating a type that implements this trait
/// directly from a RNG, since they output [`u64`].
pub trait PrimitiveInteger:
    Sized + core::cmp::PartialOrd + core::ops::Add<Output = Self> + core::ops::Sub<Output = Self>
{
    /// Number of times this type can fit into a u64.
    const N_U64: usize;
    /// Size of type in bytes.
    const SIZE_BYTES: usize;
    /// The literal `1`.
    const ONE: Self;
    /// The literal `0`.
    const ZERO: Self;
    /// The literal `2`.
    const TWO: Self;
    /// Truncation using `as` cast.
    fn truncate_from_u64(value: u64) -> Self;
    /// Truncation using `as` cast.
    fn truncate_from_u128(value: u128) -> Self;
    /// Cast to a [`u64`] using `as`.
    fn cast_to_u64(value: Self) -> u64;
    /// Subtract `rhs` from `self`. Without going below `Self::MIN`.
    #[must_use]
    fn saturating_sub(self, rhs: &Self) -> Self;
}

/// Implement the truncation functions for a type
/// that allows casting from [`u64`] and [`u128`] via `as`.
macro_rules! impl_primitive_int {
    ($($t:ty),*) => {
        $(
            #[allow(clippy::cast_sign_loss)]
            #[allow(clippy::cast_lossless)]
            #[allow(clippy::cast_possible_wrap)]
            impl PrimitiveInteger for $t {
                const SIZE_BYTES: usize = core::mem::size_of::<$t>();
                const N_U64: usize = (u64::BITS / <$t>::BITS) as usize;
                const ONE: Self = 1;
                const ZERO: Self = 0;
                const TWO: Self = 2;
                #[inline]
                #[allow(clippy::cast_possible_truncation)]
                fn truncate_from_u128(value: u128) -> Self {
                    value as $t
                }
                #[inline]
                #[allow(clippy::cast_possible_truncation)]
                fn truncate_from_u64(value: u64) -> Self {
                    value as $t
                }
                #[inline]
                fn cast_to_u64(value: $t) -> u64 {
                    value as u64
                }
                #[inline]
                fn saturating_sub(self, rhs: &Self) -> Self {
                    <$t>::saturating_sub(self, *rhs)
                }
            }
        )*
    };
}

impl_primitive_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wyrand::WyRand;

    #[test]
    /// We verify that the float generation algorithm
    /// will always generate outputs less than one.
    fn test_float_distribuition() {
        assert!((u64::MAX >> 11) as f64 * INV_2POW53 < 1.0);
        assert!((u32::MAX >> 8) as f32 * INV_2POW24 < 1.0);
    }

    #[test]
    fn test_lim_generation() {
        let mut prng = WyRand::new(0xDEADBEEF);
        assert_eq!(0, prng.next_lim_u64(0));
    }
}
