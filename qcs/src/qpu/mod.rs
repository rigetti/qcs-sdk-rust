//! This module contains all the functionality for running Quil programs on a real QPU. Specifically,
//! the [`run_program`] function in this module.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use eyre::{eyre, Result, WrapErr};
use log::{info, trace};

use engagement::get;
use lodgepole::{execute, Buffer};
use qcs_api::apis::quantum_processors_api as qpu_api;
use qcs_api::models::InstructionSetArchitecture;
use translation::translate;

use crate::configuration::Configuration;
pub(crate) use crate::qpu::lodgepole::Register;
use crate::ProgramResult;

mod engagement;
mod lodgepole;
mod quilc;
mod rpcq;
mod translation;

/// Run a Quil program on a real QPU
///
/// # Arguments
/// 1. `quil`: The Quil program as a string,
/// 2. `shots`: The number of times the program should run.
/// 3. `register`: The name of the register containing results that should be read out from QVM.
/// 4. `quantum_processor_id`: The name of the QPU to run on.
///
/// # Warning
///
/// This function is `async` because of the HTTP client under the hood, but it will block your
/// thread waiting on the RPCQ-based functions.
///
/// # Returns
///
/// The generic type `ResultType`. Built-in supported types are `Vec<Vec<f64>>` and `Vec<Vec<u16>>`
///
/// # Errors
/// All errors are human readable by way of [`mod@eyre`]. Some common errors are:
///
/// 1. You are not authenticated for QCS
/// 1. Your credentials don't have an active reservation for the QPU you requested
/// 1. [quilc] was not running.
///
/// [quilc]: https://github.com/quil-lang/quilc
pub async fn run_program(
    quil: &str,
    shots: u16,
    register: &str,
    quantum_processor_id: &str,
) -> Result<ProgramResult> {
    info!("Running program on {}", quantum_processor_id);
    let (isa, config) = get_isa(quantum_processor_id).await?;
    trace!("Fetched ISA successfully");
    let native_quil = quilc::compile_program(quil, &isa, &config)
        .wrap_err("When attempting to compile your program to Native Quil")?;
    trace!("Converted to Native Quil successfully");
    let executable = translate(native_quil, shots, quantum_processor_id, &config)
        .await
        .wrap_err("Could not convert native quil to executable")?;
    trace!("Translation complete.");
    let ro_sources = executable
        .ro_sources
        .ok_or_else(|| eyre!("No read out sources were defined, did you forget to `MEASURE`?"))?;
    let buffer_names = BufferName::from_ro_sources(ro_sources, register)?;

    let engagement = get(Some(quantum_processor_id.to_string()), &config)
        .await
        .wrap_err(
            "Could not get an engagement for the requested QPU. Do you have an active reservation?",
        )?;
    trace!("Engagement retrieved.");

    let buffers = execute(executable.program, engagement)?;
    trace!("Program executed.");
    ProgramResult::try_from_registers(process_buffers(buffers, buffer_names)?, shots)
}

#[cfg(test)]
mod descibe_run_program {
    // TODO: Write a test against test servers which checks e2e
}

/// Process the buffers that come back from a Lodgepole QPU call and map them to the
/// `buffer_names` provided by the translation service, then attempt to fit that data into the (generic)
/// requested structure.
fn process_buffers(
    mut buffers: HashMap<String, Buffer>,
    buffer_names: Vec<BufferName>,
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

async fn get_isa(
    quantum_processor_id: &str,
) -> Result<(InstructionSetArchitecture, Configuration)> {
    let mut config = Configuration::load()
        .await
        .wrap_err("Error loading configuration")?;
    let initial =
        qpu_api::get_instruction_set_architecture(config.as_ref(), quantum_processor_id).await;
    if let Ok(data) = initial {
        Ok((data, config))
    } else {
        config = config
            .refresh()
            .await
            .wrap_err("Error refreshing your QCS credentials.")?;
        let data = qpu_api::get_instruction_set_architecture(config.as_ref(), quantum_processor_id)
            .await
            .wrap_err("Could not load data for the requested quantum processor")?;
        Ok((data, config))
    }
}
