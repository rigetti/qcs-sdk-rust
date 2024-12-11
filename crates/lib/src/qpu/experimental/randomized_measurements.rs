//! This module contains generalized support for adding randomized measurements
//! to a Quil program. It builds off of the primitives defined in [`super::random`].
//!
//! There are three critical components to correctly adding randomized measurements
//! to a program:
//!
//! 1. Program construction - adding the classical randomization calls to the
//!     prologue of the program (i.e. before the pulse program begins) and referencing
//!     those randomized values within a unitary decomposition prior to measurement.
//! 2. Parameter construction - building a map of [`Parameters`] with seeds for
//!     each qubit.
//! 3. PRNG reconstruction - backing out the random indices that played on each
//!     qubit during program execution.
//!
//! Recall this is not a QIS (quantum information science) library, but rather an
//! SDK for collecting data from Rigetti QPUs. As such, defining the proper
//! unitary set and using randomized measurement data is beyond the scope of this
//! library.

use std::{collections::HashMap, convert::TryFrom};

use itertools::Itertools;
use ndarray::{Array2, Order};
use num::{complex::Complex64, ToPrimitive};
use quil_rs::{
    expression::Expression,
    instruction::{
        Call, CallError, Declaration, Delay, Fence, Gate, Instruction, Measurement,
        MemoryReference, Qubit, ScalarType, Vector,
    },
    quil::Quil,
    Program,
};

use super::random::PrngSeedValue;
use crate::executable::Parameters;
use crate::qpu::externed_call::ExternedCall;

/// An error that may occur when constructing randomized measurements.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Received measurement on non-fixed qubits.
    #[error("only measurements on fixed qubits are supported, found {0:?}")]
    UnsupportedMeasurementQubit(Qubit),
    /// An error occurred while constructing `PRAGMA EXTERN` instruction.
    #[error("error declaring extern function: {0}")]
    BuildPragma(#[from] super::random::Error),
    /// An error occurred while constructing a call instruction.
    #[error("error initializing call instruction: {0}")]
    BuildCall(#[from] CallError),
    /// A seed value was not provided for a qubit.
    #[error("seed not provided for qubit {}", .0.to_quil_or_debug())]
    MissingSeed(Qubit),
    /// An error occur while flattening an [`ndarray::Array`].
    #[error("shape error occurred during parameter conversion: {0}")]
    UnitariesShape(#[from] ndarray::ShapeError),
    /// A unitary set was specified with an incorrect number of columns.
    #[error("invalid unitary set; expected unitaries of length {expected}, found {found}")]
    InvalidUnitarySet {
        /// The expected number of columns
        expected: usize,
        /// The number of columns
        found: usize,
    },
    /// The number of parameters per unitary could not be expressed losslessly as
    /// an f64.
    #[error(
        "the number of parameters per unitary must be within range [0, {}], found {0}", 2_u64.pow(f64::MANTISSA_DIGITS) - 1
    )]
    ParametersPerUnitaryF64Conversion(usize),
    /// A seed value could not be expressed losslessly as an f64.
    #[error("the seed value must be within range [0, {}], found {0}", 2_u64.pow(f64::MANTISSA_DIGITS) - 1)]
    SeedValueF64Conversion(usize),
    /// The implicit unitary count could not be expressed as a u8.
    #[error("the unitary count must be within range [0, {}], found {0}", u8::MAX)]
    UnitaryCountU8Conversion(usize),
}

/// A specialized `Result` type for randomized measurements.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
struct QubitRandomization {
    seed_declaration: Declaration,
    destination_declaration: Declaration,
    measurement: Measurement,
}

impl QubitRandomization {
    fn into_instruction(
        self,
        parameters_per_unitary: f64,
        source_declaration: &Declaration,
    ) -> Result<Instruction> {
        let externed_call = super::random::ChooseRandomRealSubRegions::try_new(
            &self.destination_declaration,
            source_declaration,
            parameters_per_unitary,
            &MemoryReference {
                name: self.seed_declaration.name.clone(),
                index: 0,
            },
        )
        .unwrap();
        Ok(Instruction::Call(Call::try_from(externed_call)?))
    }

    fn destination_reference(&self, index: u64) -> MemoryReference {
        MemoryReference::new(self.destination_declaration.name.clone(), index)
    }
}

const RANDOMIZED_MEASUREMENT_SOURCE: &str = "randomized_measurement_source";
const RANDOMIZED_MEASUREMENT_DESTINATION: &str = "randomized_measurement_destination";
const RANDOMIZED_MEASUREMENT_SEED: &str = "randomized_measurement_seed";

/// Configuration for adding randomized measurements to a Quil program.
#[derive(Debug, Clone)]
pub struct RandomizedMeasurements {
    leading_delay: Expression,
    unitary_set: UnitarySet,
    unitary_count_as_u8: u8,
    qubit_randomizations: Vec<QubitRandomization>,
    source_declaration: Declaration,
}

impl RandomizedMeasurements {
    /// Initialize a new instance of [`RandomizedMeasurements`].
    ///
    /// # Parameters
    ///
    /// * `measurements` - A vector of measurements to randomize. Note, these
    ///     measurements should not be added to a program a priori.
    /// * `unitary_set` - The set of unitaries to apply to each qubit before
    ///    measurement.
    /// * `leading_delay` - The delay to prepend to the program before the
    ///   randomized measurements begin. Typically, this will be 1e-6s to 1e-5s.
    pub fn try_new(
        measurements: Vec<Measurement>,
        unitary_set: UnitarySet,
        leading_delay: Expression,
    ) -> Result<RandomizedMeasurements> {
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
                if let Qubit::Fixed(qubit) = measurement.qubit {
                    Ok((measurement, format!("q{qubit}")))
                } else {
                    Err(Error::UnsupportedMeasurementQubit(measurement.qubit))
                }
            })
            .map_ok(|(measurement, qubit_name)| QubitRandomization {
                seed_declaration: Declaration {
                    name: format!("{RANDOMIZED_MEASUREMENT_SEED}_{}", qubit_name.clone()),
                    size: Vector::new(ScalarType::Integer, 1),
                    sharing: None,
                },
                destination_declaration: Declaration {
                    name: format!(
                        "{RANDOMIZED_MEASUREMENT_DESTINATION}_{}",
                        qubit_name.clone()
                    ),
                    size: Vector::new(
                        ScalarType::Real,
                        unitary_set.parameters_per_unitary() as u64,
                    ),
                    sharing: None,
                },
                measurement,
            })
            .collect::<Result<_>>()?;
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

impl RandomizedMeasurements {
    /// Append the randomized measurements to a Quil program. The provided program
    /// should not contain any preexisting measurements.
    pub fn append_to_program(&self, target_program: Program) -> Result<Program> {
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
            super::random::ChooseRandomRealSubRegions::pragma_extern()?,
        ));

        let parameters_per_unitary = self
            .unitary_set
            .parameters_per_unitary()
            .to_f64()
            .ok_or_else(|| {
                Error::ParametersPerUnitaryF64Conversion(self.unitary_set.parameters_per_unitary())
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
            .collect::<Result<Vec<Instruction>>>()?;
        program.add_instructions(calls);

        // Include the program body that was passed in.
        program.add_instructions(target_program.into_body_instructions());

        // Play the random unitaries on each qubit.
        program.add_instructions(
            self.unitary_set
                .to_instructions(self.qubit_randomizations.as_slice()),
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
    pub fn to_parameters(&self, seed_values: &HashMap<Qubit, PrngSeedValue>) -> Result<Parameters> {
        let mut parameters = HashMap::new();
        parameters.insert(
            RANDOMIZED_MEASUREMENT_SOURCE.to_string().into_boxed_str(),
            self.unitary_set.to_parameters()?,
        );

        for qubit_randomization in &self.qubit_randomizations {
            let seed_value = seed_values
                .get(&qubit_randomization.measurement.qubit)
                .ok_or_else(|| Error::MissingSeed(qubit_randomization.measurement.qubit.clone()))?;
            parameters.insert(
                qubit_randomization
                    .seed_declaration
                    .name
                    .clone()
                    .into_boxed_str(),
                vec![seed_value.as_f64],
            );
            parameters.insert(
                qubit_randomization
                    .destination_declaration
                    .name
                    .clone()
                    .into_boxed_str(),
                std::iter::repeat(0.0)
                    .take(self.unitary_set.parameters_per_unitary())
                    .collect(),
            );
        }

        Ok(parameters)
    }

    /// Given a map of qubits to seed values, return the random indices that
    /// were played on each qubit during program execution.
    #[must_use]
    pub fn get_random_indices(
        &self,
        seed_values: &HashMap<Qubit, PrngSeedValue>,
        shot_count: u32,
    ) -> HashMap<Qubit, Vec<u8>> {
        seed_values
            .iter()
            .map(|(qubit, seed_value)| {
                (
                    qubit.clone(),
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

/// A set of unitaries, each of which may be expressed as a set of Quil
/// instructions.
#[derive(Debug, Clone)]
pub enum UnitarySet {
    /// A set of unitaries expressed as a sequence of the following gate
    /// operations:
    ///
    /// RZ(angle_0)-RX(pi/2)-RZ(angle_1)-RX(pi/2)-RZ(angle_2).
    ///
    /// The unitaries are stored in a 2D array where each row represents
    /// a single unitary expressed as the three RZ angles.
    Zxzxz(Array2<f64>),
}

impl UnitarySet {
    /// Attempt to create a new instance of [`UnitarySet`] from a 2D array
    /// of unitaries. The array must have three columns.
    pub fn try_new_zxzxz(unitaries: Array2<f64>) -> Result<UnitarySet> {
        if unitaries.ncols() != 3 {
            return Err(Error::InvalidUnitarySet {
                expected: 3,
                found: unitaries.ncols(),
            });
        }
        Ok(UnitarySet::Zxzxz(unitaries))
    }

    fn unitary_count(&self) -> usize {
        match self {
            UnitarySet::Zxzxz(unitaries) => unitaries.nrows(),
        }
    }

    const fn parameters_per_unitary(&self) -> usize {
        match self {
            UnitarySet::Zxzxz(_) => 3,
        }
    }

    fn to_parameters(&self) -> Result<Vec<f64>> {
        match self {
            UnitarySet::Zxzxz(unitaries) => Ok(unitaries
                .to_shape((unitaries.len(), Order::RowMajor))?
                .iter()
                .copied()
                .collect()),
        }
    }

    fn to_instructions(&self, qubit_randomizations: &[QubitRandomization]) -> Vec<Instruction> {
        match self {
            Self::Zxzxz(_) => Self::to_zxzxz_instructions(qubit_randomizations),
        }
    }

    fn to_zxzxz_instructions(qubit_randomizations: &[QubitRandomization]) -> Vec<Instruction> {
        let mut instructions = vec![Instruction::Fence(Fence { qubits: Vec::new() })];
        for qubit_randomization in qubit_randomizations {
            instructions.extend(vec![
                rz(
                    qubit_randomization.measurement.qubit.clone(),
                    qubit_randomization.destination_reference(0),
                ),
                rx_pi_on_2(qubit_randomization.measurement.qubit.clone()),
                rz(
                    qubit_randomization.measurement.qubit.clone(),
                    qubit_randomization.destination_reference(1),
                ),
            ]);
        }
        instructions.push(Instruction::Fence(Fence { qubits: Vec::new() }));
        for qubit_randomization in qubit_randomizations {
            instructions.extend(vec![
                rx_pi_on_2(qubit_randomization.measurement.qubit.clone()),
                rz(
                    qubit_randomization.measurement.qubit.clone(),
                    qubit_randomization.destination_reference(2),
                ),
            ]);
        }
        instructions
    }
}

fn rx_pi_on_2(qubit: Qubit) -> Instruction {
    Instruction::Gate(Gate {
        name: "RX".to_string(),
        parameters: vec![
            Expression::PiConstant / Expression::Number(Complex64 { re: 2.0, im: 0.0 }),
        ],
        qubits: vec![qubit],
        modifiers: vec![],
    })
}

fn rz(qubit: Qubit, memory_reference: MemoryReference) -> Instruction {
    Instruction::Gate(Gate {
        name: "RZ".to_string(),
        parameters: vec![
            Expression::Number(Complex64 { re: 2.0, im: 0.0 })
                * Expression::PiConstant
                * Expression::Address(memory_reference),
        ],
        qubits: vec![qubit],
        modifiers: vec![],
    })
}

#[cfg(test)]
mod tests {
    use core::f64;
    use std::str::FromStr;

    use super::*;
    use rstest::*;

    const BASE_QUIL_PROGRAM: &str = r"
DECLARE ro BIT[3]

H 0
H 1
H 2
";

    const BASE_QUIL_PROGRAM_WITH_MEASUREMENTS: &str = r#"
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
    fn unitary_set() -> Vec<f64> {
        vec![
            0.,
            f64::consts::FRAC_PI_2,
            -f64::consts::FRAC_PI_2,
            f64::consts::PI,
            f64::consts::FRAC_PI_2,
            -f64::consts::FRAC_PI_2,
            0.,
            f64::consts::FRAC_PI_2,
            f64::consts::FRAC_PI_2,
            f64::consts::PI,
            f64::consts::FRAC_PI_2,
            f64::consts::FRAC_PI_2,
            -f64::consts::FRAC_PI_2,
            f64::consts::FRAC_PI_2,
            f64::consts::PI,
            -f64::consts::FRAC_PI_2,
            f64::consts::FRAC_PI_2,
            0.,
            f64::consts::FRAC_PI_2,
            f64::consts::FRAC_PI_2,
            f64::consts::PI,
            f64::consts::FRAC_PI_2,
            f64::consts::FRAC_PI_2,
            0.,
            f64::consts::FRAC_PI_2,
            f64::consts::PI,
            -f64::consts::FRAC_PI_2,
            f64::consts::PI,
            f64::consts::PI,
            0.,
            -f64::consts::FRAC_PI_2,
            0.,
            -f64::consts::FRAC_PI_2,
            0.,
            0.,
            f64::consts::PI,
        ]
    }

    #[fixture]
    fn randomized_measurements(unitary_set: Vec<f64>) -> RandomizedMeasurements {
        let measurements = (0..3)
            .map(|i| {
                Measurement::new(
                    Qubit::Fixed(i),
                    Some(MemoryReference::new("ro".to_string(), i)),
                )
            })
            .collect();

        let unitary_set = UnitarySet::try_new_zxzxz(
            Array2::from_shape_vec((12, 3), unitary_set).expect("must be valid unitary array"),
        )
        .expect("valid unitary set");
        let leading_delay = Expression::Number(Complex64 { re: 1e-6, im: 0.0 });

        RandomizedMeasurements::try_new(measurements, unitary_set, leading_delay)
            .expect("must be valid randomized measurements")
    }

    #[rstest]
    fn test_append_to_program(randomized_measurements: RandomizedMeasurements) {
        let randomized_program = randomized_measurements
            .append_to_program(
                Program::from_str(BASE_QUIL_PROGRAM).expect("must be valid Quil program"),
            )
            .expect("must append to program");

        let expected_program = Program::from_str(BASE_QUIL_PROGRAM_WITH_MEASUREMENTS)
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
    fn seed_values(seeds: Vec<u64>) -> HashMap<Qubit, PrngSeedValue> {
        seeds
            .iter()
            .enumerate()
            .map(|(i, seed)| {
                (
                    Qubit::Fixed(i as u64),
                    PrngSeedValue::try_new(*seed).expect("valid seed value"),
                )
            })
            .collect()
    }

    #[rstest]
    fn test_to_parameters(
        randomized_measurements: RandomizedMeasurements,
        unitary_set: Vec<f64>,
        seeds: Vec<u64>,
        seed_values: HashMap<Qubit, PrngSeedValue>,
    ) {
        let mut expected_parameters = HashMap::new();
        expected_parameters.insert(
            "randomized_measurement_source".to_string().into_boxed_str(),
            unitary_set,
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

    #[rstest]
    fn test_get_random_indices(
        randomized_measurements: RandomizedMeasurements,
        seed_values: HashMap<Qubit, PrngSeedValue>,
    ) {
        let shot_count = 3;
        let expected_random_indices = [vec![0, 8, 1], vec![1, 2, 1], vec![5, 10, 5]]
            .iter()
            .enumerate()
            .map(|(i, indices)| (Qubit::Fixed(i as u64), indices.clone()))
            .collect::<HashMap<Qubit, Vec<u8>>>();
        let random_indices = randomized_measurements.get_random_indices(&seed_values, shot_count);
        assert_eq!(random_indices, expected_random_indices);
    }
}
