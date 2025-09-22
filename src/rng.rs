//! Define traits that a random number generator many implement.

use crate::traits::PrimitiveInteger;



/// One over 2 to the 24th power. Equivalent to 1.0 / (1u32 << 24) as f32.  
/// Exact float representation: 5.9604644775390625E-8
const INV_2POW24: f32 = f32::from_bits(0x33800000);

/// One over 2 to the 53th power. Equivalent to 1.0 / (1u64 << 53) as f64.  
/// Exact double representation: 1.1102230246251565404236316680908203125E-16
const INV_2POW53: f64 = f64::from_bits(0x3ca0000000000000);

/// General trait for pseudorandom number generators.
///
/// This trait defines core generator functionality.
pub trait CoreRNG: Sized {
    
    /// Initialize new generator with specified seed.
    fn new(seed: u64) -> Self;

    /// Generate a pseudorandom u64.
    /// 
    /// Advances the generator one step.
    /// Using `generate_int::<u64>()` is generally preferred.
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

    /// Reset to inital state with a given seed, equivalent to replacing with ::new(seed).
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
    /// Advances the generator one step for each group of eight bytes.
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// let mut slice = [0u8; 5];
    /// prng.fill_bytes(&mut slice);
    /// assert_eq!([0xBD, 0x9F, 0xC9, 0x8F, 0x69], slice);
    /// ```
    fn fill_bytes(&mut self, destination: &mut [u8]) {
        for block in destination.chunks_mut(8) {
            let required_bytes = block.len();
            block.copy_from_slice(&self.next().to_le_bytes()[0..required_bytes]);
        }
    }
}

/// Trait to allow seeking in the output stream of a random number generator.
///
/// Implemented for RNGs that allow skipping to different positions in the output
/// without needing to generate all the values in between.
pub trait SeekableRNG : CoreRNG {
    /// Advance the generator state by the specified number of steps.
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

    /// Reverse the generator state by the specified number of steps.
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
/// Can be implemented by all generators that implement `CoreRNG` but is only
/// used when the `float` crate feature is enabled.
#[cfg(feature = "float")]
#[cfg_attr(docsrs, doc(cfg(feature = "ﬂoat")))]
//#[doc(cfg(feature = "float"))]
pub trait FloatRNG : CoreRNG {
    /// Generate pseudorandom `f32` in the interval [0; 1).
    ///
    /// Advances the generator one step.
    /// The resulting float has 24 bits of effective entropy. 
    /// The ditribution is uniform.
    /// 
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, FloatRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(0.56167024, prng.generate_f32());
    /// ```
    #[inline]
    fn generate_f32(&mut self) -> f32 {
        ((self.next() as u32) >> 8) as f32 * INV_2POW24
    }

    /// Generate pseudorandom `f64` in the interval [0; 1).
    ///
    /// Advances the generator one step.
    /// The resulting double has 53 bits of effective entropy. 
    /// The ditribution is uniform.
    /// 
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, FloatRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(0.6274890549391671, prng.generate_f64());
    /// ```
    #[inline]
    fn generate_f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 * INV_2POW53
    }

    /// Generate pseudorandom `f32` that can take any possible value, including NaN, inf, ect.  
    ///
    /// Advances the generator one step.
    /// All possible `f32` values are equally likely, meaning the distribution is not uniform
    /// over the number line.  
    /// 
    /// # Usage
    /// ```
    /// use femtorand::{WyRand, CoreRNG, FloatRNG};
    /// let mut prng = WyRand::new(0xDEADBEEF);
    /// assert_eq!(-1.9881659e-29, prng.generate_any_f32());
    /// ```
    #[inline]
    fn generate_any_f32(&mut self) -> f32 {
        f32::from_bits(self.next() as u32)
    }

    /// Generate pseudorandom f64 that can take any possible value, including NaN, inf, ect.  
    ///
    /// Advances the generator one step.
    /// All possible `f64` values are equally likely, meaning the distribution is not uniform
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
    /// `chance` is expressed as a fraction of one. E.g 0.75 is a 75% chance of returning `true`.
    /// Advances the generator one step.
    /// For values of `chance` < 0 the output will always be `false`. For `chance` > 1 always `true`.
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