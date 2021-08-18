//! This module contains all the functionality for running Quil programs on a real QPU. Specifically,
//! the [`Execution`] struct in this module.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use eyre::{eyre, Result, WrapErr};
use log::trace;

pub(crate) use execution::{Error, Execution};
use lodgepole::Buffer;
use qcs_api::apis::quantum_processors_api as qpu_api;
use qcs_api::models::{InstructionSetArchitecture, TranslateNativeQuilToEncryptedBinaryResponse};
use translation::translate;

use crate::configuration::Configuration;
pub(crate) use crate::qpu::lodgepole::Register;
use crate::qpu::rewrite_arithmetic::RewrittenQuil;

mod engagement;
mod execution;
mod lodgepole;
mod quilc;
mod rewrite_arithmetic;
mod rpcq;
mod translation;

async fn build_executable(
    quil: RewrittenQuil,
    shots: u16,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<TranslateNativeQuilToEncryptedBinaryResponse> {
    let executable = translate(quil, shots, quantum_processor_id, config)
        .await
        .wrap_err("Could not convert native quil to executable")?;
    trace!("Translation complete.");
    Ok(executable)
}

/// Process the buffers that come back from a Lodgepole QPU call and map them to the
/// `buffer_names` provided by the translation service, then attempt to fit that data into the (generic)
/// requested structure.
fn process_buffers(
    mut buffers: HashMap<String, Buffer>,
    buffer_names: &[BufferName],
) -> Result<Vec<Register>> {
    let mut results = Vec::with_capacity(buffer_names.len());
    for buffer_name in buffer_names {
        let buffer = buffers.remove(&buffer_name.buffer_name).ok_or_else(|| {
            eyre!(
                "Response from QPU did not include expected buffer named {}",
                buffer_name.buffer_name
            )
        })?;
        results.push(
            Register::try_from(buffer).wrap_err("Could not convert buffer into requested type")?,
        );
    }
    Ok(results)
}

#[derive(Debug, PartialEq)]
struct BufferName {
    buffer_name: String,
    index: usize,
}

impl BufferName {
    /// Turn the translation service's `ro_sources` output into a Vec of [`BufferName`]
    ///
    /// # Errors
    ///
    /// 1. No buffers were found for the requested register.
    fn from_ro_sources(ro_sources: Vec<Vec<String>>, register: &str) -> Result<Vec<BufferName>> {
        let mut buffer_names: Vec<BufferName> = ro_sources
            .into_iter()
            .filter_map(|mut sources| {
                let buffer_name = sources.pop()?;
                let register_name = sources.pop()?;
                if register_name == register {
                    return Some(BufferName {
                        buffer_name,
                        index: 0,
                    });
                }
                // parse a string like ro[0] to check the reg name (ro) and index (0)
                let open_brace = register_name.find('[')?;
                let close_brace = register_name.find(']')?;
                if &register_name[..open_brace] != register {
                    return None;
                }

                let index = usize::from_str(&register_name[open_brace + 1..close_brace]).ok()?;
                Some(BufferName { buffer_name, index })
            })
            .collect();

        if buffer_names.is_empty() {
            return Err(eyre!(
                "No buffers were found for register {}, at least one is required. Did you remember to measure?",
                register
            ));
        }

        buffer_names.sort_by(|first, second| {
            first
                .index
                .partial_cmp(&second.index)
                .expect("Comparing two usize is always legal")
        });

        if buffer_names[0].index != 0 {
            return Err(eyre!("The first buffer must be at index 0."));
        }

        for i in 1..buffer_names.len() {
            let second_index = buffer_names[i].index;
            let first_index = buffer_names[i - 1].index;
            if first_index + 1 != second_index {
                return Err(eyre!(
                    "This method requires contiguous memory, but a gap was detected between {register}[{first_index}] and {register}[{second_index}]",
                    register = register,
                    first_index = first_index,
                    second_index = second_index,
                ));
            }
        }
        Ok(buffer_names)
    }
}

#[cfg(test)]
mod describe_buffer_name {
    use super::*;

    #[test]
    fn it_converts_from_translation_ro_sources() {
        let ro_sources = vec![
            vec!["ro[1]".to_string(), "q7".to_string()],
            vec!["blah".to_string(), "blah".to_string()],
            vec!["ro[0]".to_string(), "q6".to_string()],
        ];
        let register = "ro";
        let expected = vec![
            BufferName {
                buffer_name: "q6".to_string(),
                index: 0,
            },
            BufferName {
                buffer_name: "q7".to_string(),
                index: 1,
            },
        ];

        let buffer_names = BufferName::from_ro_sources(ro_sources, register)
            .expect("Failed to convert ro_sources");
        assert_eq!(buffer_names, expected);
    }

    #[test]
    fn it_errors_on_buffers_not_starting_with_0() {
        let ro_sources = vec![vec!["ro[1]".to_string(), "q7".to_string()]];
        let register = "ro";

        let result = BufferName::from_ro_sources(ro_sources, register);
        assert!(result.is_err());
    }

    #[test]
    fn it_errors_when_no_matching_buffers() {
        let ro_sources = vec![vec!["blah[0]".to_string(), "blah".to_string()]];
        let register = "ro";

        let result = BufferName::from_ro_sources(ro_sources, register);
        assert!(result.is_err());
    }

    #[test]
    fn it_errors_when_gaps_in_buffers() {
        let ro_sources = vec![
            vec!["ro[0]".to_string(), "q6".to_string()],
            vec!["ro[2]".to_string(), "q7".to_string()],
        ];
        let register = "ro";

        let result = BufferName::from_ro_sources(ro_sources, register);
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
async fn get_isa(
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<InstructionSetArchitecture> {
    qpu_api::get_instruction_set_architecture(config.as_ref(), quantum_processor_id)
        .await
        .wrap_err("Could not load data for the requested quantum processor")
}
