//! Contains QPU-specific executable stuff.

use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use log::{trace, warn};
use tokio::task::{spawn_blocking, JoinError};

use crate::configuration::Configuration;
use crate::executable::Parameters;
use crate::qpu::rewrite_arithmetic;
use crate::qpu::runner::JobId;
use crate::{ExecutionData, RegisterData};

use super::quilc::{self, NativeQuil, NativeQuilProgram, TargetDevice};
use super::rewrite_arithmetic::RewrittenProgram;
use super::rpcq::Client;
use super::runner::{self, retrieve_results, submit, DecodeError};
use super::translation::Error as TranslationError;
use super::{
    build_executable, engagement, get_isa, organize_ro_sources, process_buffers, IsaError,
};

/// Contains all the info needed for a single run of an [`crate::Executable`] against a QPU. Can be
/// updated with fresh parameters in order to re-run the same program against the same QPU with the
/// same number of shots.
#[derive(Clone)]
pub(crate) struct Execution<'a> {
    program: RewrittenProgram,
    pub(crate) quantum_processor_id: &'a str,
    pub(crate) shots: u16,
    // All the stuff needed to actually make requests to QCS, lazily initialized
    qcs: Option<Arc<Qcs>>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("No engagement for QPU: {0}")]
    NoEngagement(engagement::Error),
    #[error("QPU not found")]
    QpuNotFound,
    #[error("QPU currently unavailable, retry after {} seconds", .0.as_secs())]
    QpuUnavailable(Duration),
    #[error("Received unauthorized response from QCS, try refreshing credentials")]
    Unauthorized,
    #[error("Error communicating with QCS, check network connection and QCS status")]
    QcsCommunication,
    #[error("QCS returned an unrecognized error: {0}")]
    Qcs(String),
    #[error("problem processing the provided Quil: {0}")]
    Quil(String),
    #[error("An error that is not expected to occur. If this shows up it may be a bug in this SDK or QCS")]
    Unexpected(#[from] Unexpected),
    #[error("Problem communicating with quilc at {uri}: {details}")]
    Quilc { uri: String, details: String },
    #[error(
        "The program must first be submitted through the same Executable before retrieving results"
    )]
    ProgramNotSubmitted,
}

impl From<quilc::Error> for Error {
    fn from(source: quilc::Error) -> Self {
        match source {
            quilc::Error::Isa(source) => Self::Unexpected(Unexpected::Isa(format!("{:?}", source))),
            quilc::Error::QuilcConnection(uri, details) => Self::Quilc {
                uri,
                details: format!("{:?}", details),
            },
            quilc::Error::QuilcCompilation(details) => Self::Quil(details),
        }
    }
}

impl From<engagement::Error> for Error {
    fn from(source: engagement::Error) -> Self {
        match source {
            engagement::Error::QuantumProcessorUnavailable(duration) => {
                Self::QpuUnavailable(duration)
            }
            engagement::Error::Unauthorized => Self::Unauthorized,
            engagement::Error::Connection(_) => Self::QcsCommunication,
            engagement::Error::Schema(_)
            | engagement::Error::Unknown(_)
            | engagement::Error::Internal(_) => {
                Self::Unexpected(Unexpected::Qcs(format!("{:?}", source)))
            }
        }
    }
}

impl From<runner::Error> for Error {
    fn from(source: runner::Error) -> Self {
        match source {
            runner::Error::Engagement(e) => Self::NoEngagement(e),
            runner::Error::Connection(_) => Self::QcsCommunication,
            //runner::Error::Engagement(_) => Self::
            runner::Error::Unexpected(err) => {
                Self::Unexpected(Unexpected::Qcs(format!("{:?}", err)))
            }
            runner::Error::Qpu(err) => Self::Qcs(err),
            runner::Error::ClientLock(err) => Self::Unexpected(Unexpected::Other(err)),
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
    #[error("Problem understanding QCS")]
    Qcs(String),
    #[error("Unknown error")]
    Other(String),
}

impl From<IsaError> for Error {
    fn from(source: IsaError) -> Self {
        match source {
            IsaError::QpuNotFound => Self::QpuNotFound,
            IsaError::Unauthorized => Self::Unauthorized,
            IsaError::QcsError(source) => {
                Self::Unexpected(Unexpected::Qcs(format!("{:?}", source)))
            }
            IsaError::QcsCommunicationError(_) => Self::QcsCommunication,
        }
    }
}

impl From<TranslationError> for Error {
    fn from(source: TranslationError) -> Self {
        match source {
            TranslationError::ProgramIssue(inner) => Self::Quil(format!("{:?}", inner)),
            TranslationError::Connection(_) => Self::QcsCommunication,
            TranslationError::Serialization(inner) => {
                Self::Unexpected(Unexpected::Qcs(format!("{:?}", inner)))
            }
            TranslationError::Unknown(inner) => {
                Self::Unexpected(Unexpected::Qcs(format!("{:?}", inner)))
            }
            TranslationError::Unauthorized => Self::Unauthorized,
        }
    }
}

impl From<DecodeError> for Error {
    fn from(source: DecodeError) -> Self {
        Self::Unexpected(Unexpected::Qcs(format!("{:?}", source)))
    }
}

struct Qcs {
    /// A mapping of the register name declared in a program to the list of corresponding Buffer names
    buffer_names: HashMap<Box<str>, Vec<String>>,
    rpcq_client: Mutex<Client>,
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
        let isa = get_isa(quantum_processor_id, &config).await?;
        let target_device = TargetDevice::try_from(isa)?;

        let native_quil = if compile_with_quilc {
            trace!("Converting to Native Quil");
            let thread_config = config.clone();
            spawn_blocking(move || quilc::compile_program(&quil, target_device, &thread_config))
                .await
                .map_err(|source| {
                    Error::Unexpected(Unexpected::TaskError {
                        task_name: "quilc",
                        source,
                    })
                })??
        } else {
            trace!("Skipping conversion to Native Quil");
            NativeQuil::assume_native_quil(quil.to_string())
        };

        let program = NativeQuilProgram::try_from(native_quil).map_err(Error::Quil)?;

        Ok(Self {
            program: RewrittenProgram::try_from(program).map_err(|e| Error::Quil(e.to_string()))?,
            quantum_processor_id,
            shots,
            qcs: None,
        })
    }

    /// Run on a real QPU and wait for the results.
    pub(crate) async fn submit(
        &mut self,
        params: &Parameters,
        readouts: &[&str],
        config: &Configuration,
    ) -> Result<JobId, Error> {
        let qcs = self.refresh_qcs(readouts, config).await?;
        let qcs_for_thread = qcs.clone();

        let patch_values = self.get_substitutions(params).map_err(Error::Quil)?;

        spawn_blocking(move || {
            let guard = qcs_for_thread.rpcq_client.lock().unwrap();
            submit(&qcs_for_thread.executable, &patch_values, &guard).map_err(Error::from)
        })
        .await
        .map_err(|source| Unexpected::TaskError {
            task_name: "qpu",
            source,
        })?
    }

    pub(crate) async fn retrieve_results(&self, job_id: JobId) -> Result<ExecutionData, Error> {
        let qcs = self.qcs.clone().ok_or(Error::ProgramNotSubmitted)?;
        let qcs_for_thread = qcs.clone();

        let response = spawn_blocking(move || {
            let guard = qcs_for_thread.rpcq_client.lock().unwrap();
            retrieve_results(job_id, &guard).map_err(Error::from)
        })
        .await
        .map_err(|source| Unexpected::TaskError {
            task_name: "qpu",
            source,
        })??;

        let registers = process_buffers(response.buffers, &qcs.buffer_names)?;
        let register_data = RegisterData::try_from_registers(registers, self.shots)?;
        Ok(ExecutionData {
            registers: register_data,
            duration: response
                .execution_duration_microseconds
                .map(Duration::from_micros),
        })
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
        .await?;
        let ro_sources = response.ro_sources.ok_or_else(|| {
            Error::Quil(String::from(
                "No readout sources defined, did you forget to MEASURE?",
            ))
        })?;
        let buffer_names = organize_ro_sources(ro_sources, readouts)?;
        let engagement = engagement::get(String::from(self.quantum_processor_id), config).await?;
        let rpcq_client = Client::try_from(&engagement)
            .map_err(|e| {
                warn!("Unable to connect to QPU via RPCQ: {:?}", e);
                Error::QcsCommunication
            })
            .map(Mutex::new)?;
        let qcs = Arc::new(Qcs {
            buffer_names,
            rpcq_client,
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
    fn get_substitutions(&self, params: &Parameters) -> Result<Parameters, String> {
        rewrite_arithmetic::get_substitutions(&self.program.substitutions, params)
    }
}
