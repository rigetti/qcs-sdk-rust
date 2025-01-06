//! This module contains generalized support for adding randomized measurements
//! to a Quil program. It builds off of the primitives defined in [`super::random`].
//!
//! Measurement randomization is a technique used in both quantum tomography and
//! quantum error mitigation. Essentially, it involves randomly rotating each
//! qubit prior to measurement. This module enables per-shot randomization, where
//! random rotations are applied to each qubit independently for each shot. For
//! some background on the technique, see
//! [Predicting Many Properties of a Quantum System from Very Few Measurements
//! (arxiv:2002.08953)](https://arxiv.org/abs/2002.08953).
//!
//! The [`RandomizedMeasurements`] struct handles three critical components to
//! correctly add randomized measurements to a Rigetti QCS Quil program:
//!
//! 1. Program construction - adding the classical randomization calls to the
//!     prologue of the program (i.e. before the pulse program begins) and referencing
//!     those randomized values within a unitary decomposition prior to measurement.
//! 2. Parameter construction - building a map of [`Parameters`] with seeds for
//!     each qubit.
//! 3. PRNG reconstruction - backing out the random indices that played on each
//!     qubit during program execution.
//!
//! This is not a QIS (quantum information science) library, but rather an
//! SDK for collecting data from Rigetti QPUs. As such, defining a proper
//! unitary set and using randomized measurement data is beyond the scope of this
//! library.

use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use itertools::Itertools;
use num::ToPrimitive;
use quil_rs::{
    expression::Expression,
    instruction::{
        Call, CallError, Declaration, Delay, Fence, Instruction, Measurement, MemoryReference,
        Qubit, ScalarType, Vector,
    },
    quil::Quil,
    Program,
};

use super::random::{ChooseRandomRealSubRegions, PrngSeedValue};
use crate::executable::Parameters;

const RANDOMIZED_MEASUREMENT_SOURCE: &str = "randomized_measurement_source";
const RANDOMIZED_MEASUREMENT_DESTINATION: &str = "randomized_measurement_destination";
const RANDOMIZED_MEASUREMENT_SEED: &str = "randomized_measurement_seed";

/// An error that may occur when constructing randomized measurements.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Received measurement on non-fixed qubits.
    #[error("only measurements on fixed qubits are supported, found {0:?}")]
    UnsupportedMeasurementQubit(Qubit),
    /// The program already contains measurements.
    #[error("program contains preexisting measurements on qubits: {0:?}")]
    ProgramContainsPreexistingMeasurements(HashSet<Qubit>),
    /// The implicit unitary count could not be expressed as a u8.
    #[error("the unitary count must be within range [0, {}], found {0}", u8::MAX)]
    UnitaryCountU8Conversion(usize),
}

/// An error that may occur when constructing randomized measurements.
#[derive(Debug, thiserror::Error)]
pub enum AppendToProgramError<UnitarySetError> {
    /// An error occurred while constructing `PRAGMA EXTERN` instruction.
    #[error("error declaring extern function: {0}")]
    BuildPragma(#[from] super::random::Error),
    /// An error occurred while constructing a call instruction.
    #[error("error initializing call instruction: {0}")]
    BuildCall(#[from] CallError),
    /// The number of parameters per unitary could not be expressed losslessly as
    /// an f64.
    #[error(
        "the number of parameters per unitary must be within range [0, {}], found {0}", 2_u64.pow(f64::MANTISSA_DIGITS) - 1
    )]
    ParametersPerUnitaryF64Conversion(usize),
    /// The program already contains measurements.
    #[error("program contains preexisting measurements on qubits: {0:?}")]
    ProgramContainsPreexistingMeasurements(HashSet<Qubit>),
    /// An error occurred while building unitary set instructions.
    #[error("an error occurred while building unitary set instructions: {0}")]
    UnitarySet(UnitarySetError),
    /// An error occurred while validating the `choose_random_real_sub_regions` function.
    #[error(
        "an error occurred while validating the `choose_random_real_sub_regions` function: {0}"
    )]
    ChooseRandomRealSubRegions(super::random::Error),
}

/// An error that may occur when constructing randomized measurements.
#[derive(Debug, thiserror::Error)]
pub enum ToParametersError<UnitarySetError> {
    /// A seed value was not provided for a qubit.
    #[error("seed not provided for qubit {}", .0.to_quil_or_debug())]
    MissingSeed(Qubit),
    /// An error occurred while building unitary set parameters.
    #[error("an error occurred while building unitary set parameters: {0}")]
    UnitarySet(UnitarySetError),
}

/// The declarations and measurements required to randomize a single qubit.
#[derive(Debug, Clone)]
pub struct QubitRandomization {
    seed_declaration: Declaration,
    destination_declaration: Declaration,
    measurement: Measurement,
    qubit_index: u64,
}

impl QubitRandomization {
    /// Return a reference to the declaration that will be used as the seed to the PRNG
    /// sequence for this qubit.
    #[must_use]
    pub fn get_seed_declaration(&self) -> &Declaration {
        &self.seed_declaration
    }

    /// Return a reference to the declaration that will hold the parameters that characterize
    /// the random rotation to be applied to this qubit on a per-shot basis.
    #[must_use]
    pub fn get_destination_declaration(&self) -> &Declaration {
        &self.destination_declaration
    }

    /// Get the measurement instruction that will be used to measure this qubit.
    #[must_use]
    pub fn get_measurement(&self) -> &Measurement {
        &self.measurement
    }

    /// Get the index of the qubit that will be randomized.
    #[must_use]
    pub fn get_qubit_index(&self) -> u64 {
        self.qubit_index
    }

    fn into_instruction<E>(
        self,
        parameters_per_unitary: f64,
        source_declaration: &Declaration,
    ) -> Result<Instruction, AppendToProgramError<E>> {
        let externed_call = ChooseRandomRealSubRegions::try_new(
            &self.destination_declaration,
            source_declaration,
            parameters_per_unitary,
            &MemoryReference {
                name: self.seed_declaration.name.clone(),
                index: 0,
            },
        )
        .map_err(AppendToProgramError::ChooseRandomRealSubRegions)?;
        Ok(Instruction::Call(Call::try_from(externed_call)?))
    }
}

/// Configuration for adding randomized measurements to a Quil program.
#[derive(Debug, Clone)]
pub struct RandomizedMeasurements<U> {
    leading_delay: Expression,
    unitary_set: U,
    unitary_count_as_u8: u8,
    qubit_randomizations: Vec<QubitRandomization>,
    source_declaration: Declaration,
}

impl<U> RandomizedMeasurements<U>
where
    U: UnitarySet,
{
    /// Initialize a new instance of [`RandomizedMeasurements`].
    ///
    /// # Parameters
    ///
    /// * `measurements` - A vector of measurements to randomize. Note, these
    ///     measurements should not be added to a program a priori.
    /// * `unitary_set` - The [`UnitarySet`] from which to draw and represent
    ///     randomly selected unitaries.
    /// * `leading_delay` - The delay to prepend to the program before the
    ///     pulse program begins.
    pub fn try_new(
        measurements: Vec<Measurement>,
        unitary_set: U,
        leading_delay: Expression,
    ) -> Result<RandomizedMeasurements<U>, Error> {
        let source_declaration = Declaration {
            name: RANDOMIZED_MEASUREMENT_SOURCE.to_string(),
            size: Vector::new(
                ScalarType::Real,
                (unitary_set.parameters_per_unitary() * unitary_set.unitary_count()) as u64,
            ),
            sharing: None,
        };
        let qubit_randomizations: Vec<QubitRandomization> = measurements
            .into_iter()
            .map(|measurement| {
                if let Qubit::Fixed(qubit_index) = measurement.qubit {
                    Ok((measurement, qubit_index))
                } else {
                    Err(Error::UnsupportedMeasurementQubit(measurement.qubit))
                }
            })
            .map_ok(|(measurement, qubit_index)| QubitRandomization {
                seed_declaration: Declaration {
                    name: format!("{RANDOMIZED_MEASUREMENT_SEED}_q{qubit_index}"),
                    size: Vector::new(ScalarType::Integer, 1),
                    sharing: None,
                },
                destination_declaration: Declaration {
                    name: format!("{RANDOMIZED_MEASUREMENT_DESTINATION}_q{qubit_index}"),
                    size: Vector::new(
                        ScalarType::Real,
                        unitary_set.parameters_per_unitary() as u64,
                    ),
                    sharing: None,
                },
                qubit_index,
                measurement,
            })
            .collect::<Result<_, Error>>()?;
        let unitary_count_as_u8 = unitary_set
            .unitary_count()
            .to_u8()
            .ok_or_else(|| Error::UnitaryCountU8Conversion(unitary_set.unitary_count()))?;
        Ok(Self {
            leading_delay,
            unitary_set,
            unitary_count_as_u8,
            qubit_randomizations,
            source_declaration,
        })
    }
}

impl<U> RandomizedMeasurements<U>
where
    U: UnitarySet,
{
    /// Append the randomized measurements to a Quil program. The provided program
    /// must not contain any preexisting measurements.
    pub fn append_to_program(
        &self,
        target_program: Program,
    ) -> Result<Program, AppendToProgramError<<U as UnitarySet>::Error>> {
        let measured_qubits = target_program
            .to_instructions()
            .into_iter()
            .filter_map(|instruction| {
                if let Instruction::Measurement(measurement) = instruction {
                    Some(measurement.qubit)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        let qubits_with_redundant_measurements = self
            .qubit_randomizations
            .iter()
            .filter(|randomization| measured_qubits.contains(&randomization.measurement.qubit))
            .map(|randomization| randomization.measurement.qubit.clone())
            .collect::<HashSet<_>>();
        if !qubits_with_redundant_measurements.is_empty() {
            return Err(
                AppendToProgramError::ProgramContainsPreexistingMeasurements(
                    qubits_with_redundant_measurements,
                ),
            );
        }
        let mut program = target_program.clone_without_body_instructions();

        program.add_instruction(Instruction::Declaration(self.source_declaration.clone()));

        for qubit_randomization in &self.qubit_randomizations {
            program.add_instruction(Instruction::Declaration(
                qubit_randomization.destination_declaration.clone(),
            ));
            program.add_instruction(Instruction::Declaration(
                qubit_randomization.seed_declaration.clone(),
            ));
        }

        // prepend delay
        program.add_instruction(Instruction::Delay(Delay::new(
            self.leading_delay.clone(),
            Vec::new(),
            self.qubit_randomizations
                .iter()
                .map(|randomization| randomization.measurement.qubit.clone())
                .collect(),
        )));

        // declare "choose_random_real_sub_regions" as an external function
        program.add_instruction(Instruction::Pragma(
            ChooseRandomRealSubRegions::pragma_extern()?,
        ));

        let parameters_per_unitary = self
            .unitary_set
            .parameters_per_unitary()
            .to_f64()
            .ok_or_else(|| {
                AppendToProgramError::ParametersPerUnitaryF64Conversion(
                    self.unitary_set.parameters_per_unitary(),
                )
            })?;
        // Before the pulse program begins, set the randomized unitary for each qubit.
        let calls: Vec<_> = self
            .qubit_randomizations
            .iter()
            .cloned()
            .map(|qubit_randomization| {
                qubit_randomization
                    .into_instruction(parameters_per_unitary, &self.source_declaration)
            })
            .collect::<Result<Vec<Instruction>, AppendToProgramError<<U as UnitarySet>::Error>>>(
            )?;
        program.add_instructions(calls);

        // Include the program body that was passed in.
        program.add_instructions(target_program.into_body_instructions());

        // Play the random unitaries on each qubit.
        program.add_instructions(
            self.unitary_set
                .to_instructions(self.qubit_randomizations.as_slice())
                .map_err(AppendToProgramError::UnitarySet)?,
        );

        program.add_instruction(Instruction::Fence(Fence { qubits: Vec::new() }));
        // Measure each qubit.
        for qubit_randomization in &self.qubit_randomizations {
            program.add_instruction(Instruction::Measurement(
                qubit_randomization.measurement.clone(),
            ));
        }
        Ok(program)
    }

    /// Given a map of qubits to seed values, construct the parameters required
    /// to randomize measurements accordingly.
    pub fn to_parameters(
        &self,
        seed_values: &HashMap<u64, PrngSeedValue>,
    ) -> Result<Parameters, ToParametersError<<U as UnitarySet>::Error>> {
        let mut parameters = HashMap::new();
        parameters.insert(
            RANDOMIZED_MEASUREMENT_SOURCE.to_string().into_boxed_str(),
            self.unitary_set
                .to_parameters()
                .map_err(ToParametersError::UnitarySet)?,
        );

        for qubit_randomization in &self.qubit_randomizations {
            let seed_value = seed_values
                .get(&qubit_randomization.qubit_index)
                .ok_or_else(|| {
                    ToParametersError::MissingSeed(qubit_randomization.measurement.qubit.clone())
                })?;
            parameters.insert(
                qubit_randomization
                    .seed_declaration
                    .name
                    .clone()
                    .into_boxed_str(),
                vec![seed_value.as_f64()],
            );
            parameters.insert(
                qubit_randomization
                    .destination_declaration
                    .name
                    .clone()
                    .into_boxed_str(),
                vec![0.0; self.unitary_set.parameters_per_unitary()],
            );
        }

        Ok(parameters)
    }

    /// Given a map of qubits to seed values, return the random indices that
    /// were drawn for each qubit during program execution.
    #[must_use]
    pub fn get_random_indices(
        &self,
        seed_values: &HashMap<u64, PrngSeedValue>,
        shot_count: u32,
    ) -> HashMap<u64, Vec<u8>> {
        seed_values
            .iter()
            .map(|(qubit, seed_value)| {
                (
                    *qubit,
                    super::random::choose_random_real_sub_region_indices(
                        *seed_value,
                        0,
                        shot_count,
                        self.unitary_count_as_u8,
                    ),
                )
            })
            .collect()
    }
}

/// A trait that defines a set of unitaries for randomized measurements. This interface includes
/// both the concrete set of unitaries from which to draw, as well as the Quil representation for
/// realizing the unitaries.
///
/// See [`tests::ZxzxzUnitarySet`] for an example implementation.
pub trait UnitarySet {
    /// An error that may occur while representing the unitary set within a set of [`Parameters`]
    /// or realizing those unitaries as Quil instructions.
    type Error;

    /// The number of unitaries in the set.
    fn unitary_count(&self) -> usize;

    /// The number of parameters required to represent a unitary within a set of Quil
    /// instructions.
    fn parameters_per_unitary(&self) -> usize;

    /// Convert the unitary set to a vector of parameters. Each unitary should be represented
    /// as a contiguous subregion within the vector of length [`Self::parameters_per_unitary`].
    /// The length of the entire vector should be equal to
    /// [`Self::unitary_count`] * [`Self::parameters_per_unitary`].
    ///
    /// See [`ChooseRandomRealSubRegions`] for additional detail.
    fn to_parameters(&self) -> Result<Vec<f64>, Self::Error>;

    /// Given a slice of [`QubitRandomization`]s, return the Quil instructions that realize the unitaries
    /// randomly drawn for each qubit. For each [`QubitRandomization`] in the slice,
    /// the memory region declared by [`QubitRandomization::get_destination_declaration`] will hold
    /// the parameters representing the randomly drawn unitary.
    fn to_instructions(
        &self,
        qubit_randomizations: &[QubitRandomization],
    ) -> Result<Vec<Instruction>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use rstest::*;

    include!("unitary_set_zxzxz.rs");

    /// A base program to which randomized measurements will be appended.
    const BASE_QUIL_PROGRAM: &str = r"
DECLARE ro BIT[3]

H 0
H 1
H 2
";

    /// A base program with randomized measurements appended.
    const BASE_QUIL_PROGRAM_WITH_RANDOMIZED_MEASUREMENTS: &str = r#"
DECLARE ro BIT[3]
DECLARE randomized_measurement_source REAL[36]
DECLARE randomized_measurement_destination_q0 REAL[3]
DECLARE randomized_measurement_seed_q0 INTEGER[1]
DECLARE randomized_measurement_destination_q1 REAL[3]
DECLARE randomized_measurement_seed_q1 INTEGER[1]
DECLARE randomized_measurement_destination_q2 REAL[3]
DECLARE randomized_measurement_seed_q2 INTEGER[1]

DELAY 0 1 2 1e-6

PRAGMA EXTERN choose_random_real_sub_regions "(destination : mut REAL[], source : REAL[], sub_region_size : INTEGER, seed : mut INTEGER)"

CALL choose_random_real_sub_regions randomized_measurement_destination_q0 randomized_measurement_source 3 randomized_measurement_seed_q0
CALL choose_random_real_sub_regions randomized_measurement_destination_q1 randomized_measurement_source 3 randomized_measurement_seed_q1
CALL choose_random_real_sub_regions randomized_measurement_destination_q2 randomized_measurement_source 3 randomized_measurement_seed_q2

H 0
H 1
H 2

FENCE

RZ(2*pi*randomized_measurement_destination_q0[0]) 0
RX(pi/2) 0
RZ(2*pi*randomized_measurement_destination_q0[1]) 0

RZ(2*pi*randomized_measurement_destination_q1[0]) 1
RX(pi/2) 1
RZ(2*pi*randomized_measurement_destination_q1[1]) 1

RZ(2*pi*randomized_measurement_destination_q2[0]) 2
RX(pi/2) 2
RZ(2*pi*randomized_measurement_destination_q2[1]) 2

FENCE

RX(pi/2) 0
RZ(2*pi*randomized_measurement_destination_q0[2]) 0

RX(pi/2) 1
RZ(2*pi*randomized_measurement_destination_q1[2]) 1

RX(pi/2) 2
RZ(2*pi*randomized_measurement_destination_q2[2]) 2

FENCE

MEASURE 0 ro[0]
MEASURE 1 ro[1]
MEASURE 2 ro[2]
"#;

    #[fixture]
    fn randomized_measurements() -> RandomizedMeasurements<ZxzxzUnitarySet> {
        let measurements = (0..3)
            .map(|i| {
                Measurement::new(
                    Qubit::Fixed(i),
                    Some(MemoryReference::new("ro".to_string(), i)),
                )
            })
            .collect();

        let unitary_set = ZxzxzUnitarySet::tetrahedral().expect("must be a valid unitary set");
        let leading_delay = Expression::Number(Complex64 { re: 1e-6, im: 0.0 });

        RandomizedMeasurements::try_new(measurements, unitary_set, leading_delay)
            .expect("must be valid randomized measurements")
    }

    /// Test that [`RandomizedMeasurements`] will add the expected Quil instructions to the
    /// [`BASE_QUIL_PROGRAM`].
    #[rstest]
    fn test_append_to_program(randomized_measurements: RandomizedMeasurements<ZxzxzUnitarySet>) {
        let randomized_program = randomized_measurements
            .append_to_program(
                Program::from_str(BASE_QUIL_PROGRAM).expect("must be valid Quil program"),
            )
            .expect("must append to program");

        let expected_program = Program::from_str(BASE_QUIL_PROGRAM_WITH_RANDOMIZED_MEASUREMENTS)
            .expect("must be valid Quil program");

        assert_eq!(randomized_program, expected_program);
    }

    #[fixture]
    /// Returns a list of valid seed values. These values are valid and correspond to test expectations,
    /// but are otherwise indeed random.
    fn seeds() -> Vec<u64> {
        vec![463_692_700, 733_101_278, 925_742_198]
    }

    #[fixture]
    fn seed_values(seeds: Vec<u64>) -> HashMap<u64, PrngSeedValue> {
        seeds
            .iter()
            .enumerate()
            .map(|(i, seed)| {
                (
                    i as u64,
                    PrngSeedValue::try_new(*seed).expect("valid seed value"),
                )
            })
            .collect()
    }

    /// Test that [`RandomizedMeasurements`] will construct the expected parameters for the
    /// randomized measurements.
    #[rstest]
    fn test_to_parameters(
        randomized_measurements: RandomizedMeasurements<ZxzxzUnitarySet>,
        seeds: Vec<u64>,
        seed_values: HashMap<u64, PrngSeedValue>,
    ) {
        let mut expected_parameters = HashMap::new();
        expected_parameters.insert(
            "randomized_measurement_source".to_string().into_boxed_str(),
            TETRAHEDRAL_UNITARY_SET_RADIANS
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<f64>>(),
        );
        seeds.iter().enumerate().for_each(|(i, seed)| {
            expected_parameters.insert(
                format!("randomized_measurement_seed_q{i}").into_boxed_str(),
                vec![seed.to_f64().expect("valid f64 seed value")],
            );
            expected_parameters.insert(
                format!("randomized_measurement_destination_q{i}").into_boxed_str(),
                vec![0.0, 0.0, 0.0],
            );
        });

        let parameters = randomized_measurements
            .to_parameters(&seed_values)
            .expect("must be valid parameters");

        assert_eq!(parameters, expected_parameters);
    }

    /// Test that [`RandomizedMeasurements`] will return the expected random indices for each qubit.
    #[rstest]
    fn test_get_random_indices(
        randomized_measurements: RandomizedMeasurements<ZxzxzUnitarySet>,
        seed_values: HashMap<u64, PrngSeedValue>,
    ) {
        let shot_count = 3;
        let expected_random_indices = [vec![0, 8, 1], vec![1, 2, 1], vec![5, 10, 5]]
            .iter()
            .enumerate()
            .map(|(i, indices)| (i as u64, indices.clone()))
            .collect::<HashMap<u64, Vec<u8>>>();
        let random_indices = randomized_measurements.get_random_indices(&seed_values, shot_count);
        assert_eq!(random_indices, expected_random_indices);
    }
}
