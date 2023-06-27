//! Contains QPU-specific executable stuff.

use std::borrow::Cow;
use std::convert::TryFrom;
use std::num::NonZeroU16;
use std::sync::Arc;
use std::time::Duration;

use qcs_api_client_grpc::services::translation::TranslationOptions;
use quil_rs::program::ProgramError;
use tokio::task::{spawn_blocking, JoinError};

#[cfg(feature = "tracing")]
use tracing::trace;

use crate::executable::Parameters;
use crate::execution_data::{MemoryReferenceParseError, ResultData};
use crate::qpu::{rewrite_arithmetic, translation::translate};
use crate::{ExecutionData, JobHandle};

use super::api::{retrieve_results, submit, ConnectionStrategy, JobTarget};
use super::rewrite_arithmetic::RewrittenProgram;
use super::translation::EncryptedTranslationResult;
use super::QpuResultData;
use super::{get_isa, GetIsaError};
use crate::client::{GrpcClientError, Qcs};
use crate::compiler::quilc::{self, CompilationResult, CompilerOpts, TargetDevice};

/// Contains all the info needed for a single run of an [`crate::Executable`] against a QPU. Can be
/// updated with fresh parameters in order to re-run the same program against the same QPU with the
/// same number of shots.
#[derive(Debug, Clone)]
pub(crate) struct Execution<'a> {
    program: RewrittenProgram,
    pub(crate) quantum_processor_id: Cow<'a, str>,
    pub(crate) shots: NonZeroU16,
    client: Arc<Qcs>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("problem processing the provided Quil: {0}")]
    Quil(#[from] ProgramError),
    #[error("An error that is not expected to occur. If this shows up it may be a bug in this SDK or QCS")]
    Unexpected(#[from] Unexpected),
    #[error("Problem communicating with quilc at {uri}: {details}")]
    Quilc { uri: String, details: String },
    #[error("Problem using QCS API: {0}")]
    QcsClient(#[from] GrpcClientError),
    #[error("Problem fetching ISA: {0}")]
    IsaError(#[from] GetIsaError),
    #[error("Problem parsing memory readout: {0}")]
    ReadoutParse(#[from] MemoryReferenceParseError),
    #[error("Problem when compiling program: {details}")]
    Compilation { details: String },
    #[error("Program when translating the program: {0}")]
    RewriteArithmetic(#[from] rewrite_arithmetic::Error),
    #[error("Program when getting substitutions for program: {0}")]
    Substitution(String),
    #[error("Problem making a request to the QPU: {0}")]
    QpuApiError(#[from] super::api::QpuApiError),
}

impl From<quilc::Error> for Error {
    fn from(source: quilc::Error) -> Self {
        match source {
            quilc::Error::Isa(source) => Self::Unexpected(Unexpected::Isa(format!("{source:?}"))),
            quilc::Error::QuilcConnection(uri, details) => Self::Quilc {
                uri,
                details: format!("{details:?}"),
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
        shots: NonZeroU16,
        quantum_processor_id: Cow<'a, str>,
        client: Arc<Qcs>,
        compile_with_quilc: bool,
        compiler_options: CompilerOpts,
    ) -> Result<Execution<'a>, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            num_shots=%shots,
            %quantum_processor_id,
            %compile_with_quilc,
            ?compiler_options,
            "creating new QPU Execution",
        );

        let isa = get_isa(quantum_processor_id.as_ref(), &client).await?;
        let target_device = TargetDevice::try_from(isa)?;

        let program = if compile_with_quilc {
            #[cfg(feature = "tracing")]
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
            })?
            .map(|CompilationResult { program, .. }| program)?
        } else {
            #[cfg(feature = "tracing")]
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

    /// Translate the execution's quil program for it's given quantum processor.
    pub(crate) async fn translate(
        &mut self,
        options: Option<TranslationOptions>,
    ) -> Result<EncryptedTranslationResult, Error> {
        let encrpyted_translation_result = translate(
            self.quantum_processor_id.as_ref(),
            &self.program.to_string().0,
            self.shots.get().into(),
            self.client.as_ref(),
            options,
        )
        .await?;
        Ok(encrpyted_translation_result)
    }

    /// Run on a real QPU and wait for the results.
    pub(crate) async fn submit(
        &mut self,
        params: &Parameters,
        translation_options: Option<TranslationOptions>,
        connection_strategy: ConnectionStrategy,
    ) -> Result<JobHandle<'a>, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(quantum_processor_id=%self.quantum_processor_id, "submitting job to QPU");

        let job_target = JobTarget::QuantumProcessorId(self.quantum_processor_id.to_string());
        self.submit_to_target(params, job_target, translation_options, connection_strategy)
            .await
    }

    /// Run on specific QCS endpoint and wait for the results.
    pub(crate) async fn submit_to_endpoint_id<S>(
        &mut self,
        params: &Parameters,
        endpoint_id: S,
        translation_options: Option<TranslationOptions>,
    ) -> Result<JobHandle<'a>, Error>
    where
        S: Into<Cow<'a, str>>,
    {
        let job_target = JobTarget::EndpointId(endpoint_id.into().to_string());
        self.submit_to_target(
            params,
            job_target,
            translation_options,
            ConnectionStrategy::DirectAccess,
        )
        .await
    }

    async fn submit_to_target(
        &mut self,
        params: &Parameters,
        job_target: JobTarget,
        translation_options: Option<TranslationOptions>,
        connection_strategy: ConnectionStrategy,
    ) -> Result<JobHandle<'a>, Error> {
        let EncryptedTranslationResult { job, readout_map } =
            self.translate(translation_options).await?;

        let patch_values = self
            .get_substitutions(params)
            .map_err(Error::Substitution)?;

        let job_id = submit(
            &job_target,
            job,
            &patch_values,
            self.client.as_ref(),
            connection_strategy,
        )
        .await?;

        let endpoint_id = match job_target {
            JobTarget::EndpointId(endpoint_id) => Some(endpoint_id),
            JobTarget::QuantumProcessorId(_) => None,
        };

        Ok(JobHandle::new(
            job_id,
            self.quantum_processor_id.to_string(),
            endpoint_id,
            readout_map,
            connection_strategy,
        ))
    }

    pub(crate) async fn retrieve_results(
        &self,
        job_handle: JobHandle<'a>,
    ) -> Result<ExecutionData, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            job_id=%job_handle.job_id(),
            num_shots = %self.shots,
            quantum_processor_id=%self.quantum_processor_id,
            "retrieving execution results for job",
        );

        let response = retrieve_results(
            job_handle.job_id(),
            &job_handle.job_target(),
            self.client.as_ref(),
            job_handle.connection_strategy(),
        )
        .await?;

        Ok(ExecutionData {
            result_data: ResultData::Qpu(QpuResultData::from_controller_mappings_and_values(
                job_handle.readout_map(),
                &response.readout_values,
            )),
            duration: Some(response.execution_duration_microseconds).map(Duration::from_micros),
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
