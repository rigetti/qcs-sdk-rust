//! TODO: Docs
//!
use enum_as_inner::EnumAsInner;
use num::complex::Complex64;
use quil_rs::instruction::MemoryReference;
use std::collections::HashMap;

use qcs_api_client_grpc::models::controller::{
    readout_values as controller_readout_values, ReadoutValues as ControllerReadoutValues,
};

/// TODO: Docs
#[derive(Debug, Clone, EnumAsInner, PartialEq)]
pub enum ReadoutValues {
    /// A vector of all readout values across all shots for integer typed registers.
    Integer(Vec<i32>),
    /// A vector of all readout values across all shots for complex typed registers.
    Complex(Vec<Complex64>),
}

/// TODO: Docs
#[derive(Debug, Clone, PartialEq)]
pub struct QPUReadout {
    /// Mappings of a memory region (ie. "ro[0]") to it's key name in `readout_values` (ie. "q0")
    pub mappings: HashMap<String, String>,
    /// Mapping of a readout values identifier (ie. "q0") to a set of [`ReadoutValues`]
    pub readout_values: HashMap<String, ReadoutValues>,
}

impl QPUReadout {
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
                                ReadoutValues::Integer(v.values.clone())
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

    /// TODO: Docs
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
