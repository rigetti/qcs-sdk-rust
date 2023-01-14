use num::complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;

use ndarray::prelude::*;

use crate::qpu::readout_data::ReadoutValues;
use crate::{qpu::readout_data::QPUReadout, qvm::QVMMemory, RegisterData};

/// TODO: Docstring
#[derive(Debug, Clone, PartialEq)]
pub enum ReadoutData {
    Qvm(QVMMemory),
    Qpu(QPUReadout),
}

/// The result of executing an [`Executable`](crate::Executable)
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionData {
    /// The readout data that was read from the [`Executable`](crate::Executable).
    /// Key is the name of the register, value is the data of the register after execution.
    /// TODO: expand docstring
    pub readout_data: ReadoutData,
    /// The time it took to execute the program on the QPU, not including any network or queueing
    /// time. If paying for on-demand execution, this is the amount you will be billed for.
    ///
    /// This will always be `None` for QVM execution.
    pub duration: Option<Duration>,
}

/// An enum representing every possible readout type
/// TODO: Doc, Option?
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RegisterMatrix {
    /// An integer readout value
    Integer(Array2<i32>),
    /// A real numbered readout value
    Real(Array2<f64>),
    /// A complex numbered readout value
    Complex(Array2<Complex64>),
}

/// TODO: Doc
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ReadoutMap(HashMap<String, RegisterMatrix>);

/// Errors that may occur when trying to interpret [`RegisterMatrix`] values as a single data type.
/// TODO: Refactor as error type for translating readout data to ``RegisterMatrix``
#[derive(Copy, Clone, Debug, thiserror::Error)]
pub enum RegisterMatrixConversionError {
    /// The value at a given position was an unexpected format.
    #[error("Could not convert data to a register matrix as it was not rectangular")]
    InvalidShape,

    /// The value at a given position was an unexpected format.
    #[error("Could not parse the memory reference")]
    MemoryReferenceParseError,
}

impl ReadoutMap {
    /// Returns a [`ReadoutMap`] with the underlying [`RegisterMatrix`] data
    #[must_use]
    pub fn from_hashmap(map: HashMap<String, RegisterMatrix>) -> Self {
        Self(map)
    }

    /// Returns a [`ReadoutMap`] built from [`QVMMemory`]
    #[must_use]
    pub fn from_qvm_memory(memory: &QVMMemory) -> Self {
        Self(
            memory
                .iter()
                .map(|(name, register)| {
                    let register_matrix = match register {
                        RegisterData::I8(data) => RegisterMatrix::Integer(data.mapv(i32::from)),
                        RegisterData::I16(data) => RegisterMatrix::Integer(data.mapv(i32::from)),
                        RegisterData::F64(data) => RegisterMatrix::Real(data.mapv(f64::from)),
                        RegisterData::Complex32(data) => RegisterMatrix::Complex(
                            data.map(|c| Complex64::new(c.re.into(), c.im.into())),
                        ),
                    };
                    (name.clone(), register_matrix)
                })
                .collect(),
        )
    }

    /// TODO: Docs, cleanup, error messages
    pub fn from_qpu_readout_data(
        readout_data: &QPUReadout,
    ) -> Result<Self, RegisterMatrixConversionError> {
        Ok(
            Self(
                readout_data
                    .mappings
                    .iter()
                    // Pair all the memory references with their readout values
                    .map(|(memory_reference, alias)| {
                        Ok((
                            parse_readout_register(memory_reference).map_err(|_| {
                                RegisterMatrixConversionError::MemoryReferenceParseError
                            })?,
                            readout_data
                                .readout_values
                                .get(alias)
                                .ok_or(RegisterMatrixConversionError::InvalidShape)?,
                        ))
                    })
                    // Collect into a type that will sort them by memory reference, this allows us
                    // to make sure indices are sequential.
                    .collect::<Result<
                        BTreeMap<MemoryReference, &ReadoutValues>,
                        RegisterMatrixConversionError,
                    >>()?
                    .iter()
                    // Iterate over them in reverse. Starting with the last index lets us:
                    //     (1): Initialize the RegisterMatrix with the correct number of rows
                    //     (2): Use a memory reference with an index of 0 as a setinel.
                    .rev()
                    .try_fold(
                        (
                            HashMap::with_capacity(readout_data.readout_values.len()),
                            None::<&MemoryReference>,
                        ),
                        |(mut readout_data, previous_reference), (reference, values)| {
                            // If we haven't started on a new register, make sure we aren't
                            // skipping an index. For example, if we jumped from ro[5] to ro[3], or
                            // ro[2] to theta[1], we skipped a column.
                            if let Some(previous) = previous_reference {
                                if previous.name != reference.name
                                    || previous.index != reference.index + 1
                                {
                                    return Err(RegisterMatrixConversionError::InvalidShape);
                                }
                            }

                            let matrix = readout_data.entry(reference.name.clone()).or_insert(
                                match values {
                                    ReadoutValues::Integer(v) => RegisterMatrix::Integer(
                                        Array2::zeros((v.len(), reference.index + 1)),
                                    ),
                                    ReadoutValues::Complex(v) => RegisterMatrix::Complex(
                                        Array2::zeros((v.len(), reference.index + 1)),
                                    ),
                                },
                            );

                            // Insert the readout values as a column iff it fits within the
                            // dimensions of the matrix. Otherwise, the readout data must be
                            // jagged and we return an error.
                            match (matrix, values) {
                                (RegisterMatrix::Integer(m), ReadoutValues::Integer(v))
                                    if m.nrows() == v.len() =>
                                {
                                    m.column_mut(reference.index)
                                        .assign(&Array::from_vec(v.clone()));
                                }
                                (RegisterMatrix::Complex(m), ReadoutValues::Complex(v))
                                    if m.nrows() == v.len() =>
                                {
                                    m.column_mut(reference.index)
                                        .assign(&Array::from_vec(v.clone()));
                                }
                                _ => return Err(RegisterMatrixConversionError::InvalidShape),
                            }

                            let previous_reference = if reference.index == 0 {
                                None
                            } else {
                                Some(reference)
                            };

                            Ok((readout_data, previous_reference))
                        },
                    )?
                    .0,
            ),
        )
    }
}

// This is a copy of [`quil_rs::instruction::MemoryReference`] that uses `usize` for the index
// instead of `u64` for compatibility with the containers we use for [`ReadoutMap`].
// It's possible `quil_rs` will use `usize` for it's `MemoryReference` in the future. If so, we
// should use it to replace this.
// See https://github.com/rigetti/qcs-sdk-rust/issues/224
#[derive(PartialEq, PartialOrd, Eq, Ord)]
struct MemoryReference {
    name: String,
    index: usize,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MemoryReferenceParseError {
    #[error("Could not parse memory reference: {reason}")]
    InvalidFormat { reason: String },

    #[error("Could not parse index from reference name: {0}")]
    InvalidIndex(#[from] ParseIntError),
}

// Note: MemoryReference may have a from_string in the fututre
fn parse_readout_register(
    register_name: &str,
) -> Result<MemoryReference, MemoryReferenceParseError> {
    let open_brace =
        register_name
            .find('[')
            .ok_or_else(|| MemoryReferenceParseError::InvalidFormat {
                reason: "Opening brace not found".into(),
            })?;
    let close_brace =
        register_name
            .find(']')
            .ok_or_else(|| MemoryReferenceParseError::InvalidFormat {
                reason: "Closing brace not found".into(),
            })?;

    Ok(MemoryReference {
        name: String::from(&register_name[..open_brace]),
        index: usize::from_str(&register_name[open_brace + 1..close_brace])?,
    })
}

// TODO: Re-instate tests
// #[cfg(test)]
// mod describe_readout_map {
//     use maplit::hashmap;
//     use ndarray::prelude::*;
//     use std::collections::HashMap;
//     use test_case::test_case;
//
//     use super::{ReadoutMap, RegisterData};
//     use qcs_api_client_grpc::models::controller::readout_values::Values;
//     use qcs_api_client_grpc::models::controller::IntegerReadoutValues;
//
//     fn dummy_readout_values(v: i32) -> ReadoutValues {
//         ReadoutValues {
//             values: Some(Values::IntegerValues(IntegerReadoutValues {
//                 values: vec![v],
//             })),
//         }
//     }
//
//     #[test]
//     fn it_converts_from_translation_readout_mappings() {
//         let readout_mappings = hashmap! {
//             String::from("ro[1]") => String::from("qA"),
//             String::from("ro[2]") => String::from("qB"),
//             String::from("ro[0]") => String::from("qC"),
//             String::from("bar[3]") => String::from("qD"),
//             String::from("bar[5]") => String::from("qE"),
//         };
//
//         let readout_values = hashmap! {
//             String::from("qA") => dummy_readout_values(11),
//             String::from("qB") => dummy_readout_values(22),
//             String::from("qD") => dummy_readout_values(33),
//             String::from("qE") => dummy_readout_values(44),
//         };
//
//         let readout_map = ReadoutMap::from_mappings_and_values(&readout_mappings, &readout_values);
//         let register = readout_map
//             .get_index_wise_matrix("ro")
//             .expect("ReadoutMap should have ro");
//
//         let expected = arr2(&[
//             [None],
//             [Some(ReadoutValue::Integer(11))],
//             [Some(ReadoutValue::Integer(22))],
//         ]);
//
//         assert_eq!(register, expected);
//
//         let bar = readout_map
//             .get_index_wise_matrix("bar")
//             .expect("ReadoutMap should have field `bar`");
//
//         let expected = arr2(&[
//             [None],
//             [None],
//             [None],
//             [Some(ReadoutValue::Integer(33))],
//             [None],
//             [Some(ReadoutValue::Integer(44))],
//         ]);
//
//         assert_eq!(bar, expected);
//     }
//
//     #[test]
//     fn it_converts_from_register_data_map() {
//         let registers: HashMap<Box<str>, RegisterData> = hashmap! {
//             String::from("ro").into() => RegisterData::I8(vec![vec![1, 0, 1]]),
//         };
//
//         let readout_map = ReadoutMap::from_register_data_map(&registers);
//
//         let ro = readout_map
//             .get_index_wise_matrix("ro")
//             .expect("ReadoutMap should have ro");
//
//         let expected = arr2(&[
//             [Some(ReadoutValue::Integer(1))],
//             [Some(ReadoutValue::Integer(0))],
//             [Some(ReadoutValue::Integer(1))],
//         ]);
//         assert_eq!(ro, expected);
//     }
//
//     #[test_case(0, 0 => Some(ReadoutValue::Integer(1)) ; "returns value when both indices are in bounds")]
//     #[test_case(1, 0 => None ; "returns None when row index is off by 1")]
//     #[test_case(0, 1 => None ; "returns None when column index is off by 1")]
//     fn get_value(row: usize, col: usize) -> Option<ReadoutValue> {
//         let registers: HashMap<Box<str>, RegisterData> = hashmap! {
//             String::from("ro").into() => RegisterData::I8(vec![vec![1]]),
//         };
//
//         ReadoutMap::from_register_data_map(&registers).get_value("ro", row, col)
//     }
//
//     #[test_case(0 => Some(vec![Some(ReadoutValue::Integer(1)), Some(ReadoutValue::Integer(2))]) ; "returns correct row when index is in bounds")]
//     #[test_case(2 => None ; "returns None when index is off by 1")]
//     fn get_values_by_memory_index(index: usize) -> Option<Vec<Option<ReadoutValue>>> {
//         let registers: HashMap<Box<str>, RegisterData> = hashmap! {
//             String::from("ro").into() => RegisterData::I8(vec![vec![1, 9], vec![2, 10]]),
//         };
//
//         ReadoutMap::from_register_data_map(&registers).get_values_by_memory_index("ro", index)
//     }
//
//     #[test_case(0 => Some(vec![Some(ReadoutValue::Integer(1)), Some(ReadoutValue::Integer(2))]) ; "returns correct column when index is in bounds")]
//     #[test_case(2 => None ; "returns None when index is off by 1")]
//     fn get_values_by_shot(shot_num: usize) -> Option<Vec<Option<ReadoutValue>>> {
//         let registers: HashMap<Box<str>, RegisterData> = hashmap! {
//             String::from("ro").into() => RegisterData::I8(vec![vec![1, 2], vec![3, 4]]),
//         };
//
//         ReadoutMap::from_register_data_map(&registers).get_values_by_shot("ro", shot_num)
//     }
//
//     #[test]
//     fn test_get_index_wise_matrix() {
//         let registers: HashMap<Box<str>, RegisterData> = hashmap! {
//             String::from("ro").into() => RegisterData::I8(vec![vec![1, 3], vec![2, 4]]),
//         };
//
//         let expected = arr2(&[
//             [
//                 Some(ReadoutValue::Integer(1)),
//                 Some(ReadoutValue::Integer(2)),
//             ],
//             [
//                 Some(ReadoutValue::Integer(3)),
//                 Some(ReadoutValue::Integer(4)),
//             ],
//         ]);
//
//         let readout_data = ReadoutMap::from_register_data_map(&registers);
//         let matrix = readout_data
//             .get_index_wise_matrix("ro")
//             .expect("ReadoutMap should have ro");
//         assert_eq!(matrix, expected);
//     }
//
//     #[test]
//     fn test_get_shot_wise_matrix() {
//         let registers: HashMap<Box<str>, RegisterData> = hashmap! {
//             String::from("ro").into() => RegisterData::I8(vec![vec![1, 3], vec![2, 4]]),
//         };
//
//         let expected = arr2(&[
//             [
//                 Some(ReadoutValue::Integer(1)),
//                 Some(ReadoutValue::Integer(3)),
//             ],
//             [
//                 Some(ReadoutValue::Integer(2)),
//                 Some(ReadoutValue::Integer(4)),
//             ],
//         ]);
//
//         let readout_data = ReadoutMap::from_register_data_map(&registers);
//         let matrix = readout_data
//             .get_shot_wise_matrix("ro")
//             .expect("ReadoutMap should have ro");
//         assert_eq!(matrix, expected);
//     }
// }
