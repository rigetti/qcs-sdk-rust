//! This module supports low-level primitives for randomization on Rigetti's QPUs.
use std::{convert::TryFrom, ops::BitXor};

use num::{complex::Complex64, ToPrimitive};
use quil_rs::instruction::{ExternParameter, ExternParameterType, ExternSignature};
use quil_rs::{
    instruction::{Call, CallError, ExternError, UnresolvedCallArgument},
    quil::ToQuilError,
};

/// Hardware values are 48 bits long.
const MAX_SEQUENCER_VALUE: u64 = 0x0000_FFFF_FFFF_FFFF;

/// Hardware multiplication currently uses the lower 16 bits of
/// the PRNG value.
const MAX_UNSIGNED_MULTIPLIER: u64 = 0x0000_0000_0000_FFFF;

/// The taps for the LFSR used on Rigetti control systems. These taps
/// have been shown to produce maximal sequence lengths for 48-bit
/// strings.
const V1_TAPS: [u32; 4] = [47, 46, 20, 19];

/// An error that may occur using the randomization primitives defined
/// in this module.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An invalid seed value was provided.
    #[error(
        "seed values must be in range [1, {MAX_SEQUENCER_VALUE}] and losslessly convertible to f64, found {0}"
    )]
    InvalidSeed(u64),
    /// An error occurred while converting to Quil.
    #[error("error converting to Quil: {0}")]
    ToQuilError(#[from] ToQuilError),
    /// An error occurred while constructing an extern signature.
    #[error("error constructing extern signature: {0}")]
    ExternSignatureError(#[from] ExternError),
    /// The destination must be a `REAL[]`.
    #[error("destination must be a REAL[], found {destination_type:?}")]
    InvalidDestinationType {
        /// The type on the destination declaration.
        destination_type: quil_rs::instruction::ScalarType,
    },
    /// The source must be a `REAL[]`.
    #[error("source must be a REAL[], found {source_type:?}")]
    InvalidSourceType {
        /// The type on the source declaration.
        source_type: quil_rs::instruction::ScalarType,
    },
    /// The destination length must be divisible by the sub-region size.
    #[error(
        "destination length must be in range [0, {}] and divisible by the sub-region size, found {destination_length} % {sub_region_size}", 2u64.pow(f64::MANTISSA_DIGITS) - 1
    )]
    InvalidDestinationLength {
        /// The length of the destination declaration.
        destination_length: u64,
        /// The size of each sub-region in source and destination memory arrays.
        sub_region_size: f64,
    },
    /// The source length must be divisible by the sub-region size.
    #[error(
        "source length must be in range [0, {}] and divisible by the sub-region size, found {source_length} % {sub_region_size}", 2u64.pow(f64::MANTISSA_DIGITS) - 1
    )]
    InvalidSourceLength {
        /// The length of the source declaration.
        source_length: u64,
        /// The size of each sub-region in source and destination memory arrays.
        sub_region_size: f64,
    },
}

/// A specialized `Result` type for hardware extern function calls.
#[allow(clippy::module_name_repetitions)]
pub type RandomResult<T> = Result<T, Error>;

/// An [`ExternedCall`] that may be used to select one or more random
/// sub-regions from a source array of real values to a destination array.
#[derive(Debug, Clone)]
pub struct ChooseRandomRealSubRegions {
    destination_memory_region_name: String,
    source_memory_region_name: String,
    sub_region_size: f64,
    seed_memory_region_name: String,
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
    ///   `len(destination) % sub_region_size` must be zero.
    /// * `seed` - The name of the seed value.
    ///
    /// The values provided for `destination`, `source`, and `seed` must
    /// be declared within the Quil program where the call is made.
    pub fn try_new<T: Into<f64> + Copy>(
        destination: &quil_rs::instruction::Declaration,
        source: &quil_rs::instruction::Declaration,
        sub_region_size: T,
        seed: &quil_rs::instruction::MemoryReference,
    ) -> RandomResult<Self> {
        if !matches!(
            destination.size.data_type,
            quil_rs::instruction::ScalarType::Real
        ) {
            return Err(Error::InvalidDestinationType {
                destination_type: destination.size.data_type,
            });
        }
        if !matches!(
            source.size.data_type,
            quil_rs::instruction::ScalarType::Real
        ) {
            return Err(Error::InvalidSourceType {
                source_type: source.size.data_type,
            });
        }
        if destination
            .size
            .length
            .to_f64()
            .is_none_or(|destination_length| destination_length % sub_region_size.into() != 0f64)
        {
            return Err(Error::InvalidDestinationLength {
                destination_length: destination.size.length,
                sub_region_size: sub_region_size.into(),
            });
        }

        if source
            .size
            .length
            .to_f64()
            .is_none_or(|source_length| source_length % sub_region_size.into() != 0f64)
        {
            return Err(Error::InvalidSourceLength {
                source_length: source.size.length,
                sub_region_size: sub_region_size.into(),
            });
        }

        Ok(Self {
            destination_memory_region_name: destination.name.clone(),
            source_memory_region_name: source.name.clone(),
            sub_region_size: sub_region_size.into(),
            seed_memory_region_name: seed.name.clone(),
        })
    }

    /// The name of the function referenced by the `PRAGMA EXTERN` and `CALL` instructions.
    pub const EXTERN_NAME: &str = "choose_random_real_sub_regions";

    #[allow(clippy::doc_markdown)]
    /// Build the signature for the `PRAGMA EXTERN choose_random_real_sub_regions`
    /// instruction. The signature expressed in Quil is as follows:
    ///
    /// ```text
    /// "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"
    /// ```
    pub fn build_signature() -> RandomResult<ExternSignature> {
        vec![
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
        .collect::<RandomResult<Vec<ExternParameter>>>()
        .map(|parameters| ExternSignature::new(None, parameters))
    }

    /// Build a `PRAGMA EXTERN` instruction for the externed function.
    pub(super) fn pragma_extern() -> Result<quil_rs::instruction::Pragma, Error> {
        use quil_rs::quil::Quil;

        Ok(quil_rs::instruction::Pragma::new(
            quil_rs::instruction::RESERVED_PRAGMA_EXTERN.to_string(),
            vec![quil_rs::instruction::PragmaArgument::Identifier(
                Self::EXTERN_NAME.to_string(),
            )],
            Some(Self::build_signature()?.to_quil().map_err(Error::from)?),
        ))
    }
}

impl TryFrom<ChooseRandomRealSubRegions> for Call {
    type Error = CallError;

    fn try_from(value: ChooseRandomRealSubRegions) -> Result<Self, Self::Error> {
        Self::try_new(
            ChooseRandomRealSubRegions::EXTERN_NAME.to_string(),
            vec![
                UnresolvedCallArgument::Identifier(value.destination_memory_region_name),
                UnresolvedCallArgument::Identifier(value.source_memory_region_name),
                UnresolvedCallArgument::Immediate(Complex64 {
                    re: value.sub_region_size,
                    im: 0.0,
                }),
                UnresolvedCallArgument::Identifier(value.seed_memory_region_name),
            ],
        )
    }
}

/// A valid seed value that may be used to initialize the PRNG. Such
/// values are in the range `[1, MAX_SEQUENCER_VALUE]` and are losslessly
/// convertible to `f64`.
#[derive(Debug, Clone, Copy)]
pub struct PrngSeedValue {
    u64_value: u64,
    f64_value: f64,
}

impl PrngSeedValue {
    /// Attempt to create a new instance of `PrngSeedValue` from a `u64`.
    /// The value must be in the range `[1, MAX_SEQUENCER_VALUE]` and
    /// losslessly convertible to `f64`.
    pub fn try_new(value: u64) -> RandomResult<Self> {
        if !(1..=MAX_SEQUENCER_VALUE).contains(&value) {
            return Err(Error::InvalidSeed(value));
        }
        if let Some(f64_value) = value.to_f64() {
            Ok(Self {
                u64_value: value,
                f64_value,
            })
        } else {
            Err(Error::InvalidSeed(value))
        }
    }

    pub(super) fn as_f64(&self) -> f64 {
        self.f64_value
    }
}

fn lfsr_next(seed: u64, taps: &[u32]) -> u64 {
    let feedback_value = taps.iter().fold(0, |acc, tap| {
        let base = 2u64.pow(*tap);
        let bit = u64::from((seed & base) != 0);
        acc.bitxor(bit)
    });
    ((seed << 1) & MAX_SEQUENCER_VALUE) | feedback_value
}

/// This represents the [linear feedback shift
/// register](https://en.wikipedia.org/wiki/Linear-feedback_shift_register)
/// currently implemented on Rigetti control systems. Specifically,
/// it implements a 48-bit LFSR with taps at 0-based indices 47, 46, 20, and 19.
/// The taps have been shown to produce maximal sequence lengths for 48-bit strings.
#[must_use]
pub fn lfsr_v1_next(seed: PrngSeedValue) -> u64 {
    lfsr_next(seed.u64_value, &V1_TAPS)
}

fn generate_lfsr_v1_sequence(seed: u64, start_index: u32, series_length: u32) -> Vec<u64> {
    let mut lfsr = seed & MAX_SEQUENCER_VALUE;

    let range = start_index..(start_index + series_length);
    let mut collection = vec![];
    for i in 0..(start_index + series_length) {
        lfsr = lfsr_next(lfsr, &V1_TAPS);
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
/// will generate and return the sequence of pseudo-randomly chosen indices on
/// the Rigetti control systems.
///
/// For instance, if the following Quil program is run for 100 shots:
///
/// ```quil
/// # presumed sub-region size is 3.
/// DECLARE destination REAL[6] # prng invocations per shot = (6 / sub_region_size)  = 2
/// DECLARE source REAL[12]     # implicit sub-region count = (12 / sub_region_size) = 4
/// DECLARE seed INTEGER[1]
/// DECLARE ro BIT[1]
///
/// DELAY 0 1e-6
///
/// PRAGMA EXTERN choose_random_real_sub_regions "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"
/// CALL choose_random_real_sub_regions destination source 3 seed
/// ```
///
/// with a seed of 639,523, the following will provide the random sequence of sub-region indices:
///
/// ```rust
/// use qcs::qpu::experimental::random::{choose_random_real_sub_region_indices, PrngSeedValue};
///
/// let seed = PrngSeedValue::try_new(639_523).unwrap();
/// let start_index = 0;
/// let prng_invocations_per_shot = 2;
/// let shot_count = 100;
/// let series_length = prng_invocations_per_shot * shot_count;
/// let sub_region_count = 4;
/// let _random_indices = choose_random_real_sub_region_indices(seed, start_index, series_length, sub_region_count);
/// ```
#[must_use]
pub fn choose_random_real_sub_region_indices(
    seed: PrngSeedValue,
    start_index: u32,
    series_length: u32,
    sub_region_count: u8,
) -> Vec<u8> {
    generate_lfsr_v1_sequence(seed.u64_value, start_index, series_length)
        .iter()
        .map(|&value| prng_value_to_sub_region_index(value, sub_region_count))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs::File};

    /// These are values that have been validated as final memory read off Rigetti QPUs.
    fn prng_sequences() -> HashMap<u32, Vec<(u64, u64)>> {
        serde_json::de::from_reader(
            File::open(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/prng_test_cases.json"
            ))
            .unwrap(),
        )
        .unwrap()
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
