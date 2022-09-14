//! This module contains all the functionality for running Quil programs on a real QPU. Specifically,
//! the [`Execution`] struct in this module.

use core::mem;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::str::FromStr;

use log::trace;
use reqwest::StatusCode;

pub(crate) use execution::{Error as ExecutionError, Execution};
use qcs_api::apis::quantum_processors_api::{
    get_instruction_set_architecture, GetInstructionSetArchitectureError,
};
use qcs_api::apis::Error as QcsError;
use qcs_api::models::{InstructionSetArchitecture, TranslateNativeQuilToEncryptedBinaryResponse};
use runner::Buffer;
use translation::{translate, Error as TranslationError};

use crate::configuration::Configuration;
pub(crate) use crate::qpu::rewrite_arithmetic::RewrittenQuil;
pub(crate) use crate::qpu::runner::DecodeError;
pub(crate) use crate::qpu::runner::JobId;
pub(crate) use crate::qpu::runner::Register;

pub(crate) mod engagement;
mod execution;
pub mod quilc;
pub(crate) mod rewrite_arithmetic;
pub(crate) mod rpcq;
pub(crate) mod runner;
pub(crate) mod translation;

async fn build_executable(
    quil: RewrittenQuil,
    shots: u16,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<TranslateNativeQuilToEncryptedBinaryResponse, TranslationError> {
    let executable = translate(quil, shots, quantum_processor_id, config).await?;
    trace!("Translation complete.");
    Ok(executable)
}

/// Process the buffers that come back from a QPU call and map them to the
/// `buffer_names` provided by the translation service, then attempt to fit that data into the (generic)
/// requested structure.
pub(crate) fn process_buffers(
    mut buffers: HashMap<String, Buffer>,
    buffer_names: &HashMap<Box<str>, Vec<String>>,
) -> Result<HashMap<Box<str>, Vec<Register>>, DecodeError> {
    buffer_names
        .iter()
        .map(|(register_name, buffer_names)| {
            let registers: Result<Vec<Register>, DecodeError> = buffer_names
                .iter()
                .map(|buffer_name| {
                    buffers
                        .remove(buffer_name)
                        .ok_or_else(|| DecodeError::MissingBuffer(buffer_name.clone()))
                        .and_then(Register::try_from)
                })
                .collect();
            registers.map(|registers| (register_name.clone(), registers))
        })
        .collect()
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
struct BufferName {
    register_name: String,
    buffer_name: String,
    index: usize,
}

/// The QPU is going to return the data of an execution mapped to its own named buffers.
/// Translation gives us the info we need to translate those buffers back to the declared memory
/// space in the Quil program. This function reorganizes the buffer names into a form more useful
/// for later processing and validates that we have all of the buffers we're expecting.
///
/// # Errors
///
/// 1. No buffers found for a provided `ro_source`.
/// 2. There was a gap in the readout memory which must be contiguous.
///
/// # Arguments
///
/// 1. `ro_sources` is the 2D vec of strings that comes back from translation which we need to decode.
/// 2. `readouts` is the slice of register names that the user wants to read from
///
/// # Returns
///
/// A map of the name of a declared register to the vector of buffers that represent it in a
/// QPU response.
///
/// # Example
///
/// Declared memory which looks like this:
/// ```quil
/// DECLARE first BIT[1]
/// DECLARE second BIT[2]
/// ```
///
/// Will return a map that looks something like this (in JSON for readability):
/// ```json
/// {
///     "first": ["q1"],
///     "second": ["q3", "q5"]
/// }
/// ```
///
/// Where the `q<n>` is the name of a buffer that the QPU will return.
fn organize_ro_sources(
    ro_sources: Vec<Vec<String>>,
    readouts: &[&str],
) -> Result<HashMap<Box<str>, Vec<String>>, DecodeError> {
    let readout_set: HashSet<&str> = readouts.iter().copied().collect();

    // First, collect the unordered list of buffers since we have no guarantee of what order
    // translation returned them in.
    let mut buffer_names: Vec<BufferName> = ro_sources
        .into_iter()
        .filter_map(|mut source| {
            // There will be buffer names returned that we don't care about, those are filtered out
            // by returning None from this closure.

            // A source is a tuple (declared_register, name_of_lodgepole_buffer) but it's
            // deserialized as a Vec with two elements.
            let buffer_name = source.pop()?;
            let mut register_name = source.pop()?;

            // A register_name could be a simple name like "ro" which means the same thing as "ro[0]"
            if readout_set.contains(&register_name.as_str()) {
                return Some(BufferName {
                    register_name,
                    buffer_name,
                    index: 0,
                });
            }

            // parse a string like ro[0] to check the reg name (ro) and index (0)
            let open_brace = register_name.find('[')?;
            let close_brace = register_name.find(']')?;
            let index = usize::from_str(&register_name[open_brace + 1..close_brace]).ok()?;
            register_name.truncate(open_brace);
            if !readout_set.contains(&register_name.as_str()) {
                return None;
            }

            Some(BufferName {
                register_name,
                buffer_name,
                index,
            })
        })
        .collect();

    if buffer_names.is_empty() {
        return Err(DecodeError::MissingBuffer(String::from("ro")));
    }

    // Sort so that we have one register at a time in ascending index order, making the organization
    // below simpler.
    buffer_names.sort_by(|first, second| {
        first
            .register_name
            .cmp(&second.register_name)
            .then_with(|| first.index.cmp(&second.index))
    });

    // Reorganize and validate all the BufferNames
    let first = buffer_names.remove(0);
    if first.index != 0 {
        return Err(DecodeError::ContiguousMemory {
            register: first.register_name,
            index: 0,
        });
    }
    let mut current_index = 1;
    let mut current_register_name = first.register_name;
    let mut current_names = vec![first.buffer_name];
    let mut results = HashMap::new();

    for buffer_name in buffer_names {
        let BufferName {
            register_name,
            buffer_name,
            index,
        } = buffer_name;
        if current_register_name != register_name {
            // Switching to the next register, store the last register's results
            let names = mem::take(&mut current_names);
            let previous_register_name = mem::replace(&mut current_register_name, register_name);
            results.insert(previous_register_name.into_boxed_str(), names);
            current_index = 0;
        }

        if current_index != index {
            return Err(DecodeError::ContiguousMemory {
                register: current_register_name,
                index: current_index,
            });
        }
        current_index += 1;
        current_names.push(buffer_name);
    }

    results.insert(current_register_name.into_boxed_str(), current_names);

    for expected_readout in readout_set {
        if !results.contains_key(expected_readout) {
            return Err(DecodeError::MissingBuffer(String::from(expected_readout)));
        }
    }

    Ok(results)
}

#[cfg(test)]
mod describe_organize_ro_sources {
    use maplit::hashmap;

    use crate::qpu::organize_ro_sources;

    #[test]
    fn it_converts_from_translation_ro_sources() {
        let ro_sources = vec![
            vec![String::from("ro[1]"), String::from("q7")],
            vec![String::from("blah"), String::from("blah")],
            vec![String::from("something"), String::from("something")],
            vec![String::from("ro[0]"), String::from("q6")],
        ];
        let readouts = &["ro", "something"];
        let expected = hashmap! {
            Box::from(String::from("ro")) => vec![String::from("q6"), String::from("q7")],
            Box::from(String::from("something")) => vec![String::from("something")]
        };

        let buffer_names =
            organize_ro_sources(ro_sources, readouts).expect("Failed to convert ro_sources");
        assert_eq!(buffer_names, expected);
    }

    #[test]
    fn it_errors_on_buffers_not_starting_with_0() {
        let ro_sources = vec![vec![String::from("ro[1]"), String::from("q7")]];

        let result = organize_ro_sources(ro_sources, &["ro"]);
        assert!(result.is_err());
    }

    #[test]
    fn it_errors_when_no_matching_buffers() {
        let ro_sources = vec![vec![String::from("blah[0]"), String::from("blah")]];

        let result = organize_ro_sources(ro_sources, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn it_errors_when_missing_a_buffer() {
        let ro_sources = vec![vec![String::from("blah[0]"), String::from("blah")]];

        let result = organize_ro_sources(ro_sources, &["blah", "other"]);
        assert!(result.is_err());
    }

    #[test]
    fn it_errors_when_gaps_in_buffers() {
        let ro_sources = vec![
            vec![String::from("ro[0]"), String::from("q6")],
            vec![String::from("a[0]"), String::from("a0")],
            vec![String::from("a[1]"), String::from("a1")],
            vec![String::from("ro[2]"), String::from("q7")],
        ];

        let result = organize_ro_sources(ro_sources, &["ro", "a"]);
        assert!(result.is_err());
    }
}

/// Query QCS for the ISA of the provided `quantum_processor_id`.
///
/// # Errors
///
/// 1. Problem communicating with QCS
/// 2. Unauthenticated
/// 3. Expired token
pub(crate) async fn get_isa(
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<InstructionSetArchitecture, IsaError> {
    get_instruction_set_architecture(config.as_ref(), quantum_processor_id)
        .await
        .map_err(IsaError::from)
}

/// The errors that can occur when fetching the ISA of a QPU.
#[derive(Debug, thiserror::Error)]
pub(crate) enum IsaError {
    #[error("QPU not found")]
    QpuNotFound,
    #[error("Unauthorized, refresh credentials and try again.")]
    Unauthorized,
    #[error("Problem understanding QCS, this is likely a bug")]
    QcsError(#[source] QcsError<GetInstructionSetArchitectureError>),
    #[error("Problem communicating with QCS")]
    QcsCommunicationError(#[from] std::io::Error),
}

impl From<QcsError<GetInstructionSetArchitectureError>> for IsaError {
    fn from(error: QcsError<GetInstructionSetArchitectureError>) -> Self {
        match error {
            QcsError::ResponseError(content) if content.status == StatusCode::NOT_FOUND => {
                IsaError::QpuNotFound
            }
            QcsError::ResponseError(content)
                if content.status == StatusCode::UNAUTHORIZED
                    || content.status == StatusCode::FORBIDDEN =>
            {
                IsaError::Unauthorized
            }
            QcsError::Io(io_error) => io_error.into(),
            error => IsaError::QcsError(error),
        }
    }
}
