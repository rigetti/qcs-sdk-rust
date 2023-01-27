//! This modules provides types and functions for initializing and working with
//! QPU readout data.
use enum_as_inner::EnumAsInner;
use num::complex::Complex64;
use quil_rs::instruction::MemoryReference;
use std::collections::HashMap;

use qcs_api_client_grpc::models::controller::{
    readout_values as controller_readout_values, ReadoutValues as ControllerReadoutValues,
};

/// A row of readout values from the QPU.
#[derive(Debug, Clone, EnumAsInner, PartialEq)]
pub enum ReadoutValues {
    /// Integer readout values
    Integer(Vec<i64>),
    /// Real numbered readout values
    Real(Vec<f64>),
    /// Complex readout values
    Complex(Vec<Complex64>),
}

/// This struct encapsulates the readout data returned from the QPU after executing a job.
#[derive(Debug, Clone, PartialEq)]
pub struct QpuReadout {
    /// Mappings of a memory region (ie. "ro[0]") to it's key name in `readout_values` (ie. "q0")
    pub mappings: HashMap<String, String>,
    /// Mapping of a readout values identifier (ie. "q0") to a set of [`ReadoutValues`]
    pub readout_values: HashMap<String, ReadoutValues>,
}

impl QpuReadout {
    /// Creates a new [`QPUReadout`] using data returned from controller service.
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
                                        .map(|c| {
                                            Complex64::new(
                                                c.real.unwrap_or(0.0).into(),
                                                c.imaginary.unwrap_or(0.0).into(),
                                            )
                                        })
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

    /// Returns the [`ReadoutValues`] for a [`MemoryReference`], or None if a mapping to the
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
}
