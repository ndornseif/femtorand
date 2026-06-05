//! Implement the `WyRand` pseudorandom number generator.

/// The unused imports are to guarantee working links in `rustdoc`.
#[allow(unused_imports)]
use crate::lehmer::Lehmer64;
use crate::traits::{CoreRNG, FloatRNG, SeekableRNG};

/// Fast general purpose PRNG based on the `WyHash` multiply and mix function.
///
/// [`WyRand`] is a fast pseudorandom number generator
/// derived from the `WyHash` hash function [^1].  
/// Whether [`WyRand`] or [`Lehmer64`] are faster is use case and architecture dependent,
/// if in doubt, benchmark.  
/// This generator allows freely moving to any position in the output stream
/// via the [`SeekableRNG`] trait.
/// <div class="warning">WyRand is NOT cryptographically secure.</div>
///
/// # Usage
/// ```
/// use femtorand::{CoreRNG, WyRand, SeekableRNG};
/// let mut prng = WyRand::new(0xDEADBEEF);
/// assert_eq!(0x8FC9_9FBD, prng.generate_int::<u32>());
/// prng.reseed(0); // Unlike Lehmer64 zero is not a weak seed.
/// assert_eq!(0x8D59_F0D6, prng.generate_int::<u32>());
/// prng.move_state_backwards(1); // Seeking in the output stream is supported.
/// assert_eq!(0x8D59_F0D6, prng.generate_int::<u32>());
/// ```
///
/// [^1]: [Modern Non-Cryptographic Hash Function and Pseudorandom Number Generator;
/// Wang Y., Romero D. B.,Lemire D., Jin L. (2020)](https://github.com/wangyi-fudan/wyhash/blob/master/Modern%20Non-Cryptographic%20Hash%20Function%20and%20Pseudorandom%20Number%20Generator.pdf)
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct WyRand {
    /// The generators internal state.
    /// For each step constant [`Self::WY0`] is added to the state.
    /// The value returned by [`Self::next()`] is the current state fed
    /// into the `WyHash` multiply and mix function.
    state: u64,
}

impl WyRand {
    /// Generator constant taken from the `WyHash` paper.
    const WY0: u64 = 0x2D35_8DCC_AA6C_78A5;
    /// Generator constant taken from the `WyHash` paper.
    const WY1: u64 = 0x8BB8_4B93_962E_ACC9;
}

impl CoreRNG for WyRand {
    #[inline]
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(Self::WY0);
        let c = u128::from(self.state).wrapping_mul(u128::from(self.state ^ Self::WY1));
        ((c >> 64) as u64) ^ (c as u64)
    }
}

impl SeekableRNG for WyRand {
    #[inline]
    fn move_state_forwards(&mut self, delta: u64) {
        self.state = self.state.wrapping_add(Self::WY0.wrapping_mul(delta));
    }

    #[inline]
    fn move_state_backwards(&mut self, delta: u64) {
        self.state = self.state.wrapping_sub(Self::WY0.wrapping_mul(delta));
    }
}

#[cfg(feature = "float")]
impl FloatRNG for WyRand {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Check that seeking in output stream works with large values of `delta`.
    fn test_wide_seek() {
        let mut prng = WyRand::new(0xDEADBEEF);
        let n_one = prng.next();
        prng.move_state_forwards(u64::MAX - 2);
        prng.move_state_backwards(u64::MAX - 1);
        let n_two = prng.next();
        assert_eq!(n_one, n_two);

        // Verify output has a period after 2**64 values.
        let mut prng = WyRand::new(0xDEADBEEF);
        let n_one = prng.next();
        prng.move_state_forwards(u64::MAX);
        let n_two = prng.next();
        assert_eq!(n_one, n_two);
    }
}
