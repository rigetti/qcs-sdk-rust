use enum_as_inner::EnumAsInner;
use num::complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;

use ndarray::prelude::*;

use crate::qpu::readout_data::ReadoutValues;
use crate::{qpu::readout_data::QpuReadout, qvm::QvmMemory, RegisterData};

/// Represents the two possible types of readout data returned from either the QVM or a real QPU.
/// Each variant contains the original data returned from it's respective executor.
///
/// # Usage
///
/// Your usage of [`ReadoutData`] will depend on the types of programs you are running and where.
/// The `to_readout_map()` method will attempt to build a [`ReadoutMap`] out of the data, where each
/// register name is mapped to a 2-dimensional rectangular [`RegisterMatrix`] where each row
/// represents the final values in each register index for a particular shot. This is often the
/// desired form of the data and it is _probably_ what you want. This transformation isn't always
/// possible, in which case `to_readout_map()` will return an error.
///
/// To understand why this transformation can fail, we need to understand a bit about how readout data is
/// returned from the QVM and from a real QPU:
///
/// The QVM treats each `DECLARE` statement as initialzing some amount of memory. This memory works
/// as one might expect it to. It is zero-initalized, and subsequent writes to the same region
/// overwrite the previous value. The QVM returns memory at the end of every shot. This means
/// we get the last value in every memory reference for each shot, which is exactly the
/// representation we want for a [`RegisterMatrix`]. For this reason, `to_readout_map()` should
/// always succeed for [`ReadoutData::QVM`]
///
/// The QPU on the other hand doesn't use the same memory model as the QVM. Each memory reference
/// (ie. "ro[0]") is more like a stream than a value in memory. Every `MEASURE` to a memory
/// reference emits a new value to said stream. This means that the number of values per memory
/// reference can vary per shot. For this reason, it's not always clear what the final value in
/// each shot was for a particular reference. When this is the case, `to_readout_map()` will return
/// an error as it's impossible to build a correct [`RegisterMatrix`]  from the data without
/// knowing the intent of the program that was run. Instead, it's recommended to build the
/// [`RegisterMatrix`] you need from the inner [`QpuReadout`] data using the knowledge of your
/// program to choose the correct readout values for each shot.
#[derive(Debug, Clone, PartialEq, EnumAsInner)]
pub enum ReadoutData {
    /// Data returned from the QVM, stored as [`QVMMemory`]
    Qvm(QvmMemory),
    /// Readout data returned from the QPU, stored as [`QPUReadout`]
    Qpu(QpuReadout),
}

/// The result of executing an [`Executable`](crate::Executable)
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionData {
    /// The [`ReadoutData`] that was read from the [`Executable`](crate::Executable).
    pub readout_data: ReadoutData,
    /// The time it took to execute the program on the QPU, not including any network or queueing
    /// time. If paying for on-demand execution, this is the amount you will be billed for.
    ///
    /// This will always be `None` for QVM execution.
    pub duration: Option<Duration>,
}

/// An enum representing every possible register type as a 2 dimensional matrix.
#[derive(Clone, Debug, EnumAsInner, PartialEq, Serialize, Deserialize)]
pub enum RegisterMatrix {
    /// Integer register
    Integer(Array2<i64>),
    /// Real numbered register
    Real(Array2<f64>),
    /// Complex numbered register
    Complex(Array2<Complex64>),
}

/// A mapping of a register name (ie. "ro") to a [`RegisterMatrix`] containing the values for the
/// register.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(transparent)]
pub struct ReadoutMap(HashMap<String, RegisterMatrix>);

/// Errors that may occur when trying to build a [`RegisterMatrix`] from execution data
#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum RegisterMatrixConversionError {
    /// The data could not be fit into a rectangular matrix
    #[error("The data for register {register} does fit into a rectangular matrix")]
    InvalidShape { register: String },

    /// The memory reference had no associated readout values
    #[error("The mapping of {memory_reference} to {alias} had no readout values")]
    UnmappedAlias {
        memory_reference: String,
        alias: String,
    },

    /// A row of readout values for a register were missing
    #[error("Missing readout values for {register}[{index}]")]
    MissingRow { register: String, index: usize },

    /// The memory reference could not be parsed
    #[error("{0}")]
    MemoryReferenceParseError(MemoryReferenceParseError),
}

impl ReadoutData {
    /// Convert [`ReadoutData`] from its inner representation as [`QVMMemory`] or
    /// [`QPUReadout`] into a [`ReadoutMap`]. The [`RegisterMatrix`] for each register will be
    /// constructed such that each row contains all the values in the register for a single shot.
    ///
    /// # Errors
    ///
    /// Returns a [`RegisterMatrixConversionError`] if the inner execution data for any of the
    /// registers would result in a jagged matrix. [`QPUReadout`] data is captured per measure,
    /// meaning a value is returned for every measure to a memory reference, not just once per shot.
    /// This is often the case in programs that use mid-circuit measurement or dynamic control flow,
    /// where measurements to the same memory reference might occur multiple times in a shot, or be
    /// skipped conditionally. In these cases, building a rectangular [`RegisterMatrix`] would
    /// necessitate making assumptions about the data that could skew the data in undesirable ways.
    /// Instead, it's recommended to manually build a matrix from [`QPUReadout`] that accurately
    /// selects the last value per-shot based on the program that was run.
    pub fn to_readout_map(&self) -> Result<ReadoutMap, RegisterMatrixConversionError> {
        match self {
            ReadoutData::Qvm(data) => ReadoutMap::from_qvm_memory(data),
            ReadoutData::Qpu(data) => ReadoutMap::from_qpu_readout_data(data),
        }
    }
}

impl ReadoutMap {
    /// Returns the [`RegisterMatrix`] for the given register, if it exists.
    #[must_use]
    pub fn get_register_matrix(&self, register_name: &str) -> Option<&RegisterMatrix> {
        self.0.get(register_name)
    }

    /// Returns a [`ReadoutMap`] with the underlying [`RegisterMatrix`] data
    #[must_use]
    pub fn from_hashmap(map: HashMap<String, RegisterMatrix>) -> Self {
        Self(map)
    }

    /// Returns a [`ReadoutMap`] built from [`QVMMemory`]
    pub fn from_qvm_memory(memory: &QvmMemory) -> Result<Self, RegisterMatrixConversionError> {
        Ok(Self(
            memory
                .iter()
                .map(|(name, register)| {
                    let register_matrix = match register {
                        RegisterData::I8(data) => RegisterMatrix::Integer(
                            Array::from_shape_vec(
                                (data.len(), data.first().map_or(0, Vec::len)),
                                data.iter().flatten().copied().map(i64::from).collect(),
                            )
                            .map_err(|_| {
                                RegisterMatrixConversionError::InvalidShape {
                                    register: name.to_string(),
                                }
                            })?,
                        ),
                        RegisterData::I16(data) => RegisterMatrix::Integer(
                            Array::from_shape_vec(
                                (data.len(), data.first().map_or(0, Vec::len)),
                                data.iter().flatten().copied().map(i64::from).collect(),
                            )
                            .map_err(|_| {
                                RegisterMatrixConversionError::InvalidShape {
                                    register: name.to_string(),
                                }
                            })?,
                        ),
                        RegisterData::F64(data) => RegisterMatrix::Real(
                            Array::from_shape_vec(
                                (data.len(), data.first().map_or(0, Vec::len)),
                                data.iter().flatten().copied().collect(),
                            )
                            .map_err(|_| {
                                RegisterMatrixConversionError::InvalidShape {
                                    register: name.to_string(),
                                }
                            })?,
                        ),
                        RegisterData::Complex32(data) => RegisterMatrix::Complex(
                            Array::from_shape_vec(
                                (data.len(), data.first().map_or(0, Vec::len)),
                                data.iter()
                                    .flatten()
                                    .copied()
                                    .map(|c| Complex64::new(c.re.into(), c.im.into()))
                                    .collect(),
                            )
                            .map_err(|_| {
                                RegisterMatrixConversionError::InvalidShape {
                                    register: name.to_string(),
                                }
                            })?,
                        ),
                    };
                    Ok((name.clone(), register_matrix))
                })
                .collect::<Result<HashMap<String, RegisterMatrix>, RegisterMatrixConversionError>>(
                )?,
        ))
    }

    /// Attempts to build a [`ReadoutMap`] from [`QpuReadout`].
    ///
    /// # Errors
    ///
    /// This fails if the underlying [`QpuReadout`] data is jagged. See [`ReadoutData`] for more
    /// detailed explanations of why and when this occurs.
    pub fn from_qpu_readout_data(
        readout_data: &QpuReadout,
    ) -> Result<Self, RegisterMatrixConversionError> {
        Ok(
            Self(
                readout_data
                    .mappings
                    .iter()
                    // Pair all the memory references with their readout values
                    .map(|(memory_reference, alias)| {
                        Ok((
                            parse_readout_register(memory_reference).map_err(|e| {
                                RegisterMatrixConversionError::MemoryReferenceParseError(e)
                            })?,
                            readout_data.readout_values.get(alias).ok_or(
                                RegisterMatrixConversionError::UnmappedAlias {
                                    memory_reference: memory_reference.to_string(),
                                    alias: alias.to_string(),
                                },
                            )?,
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
                                    return Err(RegisterMatrixConversionError::MissingRow {
                                        register: reference.name.clone(),
                                        index: previous.index + 1,
                                    });
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
                                    ReadoutValues::Real(v) => RegisterMatrix::Real(Array2::zeros(
                                        (v.len(), reference.index + 1),
                                    )),
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
                                _ => {
                                    return Err(RegisterMatrixConversionError::InvalidShape {
                                        register: reference.name.clone(),
                                    })
                                }
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
pub enum MemoryReferenceParseError {
    #[error("Could not parse memory reference: {reason}")]
    InvalidFormat { reason: String },

    #[error("Could not parse index from reference name: {0}")]
    InvalidIndex(#[from] ParseIntError),
}

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

    use crate::qpu::readout_data::QpuReadout;
    use crate::qvm::QvmMemory;

    use super::{ReadoutMap, RegisterData};
    use qcs_api_client_grpc::models::controller::readout_values::Values;
    use qcs_api_client_grpc::models::controller::{IntegerReadoutValues, ReadoutValues};

    fn dummy_readout_values(v: Vec<i32>) -> ReadoutValues {
        ReadoutValues {
            values: Some(Values::IntegerValues(IntegerReadoutValues { values: v })),
        }
    }

    #[test]
    fn it_converts_rectangular_qpu_readout_to_readout_map() {
        let readout_mappings = hashmap! {
            String::from("ro[1]") => String::from("qB"),
            String::from("ro[2]") => String::from("qC"),
            String::from("ro[0]") => String::from("qA"),
        };

        let readout_values = hashmap! {
            String::from("qA") => dummy_readout_values(vec![1, 2]),
            String::from("qB") => dummy_readout_values(vec![3, 4]),
            String::from("qC") => dummy_readout_values(vec![5, 6]),
        };

        let qpu_readout =
            QpuReadout::from_controller_mappings_and_values(&readout_mappings, &readout_values);

        let readout_map = ReadoutMap::from_qpu_readout_data(&qpu_readout)
            .expect("Should be able to create ReadoutMap from rectangular QPU readout");

        let register = readout_map
            .get_register_matrix("ro")
            .expect("ReadoutMap should have ro")
            .as_integer()
            .expect("Should be a register of integer values");

        let expected = arr2(&[[1, 3, 5], [2, 4, 6]]);

        assert_eq!(register, expected);
    }

    #[test]
    fn it_fails_to_convert_missing_readout_indices_to_readout_map() {
        let readout_mappings = hashmap! {
            String::from("ro[1]") => String::from("qA"),
            String::from("ro[2]") => String::from("qB"),
            String::from("ro[0]") => String::from("qC"),
            String::from("bar[3]") => String::from("qD"),
            String::from("bar[5]") => String::from("qE"),
        };

        let readout_values = hashmap! {
            String::from("qA") => dummy_readout_values(vec![11]),
            String::from("qB") => dummy_readout_values(vec![22]),
            String::from("qD") => dummy_readout_values(vec![33]),
            String::from("qE") => dummy_readout_values(vec![44]),
        };

        let qpu_readout =
            QpuReadout::from_controller_mappings_and_values(&readout_mappings, &readout_values);

        ReadoutMap::from_qpu_readout_data(&qpu_readout)
            .expect_err("Should not be able to create ReadoutMap from QPU readout with missing indices for a register");
    }

    #[test]
    fn it_fails_to_convert_jagged_qpu_readout_to_readout_map() {
        let readout_mappings = hashmap! {
            String::from("ro[1]") => String::from("qB"),
            String::from("ro[2]") => String::from("qC"),
            String::from("ro[0]") => String::from("qA"),
        };

        let readout_values = hashmap! {
            String::from("qA") => dummy_readout_values(vec![1, 2]),
            String::from("qB") => dummy_readout_values(vec![2]),
            String::from("qC") => dummy_readout_values(vec![3]),
        };

        let qpu_readout =
            QpuReadout::from_controller_mappings_and_values(&readout_mappings, &readout_values);

        ReadoutMap::from_qpu_readout_data(&qpu_readout)
            .expect_err("Should not be able to create ReadoutMap from QPU readout with jagged data for a register");
    }

    #[test]
    fn it_converts_from_qvm_memory() {
        let qvm_memory: QvmMemory = hashmap! {
            String::from("ro") => RegisterData::I8(vec![vec![1, 0, 1]]),
        };

        let readout_map = ReadoutMap::from_qvm_memory(&qvm_memory)
            .expect("Should be able to create ReadoutMap from QVMMemory");

        let ro = readout_map
            .get_register_matrix("ro")
            .expect("ReadoutMap should have ro")
            .as_integer()
            .expect("Should be a register of integers");

        let expected = arr2(&[[1, 0, 1]]);
        assert_eq!(ro, expected);
    }
}
