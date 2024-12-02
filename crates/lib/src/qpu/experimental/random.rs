//! This module supports low-level primitives for randomization on Rigetti's QPUs.
use std::{convert::TryFrom, ops::BitXor};

use num::{complex::Complex64, ToPrimitive};
use quil_rs::instruction::{Call, CallError, ExternError, UnresolvedCallArgument};

use crate::qpu::externed_call::ExternedCall;

/// An error that may occur when simulating control system extern
/// function calls.
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    /// An invalid seed value was provided.
    #[error(
        "seed values must be in range [1, {MAX_SEQUENCER_VALUE}) and losslessly convertible to f64, found {0}"
    )]
    InvalidSeed(u64),
    /// An error occurred while converting to Quil.
    #[error("error converting to Quil: {0}")]
    ToQuilError(String),
    /// An error occurred while constructing an extern signature.
    #[error("error constructing extern signature: {0}")]
    ExternSignatureError(#[from] ExternError),
}

impl From<quil_rs::quil::ToQuilError> for Error {
    fn from(e: quil_rs::quil::ToQuilError) -> Self {
        Self::ToQuilError(e.to_string())
    }
}

/// A specialized `Result` type for hardware extern function calls.
pub type Result<T> = std::result::Result<T, Error>;

/// An [`ExternedCall`] that may be used to select one or more random
/// sub-regions from a source array of real values to a destination array.
#[derive(Debug, Clone)]
pub struct ChooseRandomRealSubRegions {
    destination: String,
    source: String,
    sub_region_size: Complex64,
    seed: String,
}

impl ChooseRandomRealSubRegions {
    /// Create a new instance of [`ChooseRandomRealSubRegions`].
    ///
    /// # Parameters
    ///
    /// * `destination` - The name of the destination array.
    /// * `source` - The identifier of the source array.
    /// * `sub_region_size` - The size of the sub-regions to select from
    ///   the source array. Note, `len(source) % sub_region_size` and
    ///   `len(destination)` must be zero.
    /// * `seed` - The name of the seed value.
    ///
    /// The values provided for `destination`, `source`, and `seed` must
    /// be declared within the Quil program where the call is made.
    pub fn new<T: Into<f64>>(
        destination: String,
        source: String,
        sub_region_size: T,
        seed: String,
    ) -> Self {
        Self {
            destination,
            source,
            sub_region_size: Complex64 {
                re: sub_region_size.into(),
                im: 0.0,
            },
            seed,
        }
    }
}

impl TryFrom<ChooseRandomRealSubRegions> for Call {
    type Error = CallError;

    fn try_from(value: ChooseRandomRealSubRegions) -> std::result::Result<Self, Self::Error> {
        Self::try_new(
            ChooseRandomRealSubRegions::NAME.to_string(),
            vec![
                UnresolvedCallArgument::Identifier(value.destination.clone()),
                UnresolvedCallArgument::Identifier(value.source.to_string()),
                UnresolvedCallArgument::Immediate(value.sub_region_size),
                UnresolvedCallArgument::Identifier(value.seed.clone()),
            ],
        )
    }
}

impl ExternedCall for ChooseRandomRealSubRegions {
    type Error = Error;

    const NAME: &str = "choose_random_real_sub_regions";

    #[allow(clippy::doc_markdown)]
    /// Build the signature for the `PRAGMA EXTERN choose_random_real_sub_regions`
    /// instruction. The signature is:
    ///
    /// rust ignore
    ///     "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"
    fn build_signature() -> Result<quil_rs::instruction::ExternSignature> {
        use quil_rs::instruction::{ExternParameter, ExternParameterType};

        let parameters = vec![
            ExternParameter::try_new(
                "destination".to_string(),
                true,
                ExternParameterType::VariableLengthVector(quil_rs::instruction::ScalarType::Real),
            ),
            ExternParameter::try_new(
                "source".to_string(),
                false,
                ExternParameterType::VariableLengthVector(quil_rs::instruction::ScalarType::Real),
            ),
            ExternParameter::try_new(
                "sub_region_size".to_string(),
                false,
                ExternParameterType::Scalar(quil_rs::instruction::ScalarType::Integer),
            ),
            ExternParameter::try_new(
                "seed".to_string(),
                true,
                ExternParameterType::Scalar(quil_rs::instruction::ScalarType::Integer),
            ),
        ]
        .into_iter()
        .map(|r| r.map_err(Error::from))
        .collect::<Result<Vec<ExternParameter>>>()?;
        Ok(quil_rs::instruction::ExternSignature::new(None, parameters))
    }
}

/// Hardware values are 48 bits long.
const MAX_SEQUENCER_VALUE: u64 = 0xFFFF_FFFF_FFFF;

/// Hardware multiplication currently uses the lower 16 bits of
/// the PRNG value.
const MAX_UNSIGNED_MULTIPLIER: u64 = 0x0000_0000_FFFF;

/// A valid seed value that may be used to initialize the PRNG. Such
/// values are in the range `[1, MAX_SEQUENCER_VALUE]` and are losslessly
/// convertible to `f64`.
#[derive(Debug, Clone, Copy)]
pub struct PrngSeedValue {
    as_u64: u64,
    pub(super) as_f64: f64,
}

impl PrngSeedValue {
    /// Attempt to create a new instance of `PrngSeedValue` from a `u64`.
    /// The value must be in the range `[1, MAX_SEQUENCER_VALUE]` and
    /// losslessly convertible to `f64`.
    pub fn try_new(value: u64) -> Result<Self> {
        if !(1..=MAX_SEQUENCER_VALUE).contains(&value) {
            return Err(Error::InvalidSeed(value));
        }
        if let Some(f64_value) = value.to_f64() {
            Ok(Self {
                as_u64: value,
                as_f64: f64_value,
            })
        } else {
            Err(Error::InvalidSeed(value))
        }
    }
}

fn lfsr_next(seed: u64, taps: &[u32]) -> u64 {
    let feedback_value = taps.iter().fold(0, |acc, tap| {
        let base = 2u64.pow(*tap - 1);
        let bit = u64::from((seed & base) != 0);
        acc.bitxor(bit)
    });
    (seed << 1) & MAX_SEQUENCER_VALUE | feedback_value
}

/// This represents the LFSR currently implemented on Rigetti control systems. Specifically,
/// it implements a 48-bit LFSR with taps at indices 48, 47, 21, and 20.
#[must_use]
pub fn lfsr_v1_next(seed: PrngSeedValue) -> u64 {
    lfsr_next(seed.as_u64, &[48, 47, 21, 20])
}

fn generate_lfsr_v1_sequence(seed: u64, start_index: u32, series_length: u32) -> Vec<u64> {
    let mut lfsr = seed & MAX_SEQUENCER_VALUE;

    let range = start_index..(start_index + series_length);
    let mut collection = vec![];
    for i in 0..(start_index + series_length) {
        lfsr = lfsr_next(lfsr, &[48, 47, 21, 20]);
        if range.contains(&i) {
            collection.push(lfsr);
        }
    }
    collection
}

fn prng_value_to_sub_region_index(value: u64, sub_region_count: u8) -> u8 {
    ((value & MAX_UNSIGNED_MULTIPLIER) % u64::from(sub_region_count))
        .to_u8()
        .expect("modulo u8 should always produce a valid value")
}

/// Given a seed, start index, series length, and sub-region count, this function
/// will generate the sequence of pseudo-randomly chosen indices Rigetti control
/// systems.
///
/// For instance, if the following Quil program is run for 100 shots:
///
/// ```quil
/// # presumed sub-region size is 3.
/// DECLARE destination REAL[6] # prng invocations per shot = (6 / sub_region_size)  = 2
/// DELCARE source REAL[12]     # implicit sub-region count = (12 / sub_region_size) = 4
/// DECLARE seed INTEGER[1]
/// DECLARE ro BIT[1]
///
/// DELAY 0 1e-6
///
// PRAGMA EXTERN choose_random_real_sub_regions "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"
// CALL choose_random_real_sub_regions destination source 3 seed
/// ```
///
/// with a seed of 639523, you could backout the randomly chosen sub-regions with the following:
///
/// ```rust
/// let seed = 639523;
/// let start_index = 0;
/// let prng_invocations_per_shot = 2;
/// let shot_count = 100;
/// let series_length = prng_invocations_per_shot * shot_count;
/// let sub_region_count = 4;
/// let _random_indices = choose_random_real_sub_region_indices(seed, start_index, series_length, sub_region_count).unwrap();
/// ```
#[must_use]
pub fn choose_random_real_sub_region_indices(
    seed: PrngSeedValue,
    start_index: u32,
    series_length: u32,
    sub_region_count: u8,
) -> Vec<u8> {
    generate_lfsr_v1_sequence(seed.as_u64, start_index, series_length)
        .iter()
        .map(|&value| prng_value_to_sub_region_index(value, sub_region_count))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs::File};

    fn prng_sequences() -> HashMap<u32, Vec<(u64, u64)>> {
        serde_json::de::from_reader(File::open("tests/prng_test_cases.json").unwrap()).unwrap()
    }

    #[test]
    fn test_lfsr_v1_next() {
        for (num_shots, sequences) in prng_sequences() {
            for (seed, expected) in sequences {
                let sequence = super::generate_lfsr_v1_sequence(seed, num_shots - 1, 1);
                assert_eq!(sequence.len(), 1);
                let end_of_sequence = sequence[0];
                assert_eq!(
                    end_of_sequence, expected,
                    "seed={seed}, num_shots={num_shots}",
                );
            }
        }
    }
}
