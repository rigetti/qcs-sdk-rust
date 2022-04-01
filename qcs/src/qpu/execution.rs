//! Contains QPU-specific executable stuff.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Duration;

use eyre::{eyre, Report, Result, WrapErr};
use log::trace;
use quil_rs::expression::Expression;
use tokio::task::spawn_blocking;

use qcs_api::models::EngagementWithCredentials;

use crate::configuration::Configuration;
use crate::executable::Parameters;
use crate::qpu::quilc::{NativeQuil, NativeQuilProgram};
use crate::qpu::rewrite_arithmetic::{RewrittenProgram, SUBSTITUTION_NAME};
use crate::qpu::{engagement, get_isa, organize_ro_sources};
use crate::ExecutionResult;

use super::quilc;
use super::runner::execute;
use super::{build_executable, process_buffers};

/// Contains all the info needed for a single run of an [`crate::Executable`] against a QPU. Can be
/// updated with fresh parameters in order to re-run the same program against the same QPU with the
/// same number of shots.
pub(crate) struct Execution<'a> {
    program: RewrittenProgram,
    pub(crate) quantum_processor_id: &'a str,
    pub(crate) shots: u16,
    // All the stuff needed to actually make requests to QCS, lazily initialized
    qcs: Option<Arc<Qcs>>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("problem communicating with QCS")]
    Qcs {
        source: Report,
        retry_after: Option<Duration>,
    },
    #[error("problem processing the provided Quil")]
    Quil(#[from] Report),
}

struct Qcs {
    /// A mapping of the register name declared in a program to the list of corresponding Buffer names
    buffer_names: HashMap<Box<str>, Vec<String>>,
    engagement: EngagementWithCredentials,
    executable: String,
}

impl<'a> Execution<'a> {
    /// Construct a new [`Execution`] to prepare for running on a real QPU.
    /// This will immediately convert the provided `quil` into a form that QPUs can understand.
    ///
    /// # Arguments
    ///
    /// * `quil`: The raw Quil program to eventually be run on a QPU.
    /// * `shots`: The number of times to run this program with each call to [`Execution::run`].
    /// * `quantum_processor_id`: The QPU this Quil will be run on and should be compiled for.
    /// * `config`: A [`Configuration`] instance provided by the user which contains connection info
    ///     for QCS and the `quilc` compiler.
    ///
    /// returns: Result<Execution, Report>
    ///
    /// # Errors
    ///
    /// All errors will be human readable by way of [`mod@eyre`]. Some potential issues:
    ///
    /// 1. Unable to fetch ISA from QCS for the provided QPU. Either the QCS connection details in
    ///     `config` are wrong or that QPU does not exist.
    /// 1. Unable to compile the program to Native Quil. This probably means the `quil` is invalid.
    /// 1. Unable to parse the Native Quil that was output by `quilc`. This is probably a bug.
    /// 1. Unable to rewrite the Native Quil for a QPU. This may mean that the `quil` was invalid
    ///     for the QPU or that there is a bug in this library.
    pub(crate) async fn new(
        quil: Arc<str>,
        shots: u16,
        quantum_processor_id: &'a str,
        config: Arc<Configuration>,
        compile_with_quilc: bool,
    ) -> Result<Execution<'a>, Error> {
        let isa = get_isa(quantum_processor_id, &config)
            .await
            .map_err(|err| Error::Qcs {
                source: err.wrap_err("When getting ISA"),
                retry_after: None,
            })?;

        let native_quil = if compile_with_quilc {
            trace!("Converting to Native Quil");
            let thread_config = config.clone();
            spawn_blocking(move || {
                quilc::compile_program(&quil, &isa, &thread_config)
                    .wrap_err("When attempting to compile your program to Native Quil")
            })
            .await
            .map_err(|_| eyre!("Error in quilc thread."))??
        } else {
            trace!("Skipping conversion to Native Quil");
            NativeQuil::assume_native_quil(quil.to_string())
        };

        let program =
            NativeQuilProgram::try_from(native_quil).wrap_err("Unable to parse provided Quil")?;

        Ok(Self {
            program: RewrittenProgram::try_from(program)
                .wrap_err("When rewriting program for QPU")?,
            quantum_processor_id,
            shots,
            qcs: None,
        })
    }

    /// Run on a real QPU and wait for the results.
    pub(crate) async fn run(
        &mut self,
        params: &Parameters,
        readouts: &[&str],
        config: &Configuration,
    ) -> Result<HashMap<Box<str>, ExecutionResult>, Error> {
        let qcs = self.refresh_qcs(readouts, config).await?;
        let qcs_for_thread = qcs.clone();

        let patch_values = self
            .get_substitutions(params)
            .map_err(|e| Error::Quil(e.wrap_err("When setting provided parameters")))?;

        let buffers = spawn_blocking(move || {
            execute(
                &qcs_for_thread.executable,
                &qcs_for_thread.engagement,
                &patch_values,
            )
            .map_err(|e| Error::Qcs {
                source: e.wrap_err("While executing"),
                retry_after: None,
            })
        })
        .await
        .map_err(|_| Error::Qcs {
            source: eyre!("Execution thread did not complete."),
            retry_after: None,
        })??;

        let registers = process_buffers(buffers, &qcs.buffer_names)
            .wrap_err("When processing execution results")
            .map_err(Error::from)?;

        ExecutionResult::try_from_registers(registers, self.shots)
            .wrap_err("When decoding execution results")
            .map_err(Error::from)
    }

    /// Take or create a [`Qcs`] for this [`Execution`]. This fetches / updates engagements, builds
    /// the executable, and prepares (from the executable) the mapping of returned values into what
    /// the user expects to see.
    async fn refresh_qcs(
        &mut self,
        readouts: &[&str],
        config: &Configuration,
    ) -> Result<Arc<Qcs>, Error> {
        if let Some(qcs) = &self.qcs {
            return Ok(qcs.clone());
        }

        let response = build_executable(
            self.program.to_string(),
            self.shots,
            self.quantum_processor_id,
            config,
        )
        .await
        .wrap_err("When building executable")
        .map_err(|err| Error::Qcs {
            source: err,
            retry_after: None,
        })?;
        let ro_sources = response.ro_sources.ok_or_else(|| {
            eyre!("No read out sources were defined, did you forget to `MEASURE`?")
        })?;
        let buffer_names =
            organize_ro_sources(ro_sources, readouts).wrap_err("When parsing executable.")?;
        let engagement = engagement::get(String::from(self.quantum_processor_id), config)
            .await
            .map_err(|e| match e {
                engagement::Error::QuantumProcessorUnavailable(duration) => Error::Qcs {
                    source: eyre!(e),
                    retry_after: Some(duration),
                },
                e => Error::Qcs {
                    source: eyre!(e),
                    retry_after: None,
                },
            })?;
        let qcs = Arc::new(Qcs {
            buffer_names,
            engagement,
            executable: response.program,
        });
        self.qcs = Some(qcs.clone());
        Ok(qcs)
    }

    /// Take the user-provided map of [`Parameters`] and produce the map of substitutions which
    /// should be given to QCS with the executable.
    ///
    /// # Example
    ///
    /// If there was a Quil program:
    ///
    /// ```quil
    /// DECLARE theta REAL
    ///
    /// RX(theta) 0
    /// RX(theta + 1) 0
    /// RX(theta + 2) 0
    /// ```
    ///
    /// It would be converted  (in [`Execution::new`]) to something like:
    ///
    /// ```quil
    /// DECLARE __SUBST REAL[2]
    /// DECLARE theta REAL[1]
    ///
    /// RX(theta) 0
    /// RX(__SUBST[0]) 0
    /// RX(__SUBST[1]) 0
    /// ```
    ///
    /// Because QPUs do not evaluate expressions themselves. This function creates the values for
    /// `__SUBST` by calculating the original expressions given the user-provided params (in this
    /// case just `theta`).
    fn get_substitutions(&self, params: &Parameters) -> Result<Parameters> {
        // Convert into the format that quil-rs expects.
        let params: HashMap<&str, Vec<f64>> = params
            .iter()
            .map(|(key, value)| (key.as_ref(), value.clone()))
            .collect();
        let values = self
            .program
            .substitutions
            .iter()
            .map(|substitution: &Expression| {
                substitution
                    .evaluate(&HashMap::new(), &params)
                    .map_err(|_| eyre!("Could not evaluate expression {}", substitution))
                    .and_then(|complex| {
                        if complex.im == 0.0 {
                            Ok(complex.re)
                        } else {
                            Err(eyre!(
                                "Cannot substitute imaginary numbers for QPU execution"
                            ))
                        }
                    })
            })
            .collect::<Result<Vec<f64>>>()?;
        // Convert back to the format that this library expects
        let mut patch_values: Parameters = params
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect();
        patch_values.insert(SUBSTITUTION_NAME.into(), values);
        Ok(patch_values)
    }
}
