use enum_as_inner::EnumAsInner;
use num::complex::Complex64;
use std::collections::HashMap;
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;

use ndarray::prelude::*;

use qcs_api_client_grpc::models::controller::{readout_values, ReadoutValues};
use quil_rs::instruction::MemoryReference;

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
#[derive(Clone, Copy, Debug, EnumAsInner, PartialEq)]
pub enum ReadoutValue {
    /// An integer readout value
    Integer(i32),
    /// A real numbered readout value
    Real(f64),
    /// A complex numbered readout value
    Complex(Complex64),
}

/// A matrix where rows are the values of a memory index across all shots, and columns are values
/// for all memory indexes of a given shot.
pub type RegisterMatrix = Array2<Option<ReadoutValue>>;

/// A mapping of readout fields to their [`ReadoutValues`].
#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct ReadoutMap(HashMap<String, RegisterMatrix>);

impl ReadoutMap {
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

    /// Returns the matrix as a 2-dimensional array where the row is the memory index and the
    /// column is the shot number.
    #[must_use]
    pub fn get_index_wise_matrix(&self, register_name: &str) -> Option<&RegisterMatrix> {
        self.0.get(register_name)
    }

    /// Returns the matrix as a 2-dimensional array where the row is the shot number, and the
    /// column is the memory index.
    #[must_use]
    pub fn get_shot_wise_matrix(&self, register_name: &str) -> Option<RegisterMatrix> {
        let register = self.0.get(register_name);
        register.map(|matrix| matrix.clone().reversed_axes())
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
                    // TODO handle possible truncation
                    let shape = (reference.index as usize + 1, row.len());
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
                        // TODO handle possible truncation
                        matrix[[reference.index as usize, shot_num]] = *value;
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
        index: u64::from_str(&register_name[open_brace + 1..close_brace])?,
    })
}

#[cfg(test)]
mod describe_readout_map {
    use maplit::hashmap;
    use ndarray::prelude::*;
    use std::collections::HashMap;

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
}
