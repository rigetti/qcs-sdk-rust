use enum_as_inner::EnumAsInner;
use num::complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;

use ndarray::prelude::*;

use qcs_api_client_grpc::models::controller::{readout_values, ReadoutValues};

use crate::RegisterData;

/// The result of executing an [`Executable`](crate::Executable)
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionData {
    /// The readout data that was read from the [`Executable`](crate::Executable).
    /// Key is the name of the register, value is the data of the register after execution.
    pub readout_data: ReadoutMap,
    /// The time it took to execute the program on the QPU, not including any network or queueing
    /// time. If paying for on-demand execution, this is the amount you will be billed for.
    ///
    /// This will always be `None` for QVM execution.
    pub duration: Option<Duration>,
}

/// An enum representing every possible readout type
#[derive(Clone, Copy, Debug, EnumAsInner, PartialEq, Serialize, Deserialize)]
pub enum ReadoutValue {
    /// An integer readout value
    Integer(i32),
    /// A real numbered readout value
    Real(f64),
    /// A complex numbered readout value
    Complex(Complex64),
}

/// A matrix where rows are the values of a memory index across all shots, and columns are values
/// for all memory indices of a given shot.
pub type RegisterMatrix = Array2<Option<ReadoutValue>>;

/// Errors that may occur when trying to interpret [`RegisterMatrix`] values as a single data type.
#[derive(Debug, thiserror::Error)]
pub enum RegisterMatrixConversionError {
    /// The value at a given position was an unexpected format.
    #[error("Could not convert register matrix data: {reason} at position ({axis_x}, {axis_y})")]
    InvalidFormat {
        reason: String,
        axis_x: i32,
        axis_y: i32,
    },

    /// The value at a given position was empty.
    #[error("Empty register matrix value at position: ({axis_x}, {axis_y})")]
    Empty { axis_x: i32, axis_y: i32 },
}

/// Given a [`RegisterMatrix`], assume that all data is the `i32` type.
/// If any data is of the wrong type or missing, returns [`RegisterMatrixConversionError`].
pub fn register_matrix_as_i32(
    matrix: &RegisterMatrix,
) -> Result<Array2<i32>, RegisterMatrixConversionError> {
    let mut target = Array2::zeros(matrix.raw_dim());

    for ((axis_x, axis_y), v) in matrix.indexed_iter().into_iter() {
        let i = match v {
            Some(ReadoutValue::Integer(i)) => i.clone(),
            Some(ReadoutValue::Real(_)) => {
                return Err(RegisterMatrixConversionError::InvalidFormat {
                    reason: "value was F64".into(),
                    axis_x: axis_x as i32,
                    axis_y: axis_y as i32,
                })
            }
            Some(ReadoutValue::Complex(_)) => {
                return Err(RegisterMatrixConversionError::InvalidFormat {
                    reason: "value was Complex32".into(),
                    axis_x: axis_x as i32,
                    axis_y: axis_y as i32,
                })
            }
            None => {
                return Err(RegisterMatrixConversionError::Empty {
                    axis_x: axis_x as i32,
                    axis_y: axis_y as i32,
                })
            }
        };

        target[(axis_x, axis_y)] = i;
    }

    Ok(target)
}

/// A mapping of readout fields to their [`ReadoutValue`]s.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ReadoutMap(HashMap<String, RegisterMatrix>);

impl ReadoutMap {
    #[must_use]
    /// Returns a [`ReadoutMap`] with the underlying [`RegisterMatrix`] data
    pub fn new_from_hashmap(map: HashMap<String, RegisterMatrix>) -> Self {
        Self(map)
    }

    /// Returns a [`ReadoutValue`] for the given memory index and shot number, if any
    #[must_use]
    pub fn get_value(
        &self,
        register_name: &str,
        index: usize,
        shot_num: usize,
    ) -> Option<ReadoutValue> {
        self.0
            .get(register_name)
            .and_then(|matrix| matrix.get((index, shot_num)))
            .copied()?
    }

    /// Returns a vector of the [`ReadoutValue`]s in the given register at a particular memory
    /// index across all shots.
    #[must_use]
    pub fn get_values_by_memory_index(
        &self,
        register_name: &str,
        index: usize,
    ) -> Option<Vec<Option<ReadoutValue>>> {
        let register = self.0.get(register_name);
        if let Some(matrix) = register {
            if index >= matrix.nrows() {
                return None;
            }
            Some(matrix.row(index).to_vec())
        } else {
            None
        }
    }

    /// Returns a vector of the [`ReadoutValue`]s in the given register for a particular shot number.
    #[must_use]
    pub fn get_values_by_shot(
        &self,
        register_name: &str,
        shot: usize,
    ) -> Option<Vec<Option<ReadoutValue>>> {
        let register = self.0.get(register_name);
        if let Some(matrix) = register {
            if shot >= matrix.ncols() {
                return None;
            }
            Some(matrix.column(shot).to_vec())
        } else {
            None
        }
    }

    /// Returns a [`RegisterMatrix`] `M` where `M[index]` contains the readout values for memory
    /// offset `index` across all shots.
    #[must_use]
    pub fn get_index_wise_matrix(&self, register_name: &str) -> Option<&RegisterMatrix> {
        self.0.get(register_name)
    }

    /// Returns a [`RegisterMatrix`] `M` where `M[shot]` contains the readout values for all memory
    /// offset indices during shot `shot`.
    #[must_use]
    pub fn get_shot_wise_matrix(&self, register_name: &str) -> Option<RegisterMatrix> {
        let register = self.0.get(register_name);
        register.cloned().map(ArrayBase::reversed_axes)
    }

    /// `readout_values` maps program-defined readout to result-defined readout, e.g.:
    ///     { "ro[0]": "q0", "ro[1]": "q1" }
    /// where `ro[0]` is defined in the original program, and `q0` is what comes back in execution results.
    /// Here we map the result-defined readout values back to their original result-defined names.
    pub(crate) fn from_mappings_and_values(
        readout_mappings: &HashMap<String, String>,
        readout_values: &HashMap<String, ReadoutValues>,
    ) -> Self {
        let mut result = ReadoutMap(HashMap::new());
        for (readout_name, values) in readout_values {
            readout_mappings
                .iter()
                .filter_map(|(program_name, program_alias)| {
                    (program_alias == readout_name).then(|| program_name.as_ref())
                })
                .map(parse_readout_register)
                .filter_map(Result::ok)
                .for_each(|reference| {
                    let row = match &values.values {
                        Some(readout_values::Values::IntegerValues(ints)) => ints
                            .values
                            .clone()
                            .into_iter()
                            .map(|i| Some(ReadoutValue::Integer(i)))
                            .collect(),
                        Some(readout_values::Values::ComplexValues(comps)) => comps
                            .values
                            .clone()
                            .into_iter()
                            .map(|c| Complex64::new(c.real().into(), c.imaginary().into()))
                            .map(|c| Some(ReadoutValue::Complex(c)))
                            .collect(),
                        None => Vec::new(),
                    };
                    let shape = (reference.index + 1, row.len());
                    let matrix = result
                        .0
                        .entry(reference.name)
                        .and_modify(|m| {
                            if shape.0 > m.nrows() {
                                *m = Array2::from_shape_fn(shape, |(r, c)| {
                                    m.get((r, c)).copied().flatten()
                                });
                            }
                        })
                        .or_insert_with(|| Array2::from_elem(shape, None));
                    for (shot_num, value) in row.iter().enumerate() {
                        matrix[[reference.index, shot_num]] = *value;
                    }
                });
        }

        result
    }

    /// Creates a new `ReadoutMap` from a mapping of register names (ie. "ro") to a 2-dimensional
    /// vector containing rows of that registers memory values for each shot.
    #[must_use]
    pub fn from_register_data_map(map: &HashMap<Box<str>, RegisterData>) -> Self {
        let mut result = ReadoutMap(HashMap::new());
        for (name, data) in map {
            let shot_values: Vec<Vec<Option<ReadoutValue>>> = match data {
                RegisterData::I8(i8) => i8
                    .iter()
                    .map(|inner| {
                        inner
                            .iter()
                            .map(|&i| Some(ReadoutValue::Integer(i.into())))
                            .collect()
                    })
                    .collect(),
                RegisterData::I16(i16) => i16
                    .iter()
                    .map(|inner| {
                        inner
                            .iter()
                            .map(|&i| Some(ReadoutValue::Integer(i.into())))
                            .collect()
                    })
                    .collect(),
                RegisterData::F64(f64) => f64
                    .iter()
                    .map(|inner| inner.iter().map(|&f| Some(ReadoutValue::Real(f))).collect())
                    .collect(),
                RegisterData::Complex32(c32) => c32
                    .iter()
                    .map(|inner| {
                        inner
                            .iter()
                            .map(|&c| {
                                Some(ReadoutValue::Complex(Complex64::new(
                                    c.re.into(),
                                    c.im.into(),
                                )))
                            })
                            .collect()
                    })
                    .collect(),
            };
            if !shot_values.is_empty() {
                let nrows = shot_values[0].len();
                let ncols = shot_values.len();
                dbg!((nrows, ncols));
                let matrix = Array2::from_shape_fn((nrows, ncols), |(index, shot_num)| {
                    shot_values[shot_num][index]
                });
                result.0.insert(name.to_string(), matrix);
            }
        }

        result
    }
}

// This is a copy of [`quil_rs::instruction::MemoryReference`] that uses `usize` for the index
// instead of `u64` for compatibility with the containers we use for [`ReadoutMap`].
// It's possible `quil_rs` will use `usize` for it's `MemoryReference` in the future. If so, we
// should use it to replace this.
// See: https://github.com/rigetti/qcs-sdk-rust/issues/224
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

#[cfg(test)]
mod describe_readout_map {
    use maplit::hashmap;
    use ndarray::prelude::*;
    use std::collections::HashMap;
    use test_case::test_case;

    use super::{ReadoutMap, ReadoutValue, ReadoutValues, RegisterData};
    use qcs_api_client_grpc::models::controller::readout_values::Values;
    use qcs_api_client_grpc::models::controller::IntegerReadoutValues;

    fn dummy_readout_values(v: i32) -> ReadoutValues {
        ReadoutValues {
            values: Some(Values::IntegerValues(IntegerReadoutValues {
                values: vec![v],
            })),
        }
    }

    #[test]
    fn it_converts_from_translation_readout_mappings() {
        let readout_mappings = hashmap! {
            String::from("ro[1]") => String::from("qA"),
            String::from("ro[2]") => String::from("qB"),
            String::from("ro[0]") => String::from("qC"),
            String::from("bar[3]") => String::from("qD"),
            String::from("bar[5]") => String::from("qE"),
        };

        let readout_values = hashmap! {
            String::from("qA") => dummy_readout_values(11),
            String::from("qB") => dummy_readout_values(22),
            String::from("qD") => dummy_readout_values(33),
            String::from("qE") => dummy_readout_values(44),
        };

        let readout_map = ReadoutMap::from_mappings_and_values(&readout_mappings, &readout_values);
        let register = readout_map
            .get_index_wise_matrix("ro")
            .expect("ReadoutMap should have ro");

        let expected = arr2(&[
            [None],
            [Some(ReadoutValue::Integer(11))],
            [Some(ReadoutValue::Integer(22))],
        ]);

        assert_eq!(register, expected);

        let bar = readout_map
            .get_index_wise_matrix("bar")
            .expect("ReadoutMap should have field `bar`");

        let expected = arr2(&[
            [None],
            [None],
            [None],
            [Some(ReadoutValue::Integer(33))],
            [None],
            [Some(ReadoutValue::Integer(44))],
        ]);

        assert_eq!(bar, expected);
    }

    #[test]
    fn it_converts_from_register_data_map() {
        let registers: HashMap<Box<str>, RegisterData> = hashmap! {
            String::from("ro").into() => RegisterData::I8(vec![vec![1, 0, 1]]),
        };

        let readout_map = ReadoutMap::from_register_data_map(&registers);

        let ro = readout_map
            .get_index_wise_matrix("ro")
            .expect("ReadoutMap should have ro");

        let expected = arr2(&[
            [Some(ReadoutValue::Integer(1))],
            [Some(ReadoutValue::Integer(0))],
            [Some(ReadoutValue::Integer(1))],
        ]);
        assert_eq!(ro, expected);
    }

    #[test_case(0, 0 => Some(ReadoutValue::Integer(1)) ; "returns value when both indices are in bounds")]
    #[test_case(1, 0 => None ; "returns None when row index is off by 1")]
    #[test_case(0, 1 => None ; "returns None when column index is off by 1")]
    fn get_value(row: usize, col: usize) -> Option<ReadoutValue> {
        let registers: HashMap<Box<str>, RegisterData> = hashmap! {
            String::from("ro").into() => RegisterData::I8(vec![vec![1]]),
        };

        ReadoutMap::from_register_data_map(&registers).get_value("ro", row, col)
    }

    #[test_case(0 => Some(vec![Some(ReadoutValue::Integer(1)), Some(ReadoutValue::Integer(2))]) ; "returns correct row when index is in bounds")]
    #[test_case(2 => None ; "returns None when index is off by 1")]
    fn get_values_by_memory_index(index: usize) -> Option<Vec<Option<ReadoutValue>>> {
        let registers: HashMap<Box<str>, RegisterData> = hashmap! {
            String::from("ro").into() => RegisterData::I8(vec![vec![1, 9], vec![2, 10]]),
        };

        ReadoutMap::from_register_data_map(&registers).get_values_by_memory_index("ro", index)
    }

    #[test_case(0 => Some(vec![Some(ReadoutValue::Integer(1)), Some(ReadoutValue::Integer(2))]) ; "returns correct column when index is in bounds")]
    #[test_case(2 => None ; "returns None when index is off by 1")]
    fn get_values_by_shot(shot_num: usize) -> Option<Vec<Option<ReadoutValue>>> {
        let registers: HashMap<Box<str>, RegisterData> = hashmap! {
            String::from("ro").into() => RegisterData::I8(vec![vec![1, 2], vec![3, 4]]),
        };

        ReadoutMap::from_register_data_map(&registers).get_values_by_shot("ro", shot_num)
    }

    #[test]
    fn test_get_index_wise_matrix() {
        let registers: HashMap<Box<str>, RegisterData> = hashmap! {
            String::from("ro").into() => RegisterData::I8(vec![vec![1, 3], vec![2, 4]]),
        };

        let expected = arr2(&[
            [
                Some(ReadoutValue::Integer(1)),
                Some(ReadoutValue::Integer(2)),
            ],
            [
                Some(ReadoutValue::Integer(3)),
                Some(ReadoutValue::Integer(4)),
            ],
        ]);

        let readout_data = ReadoutMap::from_register_data_map(&registers);
        let matrix = readout_data
            .get_index_wise_matrix("ro")
            .expect("ReadoutMap should have ro");
        assert_eq!(matrix, expected);
    }

    #[test]
    fn test_get_shot_wise_matrix() {
        let registers: HashMap<Box<str>, RegisterData> = hashmap! {
            String::from("ro").into() => RegisterData::I8(vec![vec![1, 3], vec![2, 4]]),
        };

        let expected = arr2(&[
            [
                Some(ReadoutValue::Integer(1)),
                Some(ReadoutValue::Integer(3)),
            ],
            [
                Some(ReadoutValue::Integer(2)),
                Some(ReadoutValue::Integer(4)),
            ],
        ]);

        let readout_data = ReadoutMap::from_register_data_map(&registers);
        let matrix = readout_data
            .get_shot_wise_matrix("ro")
            .expect("ReadoutMap should have ro");
        assert_eq!(matrix, expected);
    }
}
