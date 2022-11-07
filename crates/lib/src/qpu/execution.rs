//! Contains QPU-specific executable stuff.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Duration;

use log::trace;
use quil_rs::program::ProgramError;
use quil_rs::Program;
use tokio::task::{spawn_blocking, JoinError};

use crate::executable::Parameters;
use crate::execution_data::{MemoryReferenceParseError, Qpu, ReadoutMap};
use crate::qpu::{rewrite_arithmetic, runner::JobId, translation::translate};
use crate::JobHandle;

use super::client::{GrpcClientError, Qcs};
use super::quilc::{self, CompilerOpts, TargetDevice};
use super::rewrite_arithmetic::RewrittenProgram;
use super::runner::{retrieve_results, submit};
use super::translation::EncryptedTranslationResult;
use super::{get_isa, IsaError};

/// Contains all the info needed for a single run of an [`crate::Executable`] against a QPU. Can be
/// updated with fresh parameters in order to re-run the same program against the same QPU with the
/// same number of shots.
#[derive(Clone)]
pub(crate) struct Execution<'a> {
    program: RewrittenProgram,
    pub(crate) quantum_processor_id: &'a str,
    pub(crate) shots: u16,
    client: Arc<Qcs>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("problem processing the provided Quil: {0}")]
    Quil(#[from] ProgramError<Program>),
    #[error("An error that is not expected to occur. If this shows up it may be a bug in this SDK or QCS")]
    Unexpected(#[from] Unexpected),
    #[error("Problem communicating with quilc at {uri}: {details}")]
    Quilc { uri: String, details: String },
    #[error("Problem using QCS API: {0}")]
    QcsClient(#[from] GrpcClientError),
    #[error("Problem fetching ISA: {0}")]
    IsaError(#[from] IsaError),
    #[error("Problem parsing memory readout: {0}")]
    ReadoutParse(#[from] MemoryReferenceParseError),
    #[error("Problem when compiling program: {details}")]
    Compilation { details: String },
    #[error("Program when translating the program: {0}")]
    RewriteArithmetic(#[from] rewrite_arithmetic::Error),
    #[error("Program when getting substitutions for program: {0}")]
    Substitution(String),
}

impl From<quilc::Error> for Error {
    fn from(source: quilc::Error) -> Self {
        match source {
            quilc::Error::Isa(source) => Self::Unexpected(Unexpected::Isa(format!("{:?}", source))),
            quilc::Error::QuilcConnection(uri, details) => Self::Quilc {
                uri,
                details: format!("{:?}", details),
            },
            quilc::Error::QuilcCompilation(details) => Self::Compilation { details },
            quilc::Error::Parse(details) => Self::Compilation {
                details: details.to_string(),
            },
        }
    }
}

/// Errors that are not expected to be returned—if they show up, it may be a bug in this library.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Unexpected {
    #[error("Task running {task_name} did not complete.")]
    TaskError {
        task_name: &'static str,
        source: JoinError,
    },
    #[error("Problem converting QCS ISA to quilc ISA")]
    Isa(String),
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
    /// * `client`: A [`qcs::qpu::client::Qcs`] instance provided by the user which contains connection info
    ///     for QCS and the `quilc` compiler.
    /// * `compile_with_quilc`: A boolean that if set, will compile the given `quil` using `quilc`
    /// * `compiler_options`: A [`qcs::qpu::quilc::CompilerOpts`] instance with configuration
    ///     options for quilc. Has no effect if `compile_with_quilc` is false
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
        client: Arc<Qcs>,
        compile_with_quilc: bool,
        compiler_options: CompilerOpts,
    ) -> Result<Execution<'a>, Error> {
        let isa = get_isa(quantum_processor_id, &client).await?;
        let target_device = TargetDevice::try_from(isa)?;

        let program = if compile_with_quilc {
            trace!("Converting to Native Quil");
            let client = client.clone();
            spawn_blocking(move || {
                quilc::compile_program(&quil, target_device, &client, compiler_options)
            })
            .await
            .map_err(|source| {
                Error::Unexpected(Unexpected::TaskError {
                    task_name: "quilc",
                    source,
                })
            })??
        } else {
            trace!("Skipping conversion to Native Quil");
            quil.parse().map_err(Error::Quil)?
        };

        Ok(Self {
            program: RewrittenProgram::try_from(program).map_err(Error::RewriteArithmetic)?,
            quantum_processor_id,
            client,
            shots,
        })
    }

    /// Run on a real QPU and wait for the results.
    pub(crate) async fn submit(&mut self, params: &Parameters) -> Result<JobHandle, Error> {
        let EncryptedTranslationResult { job, readout_map } = translate(
            self.quantum_processor_id,
            &self.program.to_string().0,
            self.shots.into(),
            self.client.as_ref(),
        )
        .await?;

        let patch_values = self
            .get_substitutions(params)
            .map_err(Error::Substitution)?;

        let job_id = submit(
            self.quantum_processor_id,
            job,
            &patch_values,
            self.client.as_ref(),
        )
        .await?;

        Ok(JobHandle::new(
            job_id,
            self.quantum_processor_id,
            readout_map,
        ))
    }

    pub(crate) async fn retrieve_results(
        &self,
        job_id: JobId,
        readout_mappings: HashMap<String, String>,
    ) -> Result<Qpu, Error> {
        let response =
            retrieve_results(job_id, self.quantum_processor_id, self.client.as_ref()).await?;

        Ok(Qpu {
            readout_data: ReadoutMap::from_mappings_and_values(
                &readout_mappings,
                &response.readout_values,
            ),
            duration: response
                .execution_duration_microseconds
                .map(Duration::from_micros),
        })
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
    fn get_substitutions(&self, params: &Parameters) -> Result<Parameters, String> {
        rewrite_arithmetic::get_substitutions(&self.program.substitutions, params)
    }
}
