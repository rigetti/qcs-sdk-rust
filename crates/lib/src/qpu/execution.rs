//! Contains QPU-specific executable stuff.

use std::borrow::Cow;
use std::convert::TryFrom;
use std::num::NonZeroU16;
use std::sync::Arc;
use std::time::Duration;

use quil_rs::program::ProgramError;
use quil_rs::quil::{Quil, ToQuilError};

use quil_rs::Program;
#[cfg(feature = "tracing")]
use tracing::trace;

use crate::compiler::rpcq;
use crate::executable::Parameters;
use crate::execution_data::{MemoryReferenceParseError, ResultData};
use crate::qpu::translation::translate;
use crate::{ExecutionData, JobHandle};

use super::api::{
    retrieve_results, submit, ConnectionStrategy, ExecutionOptions, ExecutionOptionsBuilder,
};
use super::translation::{EncryptedTranslationResult, TranslationOptions};
use super::QpuResultData;
use super::{get_isa, GetIsaError};
use crate::client::{GrpcClientError, Qcs};
use crate::compiler::quilc::{self, CompilerOpts, TargetDevice};

/// Contains all the info needed for a single run of an [`crate::Executable`] against a QPU. Can be
/// updated with fresh parameters in order to re-run the same program against the same QPU with the
/// same number of shots.
#[derive(Debug, Clone)]
pub(crate) struct Execution<'a> {
    program: Program,
    pub(crate) quantum_processor_id: Cow<'a, str>,
    pub(crate) shots: NonZeroU16,
    client: Arc<Qcs>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("problem processing the provided Quil: {0}")]
    Quil(#[from] ProgramError),
    #[error("problem converting the program to valid Quil: {0}")]
    ToQuil(#[from] ToQuilError),
    #[error("An error that is not expected to occur. If this shows up it may be a bug in this SDK or QCS")]
    Unexpected(#[from] Unexpected),
    #[error("Problem communicating with quilc at {uri}: {details}")]
    Quilc { uri: String, details: String },
    #[error("Problem using QCS API: {0}")]
    QcsClient(#[from] GrpcClientError),
    #[error(transparent)]
    Translation(#[from] super::translation::Error),
    #[error("Problem fetching ISA: {0}")]
    Isa(#[from] GetIsaError),
    #[error("Problem parsing memory readout: {0}")]
    ReadoutParse(#[from] MemoryReferenceParseError),
    #[error("Problem when compiling program: {details}")]
    Compilation { details: String },
    #[error("Problem when getting RPCQ client: {0}")]
    RpcqClient(#[from] rpcq::Error),
    #[error("Problem making a request to the QPU: {0}")]
    QpuApi(#[from] super::api::QpuApiError),
}

impl From<quilc::Error> for Error {
    fn from(source: quilc::Error) -> Self {
        match source {
            quilc::Error::Isa(source) => Self::Unexpected(Unexpected::Isa(format!("{source:?}"))),
            quilc::Error::QuilcConnection(uri, details) => Self::Quilc {
                uri,
                details: format!("{details:?}"),
            },
            quilc::Error::QuilcCompilation(details) => Self::Compilation {
                details: format!("{details:?}"),
            },
            quilc::Error::Parse(details) => Self::Compilation {
                details: format!("{details:?}"),
            },
        }
    }
}

/// Errors that are not expected to be returned—if they show up, it may be a bug in this library.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Unexpected {
    #[error("Problem converting QCS ISA to quilc ISA: {0}")]
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
        quilc_client: Option<Arc<dyn quilc::Client + Send + Sync>>,
        compiler_options: CompilerOpts,
    ) -> Result<Execution<'a>, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            num_shots=%shots,
            %quantum_processor_id,
            ?compiler_options,
            "creating new QPU Execution",
        );

        let isa = get_isa(quantum_processor_id.as_ref(), &client).await?;
        let target_device = TargetDevice::try_from(isa)?;

        let program = if let Some(client) = quilc_client {
            #[cfg(feature = "tracing")]
            trace!("Converting to Native Quil");
            client
                .compile_program(&quil, target_device, compiler_options)
                .map_err(|e| Error::Compilation {
                    details: e.to_string(),
                })?
                .program
        } else {
            #[cfg(feature = "tracing")]
            trace!("Skipping conversion to Native Quil");
            quil.parse().map_err(Error::Quil)?
        };

        Ok(Self {
            program,
            quantum_processor_id,
            shots,
            client,
        })
    }

    /// Translate the execution's quil program for it's given quantum processor.
    pub(crate) async fn translate(
        &mut self,
        options: Option<TranslationOptions>,
    ) -> Result<EncryptedTranslationResult, Error> {
        let encrpyted_translation_result = translate(
            self.quantum_processor_id.as_ref(),
            &self.program.to_quil()?,
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
        execution_options: &ExecutionOptions,
    ) -> Result<JobHandle<'a>, Error> {
        #[cfg(feature = "tracing")]
        tracing::debug!(quantum_processor_id=%self.quantum_processor_id, "submitting job to QPU");

        self.submit_to_target(
            params,
            Some(&self.quantum_processor_id.clone()),
            translation_options,
            execution_options,
        )
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
        self.submit_to_target(
            params,
            None,
            translation_options,
            &ExecutionOptionsBuilder::default()
                .connection_strategy(ConnectionStrategy::EndpointId(
                    endpoint_id.into().to_string(),
                ))
                .build()
                .expect("valid execution options"),
        )
        .await
    }

    async fn submit_to_target(
        &mut self,
        params: &Parameters,
        quantum_processor_id: Option<&str>,
        translation_options: Option<TranslationOptions>,
        execution_options: &ExecutionOptions,
    ) -> Result<JobHandle<'a>, Error> {
        let EncryptedTranslationResult { job, readout_map } =
            self.translate(translation_options).await?;

        let job_id = submit(
            quantum_processor_id,
            job,
            params,
            self.client.as_ref(),
            execution_options,
        )
        .await?;

        let endpoint_id = match execution_options.connection_strategy() {
            ConnectionStrategy::EndpointId(endpoint_id) => Some(endpoint_id),
            _ => None,
        };

        Ok(JobHandle::new(
            job_id,
            self.quantum_processor_id.to_string(),
            endpoint_id.cloned(),
            readout_map,
            execution_options.clone(),
        ))
    }

    pub(crate) async fn cancel_job(&self, job_handle: JobHandle<'a>) -> Result<(), Error> {
        crate::qpu::api::cancel_job(
            job_handle.job_id(),
            Some(job_handle.quantum_processor_id()),
            self.client.as_ref(),
            job_handle.execution_options(),
        )
        .await
        .map_err(Error::from)
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
            Some(job_handle.quantum_processor_id()),
            self.client.as_ref(),
            job_handle.execution_options(),
        )
        .await?;

        Ok(ExecutionData {
            result_data: ResultData::Qpu(QpuResultData::from_controller_mappings_and_values(
                job_handle.readout_map(),
                &response.readout_values,
                &response.memory_values,
            )),
            duration: Some(Duration::from_micros(
                response.execution_duration_microseconds,
            )),
        })
    }
}
