//! This modules provides types and functions for initializing and working with
//! data returned from the QPU
use enum_as_inner::EnumAsInner;
use num::complex::Complex64;
use quil_rs::instruction::MemoryReference;
use std::collections::HashMap;

use qcs_api_client_grpc::models::controller::{
    readout_values as controller_readout_values, ReadoutValues as ControllerReadoutValues,
};

/// A row of readout values from the QPU. Each row contains all the values emitted to a
/// memory reference across all shots.
#[derive(Debug, Clone, EnumAsInner, PartialEq)]
pub enum ReadoutValues {
    /// Integer readout values
    Integer(Vec<i64>),
    /// Real numbered readout values
    Real(Vec<f64>),
    /// Complex readout values
    Complex(Vec<Complex64>),
}

/// This struct encapsulates data returned from the QPU after executing a job.
#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, PartialEq)]
pub struct QpuResultData {
    pub(crate) mappings: HashMap<String, String>,
    pub(crate) readout_values: HashMap<String, ReadoutValues>,
}

impl QpuResultData {
    /// Builds a new [`QpuResultData`] from mappings of memory references to readout identifiers
    /// and readout identifiers to [`ReadoutValues`]
    #[must_use]
    pub fn from_mappings_and_values(
        mappings: HashMap<String, String>,
        readout_values: HashMap<String, ReadoutValues>,
    ) -> Self {
        Self {
            mappings,
            readout_values,
        }
    }

    /// Creates a new [`QpuResultData`] using data returned from controller service.
    pub(crate) fn from_controller_mappings_and_values(
        mappings: &HashMap<String, String>,
        values: &HashMap<String, ControllerReadoutValues>,
    ) -> Self {
        Self {
            mappings: mappings.clone(),
            readout_values: values
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
}
