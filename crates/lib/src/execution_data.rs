use std::collections::HashMap;
use std::convert::TryFrom;
use std::num::{ParseIntError, TryFromIntError};
use std::str::FromStr;
use std::time::Duration;

use qcs_api_client_grpc::models::controller::ReadoutValues;
use quil_rs::instruction::MemoryReference;

use crate::RegisterData;

/// The result of executing an [`Executable`](crate::Executable) via
/// [`Executable::execute_on_qvm`](crate::Executable::execute_on_qvm).
#[derive(Debug, Clone, PartialEq)]
pub struct Qvm {
    /// The readout data that was read from the [`Executable`](crate::Executable).
    /// Key is the name of the register, value is the data of the register after execution.
    pub registers: HashMap<String, RegisterData>,
    /// The time it took to execute the program on the QPU, not including any network or queueing
    /// time. If paying for on-demand execution, this is the amount you will be billed for.
    ///
    /// This will always be `None` for QVM execution.
    pub duration: Option<Duration>,
}

/// A mapping of readout fields to their [`ReadoutValues`].
#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct ReadoutMap(HashMap<MemoryReference, ReadoutValues>);

impl ReadoutMap {
    /// Given a known readout field name and index, return the result's [`ReadoutValues`], if any.
    #[must_use]
    pub fn get_readout_values(&self, field: String, index: u64) -> Option<ReadoutValues> {
        let readout_values = self.0.get(&MemoryReference { name: field, index })?;

        Some(readout_values.clone())
    }

    /// Given a known readout field name, return the result's [`ReadoutValues`] for all indices, if any.
    pub fn get_readout_values_for_field(
        &self,
        field: &str,
    ) -> Result<Option<Vec<Option<ReadoutValues>>>, TryFromIntError> {
        let mut readout_values = Vec::new();
        for (memref, values) in &self.0 {
            let MemoryReference { name, index } = memref;
            let index = usize::try_from(*index)?;
            if name == field {
                if readout_values.len() <= index {
                    readout_values.resize(index + 1, None);
                }
                readout_values[index] = Some(values.clone());
            }
        }

        Ok((!readout_values.is_empty()).then_some(readout_values))
    }

    /// `readout_values` maps program-defined readout to result-defined readout, e.g.:
    ///     { "ro[0]": "q0", "ro[1]": "q1" }
    /// where `ro[0]` is defined in the original program, and `q0` is what comes back in execution results.
    /// Here we map the result-defined readout values back to their original result-defined names.
    pub(crate) fn from_mappings_and_values(
        readout_mappings: &HashMap<String, String>,
        readout_values: &HashMap<String, ReadoutValues>,
    ) -> Self {
        let result = readout_values
            .iter()
            .flat_map(|(readout_name, values)| {
                readout_mappings
                    .iter()
                    .filter(|(_, program_alias)| *program_alias == readout_name)
                    .map(|(program_name, _)| program_name.as_ref())
                    .map(parse_readout_register)
                    .map(|reference| Ok((reference?, values.clone())))
                    .collect::<Result<Vec<_>, MemoryReferenceParseError>>()
            })
            .flatten()
            .collect::<HashMap<_, _>>()
            .into();

        result
    }
}

impl From<HashMap<MemoryReference, ReadoutValues>> for ReadoutMap {
    fn from(map: HashMap<MemoryReference, ReadoutValues>) -> Self {
        Self(map)
    }
}

/// The result of executing an [`Executable`](crate::Executable) via
/// [`Executable::execute_on_qpu`](crate::Executable::execute_on_qpu).
#[derive(Debug, Clone, PartialEq)]
pub struct Qpu {
    /// The data of all readout data that were read from
    /// (via [`Executable::read_from`](crate::Executable::read_from)). Key is the name of the
    /// register, value is the data of the register after execution.
    pub readout_data: ReadoutMap,
    /// The time it took to execute the program on the QPU, not including any network or queueing
    /// time. If paying for on-demand execution, this is the amount you will be billed for.
    ///
    /// This will always be `None` for QVM execution.
    pub duration: Option<Duration>,
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

    use super::{ReadoutMap, ReadoutValues};
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
        let ro = readout_map
            .get_readout_values_for_field("ro")
            .unwrap()
            .expect("ReadoutMap should have field `ro`");

        assert_eq!(
            ro,
            vec![
                None,
                Some(dummy_readout_values(11)),
                Some(dummy_readout_values(22)),
            ],
        );

        let bar = readout_map
            .get_readout_values_for_field("bar")
            .unwrap()
            .expect("ReadoutMap should have field `bar`");

        assert_eq!(
            bar,
            vec![
                None,
                None,
                None,
                Some(dummy_readout_values(33)),
                None,
                Some(dummy_readout_values(44))
            ],
        );
    }
}
