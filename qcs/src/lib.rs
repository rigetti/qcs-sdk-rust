#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! This crate is the primary Rust API for interacting with Rigetti products. Specifically, this
//! crate allows you to run Quil programs against real QPUs using [`qpu::run_program`] or a QVM
//! using [`qvm::run_program`].

use enum_as_inner::EnumAsInner;
use eyre::{eyre, Result, WrapErr};
use num::complex::Complex32;
use serde::Deserialize;

use crate::qpu::Register;

pub mod configuration;
pub mod qpu;
pub mod qvm;

/// Data resulting from a run of a Quil program.
///
/// This represents a single vector (or "register") of typed memory across some number of shots.
/// The register corresponds to the usage of a `DECLARE` instruction in Quil, and the name of that
/// register should be provided to one of [`qvm::run_program`] or [`qpu::run_program`] to indicate
/// that it contains the final results of your program.
///
/// There is a variant of this enum for each type of data that a register could hold.
/// Any variant of an instance of `ProgramResult` will contain a `Vec` with one entry for each shot,
/// where each entry represents the entire register.
///
/// # Usage
///
/// Typically you will already know what type of data the `ProgramResult` _should_ have, so you can
/// use the [enum-as-inner](https://docs.rs/enum-as-inner/0.3.3/enum_as_inner/) methods in order to
/// convert any variant type to its inner data.
///
/// # Example
///
/// ```rust
/// use qcs::{ProgramResult, qvm};
///
/// const PROGRAM: &str = r##"
/// DECLARE ro BIT[2]
///
/// H 0
/// CNOT 0 1
///
/// MEASURE 0 ro[0]
/// MEASURE 1 ro[1]
/// "##;
///
/// #[tokio::main]
/// async fn main() {
///     // Here we indicate to `qcs` that the `"ro"` register contains the data we'd like in our `ProgramResult`
///     let result: ProgramResult = qvm::run_program(PROGRAM, 4, "ro").await.unwrap();
///     // We know it's i8 because we declared the memory as `BIT` in Quil.
///     let data = result.into_i8().unwrap();
///     // In this case, we ran the program for 4 shots, so we know the length is 4.///
///     assert_eq!(data.len(), 4);///
///     for shot in data {
///         // Each shot will contain all the memory, in order, for the vector (or "register") we
///         // requested the results of. In this case, "ro".
///         assert_eq!(shot.len(), 2);
///         // In the case of this particular program, we know ro[0] should equal ro[1]
///         assert_eq!(shot[0], shot[1]);
///     }
/// }
///
/// ```
#[derive(Debug, Deserialize, EnumAsInner, PartialEq)]
#[serde(untagged)]
pub enum ProgramResult {
    /// Corresponds to the Quil `BIT` or `OCTET` types.
    I8(Vec<Vec<i8>>),
    /// Corresponds to the Quil `REAL` type.
    F64(Vec<Vec<f64>>),
    /// Corresponds to the Quil `INTEGER` type.
    I16(Vec<Vec<i16>>),
    /// Results containing complex numbers.
    #[serde(skip)]
    Complex32(Vec<Vec<Complex32>>),
}

impl ProgramResult {
    fn try_from_registers(registers: Vec<Register>, shots: u16) -> Result<Self> {
        if registers.is_empty() {
            return Err(eyre!(
                "No data received for register, did you forget to MEASURE to it?"
            ));
        }
        let first_register = registers.get(0).expect("Length checked above");
        match first_register {
            Register::I8(_) => Self::try_from_i8_registers(registers, shots),
            Register::I16(_) => Self::try_from_i16_registers(registers, shots),
            Register::F64(_) => Self::try_from_f64_registers(registers, shots),
            Register::Complex32(_) => Self::try_from_complex32_registers(registers, shots),
        }
    }

    fn try_from_i8_registers(registers: Vec<Register>, number_of_shots: u16) -> Result<Self> {
        let registers = registers
            .into_iter()
            .map(|register| register.into_i8().map_err(|_| eyre!("Cannot convert")))
            .collect::<Result<Vec<Vec<i8>>>>()
            .wrap_err("One or more registers had the wrong type")?;

        Ok(Self::I8(transpose(registers, number_of_shots)))
    }

    fn try_from_i16_registers(registers: Vec<Register>, number_of_shots: u16) -> Result<Self> {
        let registers = registers
            .into_iter()
            .map(|register| register.into_i16().map_err(|_| eyre!("Cannot convert")))
            .collect::<Result<Vec<Vec<i16>>>>()
            .wrap_err("One or more registers had the wrong type")?;

        Ok(Self::I16(transpose(registers, number_of_shots)))
    }

    fn try_from_f64_registers(registers: Vec<Register>, number_of_shots: u16) -> Result<Self> {
        let registers = registers
            .into_iter()
            .map(|register| register.into_f64().map_err(|_| eyre!("Cannot convert")))
            .collect::<Result<Vec<Vec<f64>>>>()
            .wrap_err("One or more registers had the wrong type")?;

        Ok(Self::F64(transpose(registers, number_of_shots)))
    }

    fn try_from_complex32_registers(
        registers: Vec<Register>,
        number_of_shots: u16,
    ) -> Result<Self> {
        let registers = registers
            .into_iter()
            .map(|register| {
                register
                    .into_complex32()
                    .map_err(|_| eyre!("Cannot convert"))
            })
            .collect::<Result<Vec<Vec<Complex32>>>>()
            .wrap_err("One or more registers had the wrong type")?;

        Ok(Self::Complex32(transpose(registers, number_of_shots)))
    }
}

fn transpose<T>(mut data: Vec<Vec<T>>, len: u16) -> Vec<Vec<T>> {
    let mut results = Vec::with_capacity(len as usize);

    for _ in 0..len {
        let mut new_inner = Vec::with_capacity(data.len());
        for old_inner in &mut data {
            new_inner.push(old_inner.remove(0))
        }
        results.push(new_inner);
    }
    results
}

#[cfg(test)]
mod describe_program_results {
    use super::*;

    #[test]
    fn it_converts_from_i8_registers() {
        let registers = vec![Register::I8(vec![1, 2, 3]), Register::I8(vec![4, 5, 6])];
        let results = ProgramResult::try_from_registers(registers, 3).unwrap();
        let expected = ProgramResult::I8(vec![vec![1, 4], vec![2, 5], vec![3, 6]]);
        assert_eq!(results, expected);
    }

    #[test]
    fn it_converts_from_i16_registers() {
        let registers = vec![Register::I16(vec![1, 2, 3]), Register::I16(vec![4, 5, 6])];
        let results = ProgramResult::try_from_registers(registers, 3).unwrap();
        let expected = ProgramResult::I16(vec![vec![1, 4], vec![2, 5], vec![3, 6]]);
        assert_eq!(results, expected);
    }

    #[test]
    fn it_converts_from_f64_registers() {
        let registers = vec![
            Register::F64(vec![1.0, 2.0, 3.0]),
            Register::F64(vec![4.0, 5.0, 6.0]),
        ];
        let results = ProgramResult::try_from_registers(registers, 3).unwrap();
        let expected = ProgramResult::F64(vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]]);
        assert_eq!(results, expected);
    }

    #[test]
    fn it_converts_from_complex32_registers() {
        let registers = vec![
            Register::Complex32(vec![
                Complex32::from(1.0),
                Complex32::from(2.0),
                Complex32::from(3.0),
            ]),
            Register::Complex32(vec![
                Complex32::from(4.0),
                Complex32::from(5.0),
                Complex32::from(6.0),
            ]),
        ];
        let results = ProgramResult::try_from_registers(registers, 3).unwrap();
        let expected = ProgramResult::Complex32(vec![
            vec![Complex32::from(1.0), Complex32::from(4.0)],
            vec![Complex32::from(2.0), Complex32::from(5.0)],
            vec![Complex32::from(3.0), Complex32::from(6.0)],
        ]);
        assert_eq!(results, expected);
    }

    #[test]
    fn it_errors_on_mismatch_types() {
        let registers = vec![Register::I8(vec![1, 2, 3]), Register::I16(vec![4, 5, 6])];
        let results = ProgramResult::try_from_registers(registers, 3);
        assert!(results.is_err());
    }
}
