use core::f64;
use ndarray::{Array2, Order};
use num::complex::Complex64;
use quil_rs::expression::Expression;
use quil_rs::instruction::{Fence, Gate, Instruction, MemoryReference, Qubit};

const TETRAHEDRAL_UNITARY_SET_RADIANS: [[f64; 3]; 12] = [
    [0., f64::consts::FRAC_PI_2, -f64::consts::FRAC_PI_2],
    [
        f64::consts::PI,
        f64::consts::FRAC_PI_2,
        -f64::consts::FRAC_PI_2,
    ],
    [0., f64::consts::FRAC_PI_2, f64::consts::FRAC_PI_2],
    [
        f64::consts::PI,
        f64::consts::FRAC_PI_2,
        f64::consts::FRAC_PI_2,
    ],
    [
        -f64::consts::FRAC_PI_2,
        f64::consts::FRAC_PI_2,
        f64::consts::PI,
    ],
    [-f64::consts::FRAC_PI_2, f64::consts::FRAC_PI_2, 0.],
    [
        f64::consts::FRAC_PI_2,
        f64::consts::FRAC_PI_2,
        f64::consts::PI,
    ],
    [f64::consts::FRAC_PI_2, f64::consts::FRAC_PI_2, 0.],
    [
        f64::consts::FRAC_PI_2,
        f64::consts::PI,
        -f64::consts::FRAC_PI_2,
    ],
    [f64::consts::PI, f64::consts::PI, 0.],
    [-f64::consts::FRAC_PI_2, 0., -f64::consts::FRAC_PI_2],
    [0., 0., f64::consts::PI],
];

/// An error that may occur when constructing randomized measurements.
#[derive(Debug, thiserror::Error)]
enum ZxzxzError {
    /// An error occur while flattening an [`ndarray::Array`].
    #[error("shape error occurred during parameter conversion: {0}")]
    UnitariesShape(#[from] ndarray::ShapeError),
}

/// A set of unitaries, each of which may be expressed as a set of Quil
/// instructions.
///
/// RZ(angle_0)-RX(pi/2)-RZ(angle_1)-RX(pi/2)-RZ(angle_2)
///
/// The unitaries are stored in a 2D array where each row represents
/// a single unitary expressed as the three RZ angles. The array must have
/// three columns.
#[derive(Debug, Clone)]
struct ZxzxzUnitarySet(Array2<f64>);

impl ZxzxzUnitarySet {
    /// Create a new unitary set with angles representing the
    /// [tetrahedral unitary ensemble](https://en.wikipedia.org/wiki/Tetrahedral_symmetry).
    fn tetrahedral() -> Result<Self, ZxzxzError> {
        Ok(Self(Array2::from_shape_vec(
            (12, 3),
            TETRAHEDRAL_UNITARY_SET_RADIANS
                .iter()
                .flatten()
                .copied()
                .collect(),
        )?))
    }
}

impl UnitarySet for ZxzxzUnitarySet {
    type Error = ZxzxzError;

    fn unitary_count(&self) -> usize {
        self.0.nrows()
    }

    fn parameters_per_unitary(&self) -> usize {
        3
    }

    fn to_parameters(&self) -> Result<Vec<f64>, Self::Error> {
        Ok(self
            .0
            .to_shape((self.0.len(), Order::RowMajor))?
            .iter()
            .copied()
            .collect())
    }

    fn to_instructions(
        &self,
        qubit_randomizations: &[QubitRandomization],
    ) -> Result<Vec<Instruction>, Self::Error> {
        let mut instructions = vec![Instruction::Fence(Fence { qubits: Vec::new() })];
        for qubit_randomization in qubit_randomizations {
            instructions.extend(vec![
                rz(
                    qubit_randomization.get_measurement().qubit.clone(),
                    MemoryReference::new(
                        qubit_randomization
                            .get_destination_declaration()
                            .name
                            .clone(),
                        0,
                    ),
                ),
                rx_pi_over_2(qubit_randomization.get_measurement().qubit.clone()),
                rz(
                    qubit_randomization.get_measurement().qubit.clone(),
                    MemoryReference::new(
                        qubit_randomization
                            .get_destination_declaration()
                            .name
                            .clone(),
                        1,
                    ),
                ),
            ]);
        }
        instructions.push(Instruction::Fence(Fence { qubits: Vec::new() }));
        for qubit_randomization in qubit_randomizations {
            instructions.extend(vec![
                rx_pi_over_2(qubit_randomization.get_measurement().qubit.clone()),
                rz(
                    qubit_randomization.get_measurement().qubit.clone(),
                    MemoryReference::new(
                        qubit_randomization
                            .get_destination_declaration()
                            .name
                            .clone(),
                        2,
                    ),
                ),
            ]);
        }
        Ok(instructions)
    }
}

fn rx_pi_over_2(qubit: Qubit) -> Instruction {
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
