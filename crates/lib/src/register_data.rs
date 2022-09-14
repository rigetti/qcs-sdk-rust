use std::collections::HashMap;

use enum_as_inner::EnumAsInner;
use num::complex::Complex32;
use serde::{Deserialize, Serialize};

use crate::qpu::{DecodeError, Register};

/// Data resulting from [`Executable::execute_on_qvm`][`crate::Executable::execute_on_qvm`] or
/// [`Executable::execute_on_qpu`][`crate::Executable::execute_on_qpu`].
///
/// This represents a single vector (or "register") of typed memory across some number of shots.
/// The register corresponds to the usage of a `DECLARE` instruction in Quil, and the name of that
/// register should be provided with [`Executable::read_from`][`crate::Executable::read_from`].
///
/// There is a variant of this enum for each type of data that a register could hold.
/// Any variant of an instance of `ExecutionResult` will contain a `Vec` with one entry for each shot,
/// where each entry represents the entire register.
///
/// # Usage
///
/// Typically you will already know what type of data the `ExecutionResult` _should_ have, so you can
/// use the [`mod@enum_as_inner`] methods (e.g. [`ExecutionResult::into_i8`]) in order to
/// convert any variant type to its inner data.
#[derive(Clone, Debug, Deserialize, EnumAsInner, PartialEq, Serialize)]
#[serde(untagged)]
pub enum RegisterData {
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

impl RegisterData {
    pub(crate) fn try_from_registers(
        registers_by_name: HashMap<Box<str>, Vec<Register>>,
        shots: u16,
    ) -> Result<HashMap<Box<str>, Self>, DecodeError> {
        registers_by_name
            .into_iter()
            .map(|(register_name, registers)| {
                let first_register = registers
                    .get(0)
                    .ok_or_else(|| DecodeError::MissingBuffer(register_name.to_string()))?;
                match first_register {
                    Register::I8(_) => Self::try_from_i8_registers(registers, shots),
                    Register::I16(_) => Self::try_from_i16_registers(registers, shots),
                    Register::F64(_) => Self::try_from_f64_registers(registers, shots),
                    Register::Complex32(_) => Self::try_from_complex32_registers(registers, shots),
                }
                .map(|execution_result| (register_name, execution_result))
            })
            .collect()
    }

    fn try_from_i8_registers(
        registers: Vec<Register>,
        number_of_shots: u16,
    ) -> Result<Self, DecodeError> {
        let registers = registers
            .into_iter()
            .map(|register| register.into_i8().map_err(|_| DecodeError::MixedTypes))
            .collect::<Result<Vec<Vec<i8>>, DecodeError>>()?;

        Ok(Self::I8(transpose(registers, number_of_shots)))
    }

    fn try_from_i16_registers(
        registers: Vec<Register>,
        number_of_shots: u16,
    ) -> Result<Self, DecodeError> {
        let registers = registers
            .into_iter()
            .map(|register| register.into_i16().map_err(|_| DecodeError::MixedTypes))
            .collect::<Result<Vec<Vec<i16>>, DecodeError>>()?;

        Ok(Self::I16(transpose(registers, number_of_shots)))
    }

    fn try_from_f64_registers(
        registers: Vec<Register>,
        number_of_shots: u16,
    ) -> Result<Self, DecodeError> {
        let registers = registers
            .into_iter()
            .map(|register| register.into_f64().map_err(|_| DecodeError::MixedTypes))
            .collect::<Result<Vec<Vec<f64>>, DecodeError>>()?;

        Ok(Self::F64(transpose(registers, number_of_shots)))
    }

    fn try_from_complex32_registers(
        registers: Vec<Register>,
        number_of_shots: u16,
    ) -> Result<Self, DecodeError> {
        let registers = registers
            .into_iter()
            .map(|register| {
                register
                    .into_complex32()
                    .map_err(|_| DecodeError::MixedTypes)
            })
            .collect::<Result<Vec<Vec<Complex32>>, DecodeError>>()?;

        Ok(Self::Complex32(transpose(registers, number_of_shots)))
    }
}

fn transpose<T>(mut data: Vec<Vec<T>>, len: u16) -> Vec<Vec<T>> {
    let mut results = Vec::with_capacity(len as usize);

    for _ in 0..len {
        let mut new_inner = Vec::with_capacity(data.len());
        for old_inner in &mut data {
            new_inner.push(old_inner.remove(0));
        }
        results.push(new_inner);
    }
    results
}

#[cfg(test)]
mod describe_program_results {
    use maplit::hashmap;
    use num::complex::Complex32;

    use crate::{qpu::Register, RegisterData};

    #[test]
    fn it_converts_from_i8_registers() {
        let registers = hashmap! {
            Box::from(String::from("ro")) => vec![Register::I8(vec![1, 2, 3]), Register::I8(vec![4, 5, 6])]
        };
        let results = RegisterData::try_from_registers(registers, 3).unwrap();
        let expected = hashmap! {
            Box::from(String::from("ro")) => RegisterData::I8(vec![vec![1, 4], vec![2, 5], vec![3, 6]])
        };
        assert_eq!(results, expected);
    }

    #[test]
    fn it_converts_from_i16_registers() {
        let registers = hashmap! {
            Box::from(String::from("ro")) => vec![Register::I16(vec![1, 2, 3]), Register::I16(vec![4, 5, 6])]
        };
        let results = RegisterData::try_from_registers(registers, 3).unwrap();
        let expected = hashmap! {
            Box::from(String::from("ro")) => RegisterData::I16(vec![vec![1, 4], vec![2, 5], vec![3, 6]])
        };
        assert_eq!(results, expected);
    }

    #[test]
    fn it_converts_from_f64_registers() {
        let registers = hashmap! {
            Box::from(String::from("ro")) => vec![
                Register::F64(vec![1.0, 2.0, 3.0]),
                Register::F64(vec![4.0, 5.0, 6.0]),
            ]
        };
        let results = RegisterData::try_from_registers(registers, 3).unwrap();
        let expected = hashmap! {
            Box::from(String::from("ro")) => RegisterData::F64(vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]])
        };
        assert_eq!(results, expected);
    }

    #[test]
    fn it_converts_from_complex32_registers() {
        let registers = hashmap! {
            Box::from(String::from("ro")) => vec![
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
            ]
        };
        let results = RegisterData::try_from_registers(registers, 3).unwrap();
        let expected = hashmap! {
            Box::from(String::from("ro")) => RegisterData::Complex32(vec![
                vec![Complex32::from(1.0), Complex32::from(4.0)],
                vec![Complex32::from(2.0), Complex32::from(5.0)],
                vec![Complex32::from(3.0), Complex32::from(6.0)],
            ])
        };
        assert_eq!(results, expected);
    }

    #[test]
    fn it_errors_on_mismatch_types() {
        let registers = hashmap! {
            Box::from(String::from("ro")) => vec![Register::I8(vec![1, 2, 3]), Register::I16(vec![4, 5, 6])]
        };
        let results = RegisterData::try_from_registers(registers, 3);
        assert!(results.is_err());
    }

    #[test]
    fn it_handles_mixed_types() {
        let registers = hashmap! {
            Box::from(String::from("first")) => vec![Register::I8(vec![1, 2, 3]), Register::I8(vec![4, 5, 6])],
            Box::from(String::from("second")) => vec![Register::I16(vec![1, 2, 3]), Register::I16(vec![4, 5, 6])],
        };
        let results = RegisterData::try_from_registers(registers, 3).unwrap();
        let expected = hashmap! {
            Box::from(String::from("first")) => RegisterData::I8(vec![vec![1, 4], vec![2, 5], vec![3, 6]]),
            Box::from(String::from("second")) => RegisterData::I16(vec![vec![1, 4], vec![2, 5], vec![3, 6]]),
        };
        assert_eq!(results, expected);
    }
}
