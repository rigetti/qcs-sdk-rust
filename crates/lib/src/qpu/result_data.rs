//! This modules provides types and functions for initializing and working with
//! data returned from the QPU
use enum_as_inner::EnumAsInner;
use num::complex::Complex64;
use quil_rs::instruction::MemoryReference;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use qcs_api_client_grpc::models::controller::{
    self, data_value as controller_memory_value, readout_values as controller_readout_values,
    DataValue as ControllerMemoryValues, ReadoutValues as ControllerReadoutValues,
};

/// A row of readout values from the QPU. Each row contains all the values emitted to a
/// memory reference across all shots.
#[derive(Debug, Clone, EnumAsInner, PartialEq, Deserialize, Serialize)]
pub enum ReadoutValues {
    /// Integer readout values
    Integer(Vec<i64>),
    /// Real numbered readout values
    Real(Vec<f64>),
    /// Complex readout values
    Complex(Vec<Complex64>),
}

/// A row of data containing the contents of each memory region at the end of a job.
#[derive(Debug, Clone, EnumAsInner, PartialEq, Deserialize, Serialize)]
pub enum MemoryValues {
    /// Values that correspond to a memory region declared with the BIT or OCTET data type.
    Binary(Vec<u8>),
    /// Values that correspond to a memory region declared with the INTEGER data type.
    Integer(Vec<i64>),
    /// Values that correspond to a memory region declared with the REAL data type.
    Real(Vec<f64>),
}

/// This struct encapsulates data returned from the QPU after executing a job.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct QpuResultData {
    pub(crate) mappings: HashMap<String, String>,
    pub(crate) readout_values: HashMap<String, ReadoutValues>,
    /// The final contents of each memory region, keyed on region name.
    pub(crate) memory_values: HashMap<String, MemoryValues>,
}

impl QpuResultData {
    /// Builds a new [`QpuResultData`] from mappings of memory references to readout identifiers
    /// and readout identifiers to [`ReadoutValues`]
    #[must_use]
    pub fn from_mappings_and_values(
        mappings: HashMap<String, String>,
        readout_values: HashMap<String, ReadoutValues>,
        memory_values: HashMap<String, MemoryValues>,
    ) -> Self {
        Self {
            mappings,
            readout_values,
            memory_values,
        }
    }

    /// Creates a new [`QpuResultData`] using data returned from controller service.
    pub(crate) fn from_controller_mappings_and_values(
        mappings: &HashMap<String, String>,
        readout_values: &HashMap<String, ControllerReadoutValues>,
        memory_values: &HashMap<String, ControllerMemoryValues>,
    ) -> Self {
        Self {
            mappings: mappings.clone(),
            readout_values: readout_values
                .iter()
                .map(|(key, readout_values)| {
                    (
                        key.clone(),
                        match &readout_values.values {
                            Some(controller_readout_values::Values::IntegerValues(v)) => {
                                ReadoutValues::Integer(
                                    v.values.iter().copied().map(i64::from).collect(),
                                )
                            }
                            Some(controller_readout_values::Values::ComplexValues(v)) => {
                                ReadoutValues::Complex(
                                    v.values
                                        .iter()
                                        .map(|c| Complex64::new(c.real.into(), c.imaginary.into()))
                                        .collect(),
                                )
                            }
                            None => ReadoutValues::Integer(Vec::new()),
                        },
                    )
                })
                .collect(),
            memory_values: memory_values
                .iter()
                .filter_map(|(key, memory_values)| {
                    memory_values.value.as_ref().map(|value| {
                        (
                            key.clone(),
                            match value {
                                controller_memory_value::Value::Binary(
                                    controller::BinaryDataValue { data: v },
                                ) => MemoryValues::Binary(v.clone()),
                                controller_memory_value::Value::Integer(
                                    controller::IntegerDataValue { data: v },
                                ) => MemoryValues::Integer(v.clone()),
                                controller_memory_value::Value::Real(
                                    controller::RealDataValue { data: v },
                                ) => MemoryValues::Real(v.clone()),
                            },
                        )
                    })
                })
                .collect(),
        }
    }

    /// Returns the [`ReadoutValues`] for a [`MemoryReference`], or `None` if a mapping to the
    /// provided memory reference doesn't exist.
    #[must_use]
    pub fn get_values_for_memory_reference(
        &self,
        reference: &MemoryReference,
    ) -> Option<&ReadoutValues> {
        self.mappings
            .get(&reference.to_string())
            .and_then(|key| self.readout_values.get(key))
    }

    /// Get mappings of a memory region (ie. "ro\[0\]") to it's key name in `readout_values` (ie. "q0")
    #[must_use]
    pub fn mappings(&self) -> &HashMap<String, String> {
        &self.mappings
    }

    /// Get mapping of a readout values identifier (ie. "q0") to a set of [`ReadoutValues`]
    #[must_use]
    pub fn readout_values(&self) -> &HashMap<String, ReadoutValues> {
        &self.readout_values
    }

    /// Get mapping of a memory region (ie. "ro") to the final contents of that memory region.
    #[must_use]
    pub fn memory_values(&self) -> &HashMap<String, MemoryValues> {
        &self.memory_values
    }
}
