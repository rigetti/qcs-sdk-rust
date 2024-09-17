//! This module contains the public-facing API for executing programs. [`Executable`] is the how
//! users will interact with QCS, quilc, and QVM.

use std::borrow::Cow;
use std::collections::HashMap;
use std::num::NonZeroU16;
use std::sync::Arc;
use std::time::Duration;

use qcs_api_client_common::configuration::LoadError;
use quil_rs::quil::ToQuilError;

use crate::client::Qcs;
use crate::compiler::quilc::{self, CompilerOpts};
use crate::execution_data::{self, ResultData};
use crate::qpu::api::{ExecutionOptions, JobId};
use crate::qpu::translation::TranslationOptions;
use crate::qpu::ExecutionError;
use crate::qvm::http::AddressRequest;
use crate::{qpu, qvm};
use quil_rs::program::ProgramError;

/// The builder interface for executing Quil programs on QVMs and QPUs.
///
/// # Example
///
/// ```rust
/// use qcs::client::Qcs;
/// use qcs::Executable;
/// use qcs::qvm;
///
///
/// const PROGRAM: &str = r##"
/// DECLARE ro BIT[2]
///
/// H 0
/// CNOT 0 1
///
/// MEASURE 0 ro[0]
/// MEASURE 1 ro[1]
/// "##;
///
/// #[tokio::main]
/// async fn main() {
///     use std::num::NonZeroU16;
///     use qcs::qvm;
///     let qvm_client = qvm::http::HttpClient::from(&Qcs::load());
///     let mut result = Executable::from_quil(PROGRAM).with_qcs_client(Qcs::default()).with_shots(NonZeroU16::new(4).unwrap()).execute_on_qvm(&qvm_client).await.unwrap();
///     // "ro" is the only source read from by default if you don't specify a .read_from()
///
///     // We first convert the readout data to a [`RegisterMap`] to get a mapping of registers
///     // (ie. "ro") to a [`RegisterMatrix`], `M`, where M[`shot`][`index`] is the value for
///     // the memory offset `index` during shot `shot`.
///     // There are some programs where QPU readout data does not fit into a [`RegisterMap`], in
///     // which case you should build the matrix you need from [`QpuResultData`] directly. See
///     // the [`RegisterMap`] documentation for more information on when this transformation
///     // might fail.
///     let data = result.result_data
///                         .to_register_map()
///                         .expect("should convert to readout map")
///                         .get_register_matrix("ro")
///                         .expect("should have data in ro")
///                         .as_integer()
///                         .expect("should be integer matrix")
///                         .to_owned();
///
///     // In this case, we ran the program for 4 shots, so we know the number of rows is 4.
///     assert_eq!(data.nrows(), 4);
///     for shot in data.rows() {
///         // Each shot will contain all the memory, in order, for the vector (or "register") we
///         // requested the results of. In this case, "ro" (the default).
///         assert_eq!(shot.len(), 2);
///         // In the case of this particular program, we know ro[0] should equal ro[1]
///         assert_eq!(shot[0], shot[1]);
///     }
/// }
///
/// ```
///
/// # A Note on Lifetimes
///
/// This structure utilizes multiple lifetimes for the sake of runtime efficiency.
/// You should be able to largely ignore these, just keep in mind that any borrowed data passed to
/// the methods most likely needs to live as long as this struct. Check individual methods for
/// specifics. If only using `'static` strings then everything should just work.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Executable<'executable, 'execution> {
    quil: Arc<str>,
    shots: NonZeroU16,
    readout_memory_region_names: Option<Vec<Cow<'executable, str>>>,
    params: Parameters,
    qcs_client: Option<Arc<Qcs>>,
    quilc_client: Option<Arc<dyn quilc::Client + Send + Sync>>,
    compiler_options: CompilerOpts,
    qpu: Option<qpu::Execution<'execution>>,
    qvm: Option<qvm::Execution>,
}

pub(crate) type Parameters = HashMap<Box<str>, Vec<f64>>;

impl<'executable> Executable<'executable, '_> {
    /// Create an [`Executable`] from a string containing a  [quil](https://github.com/quil-lang/quil)
    /// program. No additional work is done in this function, so the `quil` may actually be invalid.
    ///
    /// The constructed [`Executable`] defaults to "ro" as a read-out register and 1 for the number
    /// of shots. Those can be overridden using [`Executable::read_from`] and
    /// [`Executable::with_shots`] respectively.
    ///
    /// Note that changing the program for an associated [`Executable`] is not allowed, you'll have to
    /// create a new [`Executable`] if you want to run a different program.
    ///
    /// # Arguments
    ///
    /// 1. `quil` is a string slice representing the original program to be run. The returned
    ///     [`Executable`] will only live as long as this reference.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn from_quil<Quil: Into<Arc<str>>>(quil: Quil) -> Self {
        Self {
            quil: quil.into(),
            shots: NonZeroU16::new(1).expect("value is non-zero"),
            readout_memory_region_names: None,
            params: Parameters::new(),
            compiler_options: CompilerOpts::default(),
            qpu: None,
            qvm: None,
            qcs_client: None,
            quilc_client: None,
        }
    }

    /// Specify a memory region or "register" to read results from. This must correspond to a
    /// `DECLARE` statement in the provided Quil program. You can call this register multiple times
    /// if you need to read multiple registers. If this method is never called, it's
    /// assumed that a single register called "ro" is declared and should be read from.
    ///
    /// # Arguments
    ///
    /// 1. `register` is a string reference of the name of a register to read from. The lifetime
    ///     of this reference should be the lifetime of the [`Executable`], which is the lifetime of
    ///     the `quil` argument to [`Executable::from_quil`].
    ///
    /// # Example
    ///
    /// ```rust
    /// use qcs::client::Qcs;
    /// use qcs::Executable;
    /// use qcs::qvm;
    ///
    /// const PROGRAM: &str = r#"
    /// DECLARE first REAL[1]
    /// DECLARE second REAL[1]
    ///
    /// MOVE first[0] 3.141
    /// MOVE second[0] 1.234
    /// "#;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let qvm_client = qvm::http::HttpClient::from(&Qcs::load());
    ///     let mut result = Executable::from_quil(PROGRAM)
    ///         .with_qcs_client(Qcs::default()) // Unnecessary if you have ~/.qcs/settings.toml
    ///         .read_from("first")
    ///         .read_from("second")
    ///         .execute_on_qvm(&qvm_client)
    ///         .await
    ///         .unwrap();
    ///     let first_value = result
    ///         .result_data
    ///         .to_register_map()
    ///         .expect("qvm memory should fit readout map")
    ///         .get_register_matrix("first")
    ///         .expect("readout map should have 'first'")
    ///         .as_real()
    ///         .expect("should be real numbered register")
    ///         .get((0, 0))
    ///         .expect("should have value in first position of first register")
    ///         .clone();
    ///     let second_value = result
    ///         .result_data
    ///         .to_register_map()
    ///         .expect("qvm memory should fit readout map")
    ///         .get_register_matrix("second")
    ///         .expect("readout map should have 'second'")
    ///         .as_real()
    ///         .expect("should be real numbered register")
    ///         .get((0, 0))
    ///         .expect("should have value in first position of first register")
    ///         .clone();
    ///     assert_eq!(first_value, 3.141);
    ///     assert_eq!(second_value, 1.234);
    /// }
    /// ```
    #[must_use]
    pub fn read_from<S>(mut self, register: S) -> Self
    where
        S: Into<Cow<'executable, str>>,
    {
        let register = register.into();
        #[cfg(feature = "tracing")]
        tracing::trace!("reading from register {:?}", register);
        let mut readouts = self.readout_memory_region_names.take().unwrap_or_default();
        readouts.push(register);
        self.readout_memory_region_names = Some(readouts);
        self
    }

    /// Sets a concrete value for [parametric compilation].
    /// The validity of parameters is not checked until execution.
    ///
    /// # Arguments
    ///
    /// 1. `param_name`: Reference to the name of the parameter which should correspond to a
    ///     `DECLARE` statement in the Quil program. The lifetime of the reference should be the
    ///     same as the [`Executable`]: that is the same as the `quil` param to [`Executable::from_quil`].
    /// 2. `index`: The index into the memory vector that you're setting.
    /// 3. `value`: The value to set for the specified memory.
    ///
    /// # Example
    ///
    /// ```rust
    /// use qcs::client::Qcs;
    /// use qcs::Executable;
    /// use qcs::qvm;
    ///
    /// const PROGRAM: &str = "DECLARE theta REAL[2]";
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let qvm_client = qvm::http::HttpClient::from(&Qcs::load());
    ///     let mut exe = Executable::from_quil(PROGRAM)
    ///         .with_qcs_client(Qcs::default()) // Unnecessary if you have ~/.qcs/settings.toml
    ///         .read_from("theta");
    ///
    ///     for theta in 0..2 {
    ///         let theta = theta as f64;
    ///         let mut result = exe
    ///             .with_parameter("theta", 0, theta)
    ///             .with_parameter("theta", 1, theta * 2.0)
    ///             .execute_on_qvm(&qvm_client).await.unwrap();
    ///         let theta_register = result
    ///             .result_data
    ///             .to_register_map()
    ///             .expect("should fit readout map")
    ///             .get_register_matrix("theta")
    ///             .expect("should have theta")
    ///             .as_real()
    ///             .expect("should be real valued register")
    ///             .to_owned();
    ///
    ///         let first = theta_register
    ///             .get((0, 0))
    ///             .expect("first index, first shot of theta should have value")
    ///             .to_owned();
    ///         let second = theta_register
    ///             .get((0, 1))
    ///             .expect("first shot, second_index of theta should have value")
    ///             .to_owned();
    ///
    ///         assert_eq!(first, theta);
    ///         assert_eq!(second, theta * 2.0);
    ///     }
    /// }
    /// ```
    ///
    /// [parametric compilation]: https://pyquil-docs.rigetti.com/en/stable/basics.html?highlight=parametric#parametric-compilation
    pub fn with_parameter<Param: Into<Box<str>>>(
        &mut self,
        param_name: Param,
        index: usize,
        value: f64,
    ) -> &mut Self {
        let param_name = param_name.into();

        #[cfg(feature = "tracing")]
        tracing::trace!("setting parameter {}[{}] to {}", param_name, index, value);

        let mut values = self
            .params
            .remove(&param_name)
            .unwrap_or_else(|| vec![0.0; index]);

        if index >= values.len() {
            values.resize(index + 1, 0.0);
        }

        values[index] = value;
        self.params.insert(param_name, values);

        self
    }

    /// Set the default configuration to be used when constructing clients
    #[must_use]
    pub fn with_qcs_client(mut self, client: Qcs) -> Self {
        self.qcs_client = Some(Arc::from(client));
        self
    }

    /// Get a reference to the [`Qcs`] client used by the executable.
    ///
    /// If one has not been set, a default client is loaded, set, and returned.
    pub fn qcs_client(&mut self) -> Arc<Qcs> {
        if let Some(client) = &self.qcs_client {
            client.clone()
        } else {
            let client = Arc::new(Qcs::load());
            self.qcs_client = Some(client.clone());
            client
        }
    }
}

/// The [`Result`] from executing on a QPU or QVM.
pub type ExecutionResult = Result<execution_data::ExecutionData, Error>;

impl Executable<'_, '_> {
    /// Specify a number of times to run the program for each execution. Defaults to 1 run or "shot".
    #[must_use]
    pub fn with_shots(mut self, shots: NonZeroU16) -> Self {
        self.shots = shots;
        self
    }

    /// Set the client used for compilation.
    ///
    /// To disable compilation, set this to `None`.
    #[must_use]
    #[allow(trivial_casts)]
    pub fn with_quilc_client<C: quilc::Client + Send + Sync + 'static>(
        mut self,
        client: Option<C>,
    ) -> Self {
        self.quilc_client = client.map(|c| Arc::new(c) as _);
        self
    }

    /// If set, the value will override the default compiler options
    #[must_use]
    pub fn compiler_options(mut self, options: CompilerOpts) -> Self {
        self.compiler_options = options;
        self
    }

    fn get_readouts(&self) -> &[Cow<'_, str>] {
        return self
            .readout_memory_region_names
            .as_ref()
            .map_or(&[Cow::Borrowed("ro")], Vec::as_slice);
    }

    /// Execute on a QVM which must be available at the configured URL (default <http://localhost:5000>).
    ///
    /// # Warning
    ///
    /// This function uses [`tokio::task::spawn_blocking`] internally. See the docs for that function
    /// to avoid blocking shutdown of the runtime.
    ///
    /// # Returns
    ///
    /// An [`ExecutionResult`].
    ///
    /// # Errors
    ///
    /// See [`Error`].
    pub async fn execute_on_qvm<V: qvm::Client + ?Sized>(&mut self, client: &V) -> ExecutionResult {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            num_shots = %self.shots,
            "running Executable on QVM",
        );

        let qvm = if let Some(qvm) = self.qvm.take() {
            qvm
        } else {
            qvm::Execution::new(&self.quil)?
        };
        let result = qvm
            .run(
                self.shots,
                self.get_readouts()
                    .iter()
                    .map(|address| (address.to_string(), AddressRequest::IncludeAll))
                    .collect(),
                &self.params,
                client,
            )
            .await;
        self.qvm = Some(qvm);
        result
            .map_err(Error::from)
            .map(|registers| execution_data::ExecutionData {
                result_data: ResultData::Qvm(registers),
                duration: None,
            })
    }
}

impl<'execution> Executable<'_, 'execution> {
    /// Remove and return `self.qpu` if it's set and still valid. Otherwise, create a new one.
    async fn qpu_for_id<S>(&mut self, id: S) -> Result<qpu::Execution<'execution>, Error>
    where
        S: Into<Cow<'execution, str>>,
    {
        let id = id.into();
        if let Some(qpu) = self.qpu.take() {
            if qpu.quantum_processor_id == id.as_ref() && qpu.shots == self.shots {
                return Ok(qpu);
            }
        }
        qpu::Execution::new(
            self.quil.clone(),
            self.shots,
            id,
            self.qcs_client(),
            self.quilc_client.clone(),
            self.compiler_options,
        )
        .await
        .map_err(Error::from)
    }

    /// Compile the program and execute it on a QPU, waiting for results.
    ///
    /// # Arguments
    /// 1. `quantum_processor_id`: The name of the QPU to run on. This parameter affects the
    ///     lifetime of the [`Executable`]. The [`Executable`] will only live as long as the last
    ///     parameter passed into this function.
    ///
    /// # Warning
    ///
    /// This function uses [`tokio::task::spawn_blocking`] internally. See the docs for that function
    /// to avoid blocking shutdown of the runtime.
    ///
    /// # Returns
    ///
    /// An [`ExecutionResult`].
    ///
    /// # Errors
    /// All errors are human readable by way of [`mod@thiserror`]. Some common errors are:
    ///
    /// 1. You are not authenticated for QCS
    /// 1. Your credentials don't have an active reservation for the QPU you requested
    /// 1. [quilc] was not running.
    /// 1. The `quil` that this [`Executable`] was constructed with was invalid.
    /// 1. Missing parameters that should be filled with [`Executable::with_parameter`]
    ///
    /// [quilc]: https://github.com/quil-lang/quilc
    pub async fn execute_on_qpu_with_endpoint<S>(
        &mut self,
        quantum_processor_id: S,
        endpoint_id: S,
        translation_options: Option<TranslationOptions>,
    ) -> ExecutionResult
    where
        S: Into<Cow<'execution, str>>,
    {
        let job_handle = self
            .submit_to_qpu_with_endpoint(quantum_processor_id, endpoint_id, translation_options)
            .await?;
        self.retrieve_results(job_handle).await
    }

    /// Compile the program and execute it on a QCS endpoint, waiting for results.
    ///
    /// # Arguments
    /// 1. `quantum_processor_id`: The ID of the QPU for which to translate the program.
    ///     This parameter affects the lifetime of the [`Executable`].
    ///     The [`Executable`] will only live as long as the last parameter passed into this function.
    /// 2. `translation_options`: An optional [`TranslationOptions`] that is used to configure how
    ///    the program in translated.
    /// 3. `execution_options`: The [`ExecutionOptions`] to use. If the connection strategy used
    ///       is [`crate::qpu::api::ConnectionStrategy::EndpointId`] then direct access to that endpoint
    ///       overrides the `quantum_processor_id` parameter.
    ///
    /// # Warning
    ///
    /// This function uses [`tokio::task::spawn_blocking`] internally. See the docs for that function
    /// to avoid blocking shutdown of the runtime.
    ///
    /// # Returns
    ///
    /// An [`ExecutionResult`].
    ///
    /// # Errors
    /// All errors are human readable by way of [`mod@thiserror`]. Some common errors are:
    ///
    /// 1. You are not authenticated for QCS
    /// 1. Your credentials don't have an active reservation for the QPU you requested
    /// 1. [quilc] was not running.
    /// 1. The `quil` that this [`Executable`] was constructed with was invalid.
    /// 1. Missing parameters that should be filled with [`Executable::with_parameter`]
    ///
    /// [quilc]: https://github.com/quil-lang/quilc
    pub async fn execute_on_qpu<S>(
        &mut self,
        quantum_processor_id: S,
        translation_options: Option<TranslationOptions>,
        execution_options: &ExecutionOptions,
    ) -> ExecutionResult
    where
        S: Into<Cow<'execution, str>>,
    {
        let quantum_processor_id = quantum_processor_id.into();

        #[cfg(feature = "tracing")]
        tracing::debug!(
            num_shots = %self.shots,
            %quantum_processor_id,
            "running Executable on QPU",
        );

        let job_handle = self
            .submit_to_qpu(quantum_processor_id, translation_options, execution_options)
            .await?;
        self.retrieve_results(job_handle).await
    }

    /// Compile and submit the program to a QPU, but do not wait for execution to complete.
    ///
    /// Call [`Executable::retrieve_results`] to wait for execution to complete and retrieve the
    /// results.
    ///
    /// # Arguments
    /// 1. `quantum_processor_id`: The ID of the QPU for which to translate the program.
    ///     This parameter affects the lifetime of the [`Executable`].
    ///     The [`Executable`] will only live as long as the last parameter passed into this function.
    /// 2. `translation_options`: An optional [`TranslationOptions`] that is used to configure how
    ///    the program in translated.
    /// 3. `execution_options`: The [`ExecutionOptions`] to use. If the connection strategy used
    ///       is [`crate::qpu::api::ConnectionStrategy::EndpointId`] then direct access to that endpoint
    ///       overrides the `quantum_processor_id` parameter.
    ///
    /// # Errors
    ///
    /// See [`Executable::execute_on_qpu`].
    pub async fn submit_to_qpu<S>(
        &mut self,
        quantum_processor_id: S,
        translation_options: Option<TranslationOptions>,
        execution_options: &ExecutionOptions,
    ) -> Result<JobHandle<'execution>, Error>
    where
        S: Into<Cow<'execution, str>>,
    {
        let quantum_processor_id = quantum_processor_id.into();

        #[cfg(feature = "tracing")]
        tracing::debug!(
            num_shots = %self.shots,
            %quantum_processor_id,
            "submitting Executable to QPU",
        );

        let job_handle = self
            .qpu_for_id(quantum_processor_id)
            .await?
            .submit(&self.params, translation_options, execution_options)
            .await?;
        Ok(job_handle)
    }

    /// Compile and submit the program to a QCS endpoint, but do not wait for execution to complete.
    ///
    /// Call [`Executable::retrieve_results`] to wait for execution to complete and retrieve the
    /// results.
    ///
    /// # Errors
    ///
    /// See [`Executable::execute_on_qpu`].
    pub async fn submit_to_qpu_with_endpoint<S>(
        &mut self,
        quantum_processor_id: S,
        endpoint_id: S,
        translation_options: Option<TranslationOptions>,
    ) -> Result<JobHandle<'execution>, Error>
    where
        S: Into<Cow<'execution, str>>,
    {
        let job_handle = self
            .qpu_for_id(quantum_processor_id)
            .await?
            .submit_to_endpoint_id(&self.params, endpoint_id.into(), translation_options)
            .await?;
        Ok(job_handle)
    }

    /// Cancel a job that has yet to begin executing.
    ///
    /// This action is *not* atomic, and will attempt to cancel a job even if it cannot be cancelled. A
    /// job can be cancelled only if it has not yet started executing.
    ///
    /// Success response indicates only that the request was received. Cancellation is not guaranteed,
    /// as it is based on job state at the time of cancellation, and is completed on a best effort
    /// basis.
    pub async fn cancel_qpu_job(&mut self, job_handle: JobHandle<'execution>) -> Result<(), Error> {
        let quantum_processor_id = job_handle.quantum_processor_id.to_string();
        let qpu = self.qpu_for_id(quantum_processor_id).await?;
        Ok(qpu.cancel_job(job_handle).await?)
    }

    /// Wait for the results of a job submitted via [`Executable::submit_to_qpu`] to complete.
    ///
    /// # Errors
    ///
    /// See [`Executable::execute_on_qpu`].
    pub async fn retrieve_results(&mut self, job_handle: JobHandle<'execution>) -> ExecutionResult {
        let quantum_processor_id = job_handle.quantum_processor_id.to_string();
        let qpu = self.qpu_for_id(quantum_processor_id).await?;
        qpu.retrieve_results(job_handle).await.map_err(Error::from)
    }
}

/// The possible errors which can be returned by [`Executable::execute_on_qpu`] and
/// [`Executable::execute_on_qvm`]..
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Communicating with QCS requires appropriate settings and secrets files. By default, these
    /// should be `$HOME/.qcs/settings.toml` and `$HOME/.qcs/secrets.toml`, though those files can
    /// be overridden by setting the `QCS_SETTINGS_FILE_PATH` and `QCS_SECRETS_FILE_PATH`
    /// environment variables.
    ///
    /// This error can occur when one of those files is required but missing or there is a problem
    /// with the contents of those files.
    #[error("There was a problem related to your QCS settings: {0}")]
    Settings(String),
    /// This error occurs when the SDK was unable to authenticate a request to QCS. This could mean
    /// that your credentials are invalid or expired, or that you do not have access to the requested
    /// QPU.
    #[error("Could not authenticate a request to QCS for the requested QPU.")]
    Authentication,
    /// An API error occurred while connecting to the QPU.
    #[error("An API error occurred while connecting to the QPU: {0}")]
    QpuApiError(#[from] qpu::api::QpuApiError),
    /// This happens when the QPU is down for maintenance and not accepting new jobs. If you receive
    /// this error, internal compilation caches will have been cleared as programs should be recompiled
    /// with new settings after a maintenance window. If you are mid-experiment, you might want to
    /// start over.
    #[error("QPU currently unavailable, retry after {} seconds", .0.as_secs())]
    QpuUnavailable(Duration),
    /// Indicates a problem connecting to an external service. Check your network connection and
    /// ensure that any required local services (e.g., `qvm` or `quilc`) are running.
    #[error("Error connecting to service {0:?}")]
    Connection(Service),
    /// There was some problem with the provided Quil program. This could be a syntax error with
    /// quil, providing Quil-T to `quilc` or `qvm` (which is not supported), or forgetting to set
    /// some parameters.
    #[error("There was a problem with the Quil program: {0}")]
    Quil(#[from] ProgramError),
    /// There was some problem converting the provided Quil program to valid Quil.
    #[error("There was a problem converting the program to valid Quil: {0}")]
    ToQuil(#[from] ToQuilError),
    /// There was a problem when compiling the Quil program.
    #[error("There was a problem compiling the Quil program: {0}")]
    Compilation(String),
    /// There was a problem when translating the Quil program.
    #[error("There was a problem translating the Quil program: {0}")]
    Translation(String),
    /// There was a problem when substituting parameters in the Quil program.
    #[error("There was a problem substituting parameters in the Quil program: {0}")]
    Substitution(String),
    /// The Quil program is missing readout sources.
    #[error("The Quil program is missing readout sources")]
    MissingRoSources,
    /// This error returns when a runtime check that _should_ always pass fails. This most likely
    /// indicates a bug in the SDK and should be reported to
    /// [GitHub](https://github.com/rigetti/qcs-sdk-rust/issues),
    #[error("An unexpected error occurred, please open an issue on GitHub: {0:?}")]
    Unexpected(String),
    /// Occurs when [`Executable::retrieve_results`] is called with an invalid [`JobHandle`].
    /// Calling functions on [`Executable`] between [`Executable::submit_to_qpu`] and
    /// [`Executable::retrieve_results`] can invalidate the handle.
    #[error("The job handle was not valid")]
    InvalidJobHandle,
    /// Occurs when failing to construct a [`Qcs`] client.
    #[error("The QCS client configuration failed to load")]
    QcsConfigLoadFailure(#[from] LoadError),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
/// The external services that this SDK may connect to. Used to differentiate between networking
/// issues in [`Error::Connection`].
pub enum Service {
    /// The open source [`quilc`](https://github.com/quil-lang/quilc) compiler.
    ///
    /// This compiler must be running before calling [`Executable::execute_on_qpu`] unless the
    /// [`Executable::compile_with_quilc`] option is set to `false`. By default, it's assumed that
    /// this is running on `tcp://localhost:5555`, but this can be overridden via
    /// `[profiles.<profile_name>.applications.pyquil.quilc_url]` in your `.qcs/settings.toml` file.
    Quilc,
    /// The open source [`qvm`](https://github.com/quil-lang/qvm) simulator.
    ///
    /// This simulator must be running before calling [`Executable::execute_on_qvm`]. By default,
    /// it's assumed that this is running on `http://localhost:5000`, but this can be overridden via
    /// `[profiles.<profile_name>.applications.pyquil.qvm_url]` in your `.qcs/settings.toml` file.
    Qvm,
    /// The connection to [`QCS`](https://docs.rigetti.com/qcs/), the API for authentication,
    /// QPU lookup, and translation.
    ///
    /// You should be able to reach this service as long as you have a connection to the internet.
    Qcs,
    /// The connection to the QPU itself. You can only connect to the QPU from an authorized network
    /// (like QCS JupyterLab).
    Qpu,
}

impl From<ExecutionError> for Error {
    fn from(err: ExecutionError) -> Self {
        match err {
            ExecutionError::Unexpected(inner) => Self::Unexpected(format!("{inner:?}")),
            ExecutionError::Quilc { .. } => Self::Connection(Service::Quilc),
            ExecutionError::QcsClient(v) => Self::Unexpected(format!("{v:?}")),
            ExecutionError::Translation(v) => Self::Translation(v.to_string()),
            ExecutionError::Isa(v) => Self::Unexpected(format!("{v:?}")),
            ExecutionError::ReadoutParse(v) => Self::Unexpected(format!("{v:?}")),
            ExecutionError::Quil(e) => Self::Quil(e),
            ExecutionError::ToQuil(e) => Self::ToQuil(e),
            ExecutionError::Compilation { details } => Self::Compilation(details),
            ExecutionError::RpcqClient(e) => Self::Unexpected(format!("{e:?}")),
            ExecutionError::QpuApi(e) => Self::QpuApiError(e),
        }
    }
}

impl From<qvm::Error> for Error {
    fn from(err: qvm::Error) -> Self {
        match err {
            qvm::Error::QvmCommunication { .. } | qvm::Error::Client { .. } => {
                Self::Connection(Service::Qvm)
            }
            qvm::Error::ToQuil(q) => Self::ToQuil(q),
            qvm::Error::Parsing(_)
            | qvm::Error::ShotsMustBePositive
            | qvm::Error::RegionSizeMismatch { .. }
            | qvm::Error::RegionNotFound { .. }
            | qvm::Error::Qvm { .. } => Self::Compilation(format!("{err}")),
        }
    }
}

/// The result of calling [`Executable::submit_to_qpu`]. Represents a quantum program running on
/// a QPU. Can be passed to [`Executable::retrieve_results`] to retrieve the results of the job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobHandle<'executable> {
    job_id: JobId,
    quantum_processor_id: Cow<'executable, str>,
    endpoint_id: Option<Cow<'executable, str>>,
    readout_map: HashMap<String, String>,
    execution_options: ExecutionOptions,
}

impl<'a> JobHandle<'a> {
    #[must_use]
    pub(crate) fn new<S>(
        job_id: JobId,
        quantum_processor_id: S,
        endpoint_id: Option<S>,
        readout_map: HashMap<String, String>,
        execution_options: ExecutionOptions,
    ) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            job_id,
            quantum_processor_id: quantum_processor_id.into(),
            endpoint_id: endpoint_id.map(Into::into),
            readout_map,
            execution_options,
        }
    }

    /// The string representation of the QCS Job ID. Useful for debugging.
    #[must_use]
    pub fn job_id(&self) -> JobId {
        self.job_id.clone()
    }

    /// The ID of the quantum processor to which the job was submitted.
    #[must_use]
    pub fn quantum_processor_id(&self) -> &str {
        &self.quantum_processor_id
    }

    /// The readout map from source readout memory locations to the
    /// filter pipeline node which publishes the data.
    #[must_use]
    pub fn readout_map(&self) -> &HashMap<String, String> {
        &self.readout_map
    }

    /// The [`ExecutionOptions`] used to submit the job to the QPU.
    #[must_use]
    pub fn execution_options(&self) -> &ExecutionOptions {
        &self.execution_options
    }
}

#[cfg(test)]
#[cfg(feature = "manual-tests")]
mod describe_get_config {
    use crate::client::Qcs;
    use crate::{compiler::rpcq, Executable};

    fn quilc_client() -> rpcq::Client {
        let qcs = Qcs::load();
        let endpoint = qcs.get_config().quilc_url();
        rpcq::Client::new(endpoint).unwrap()
    }

    #[tokio::test]
    async fn it_resizes_params_dynamically() {
        let mut exe = Executable::from_quil("").with_quilc_client(Some(quilc_client()));

        exe.with_parameter("foo", 0, 0.0);
        let params = exe.params.get("foo").unwrap().len();
        assert_eq!(params, 1);

        exe.with_parameter("foo", 10, 10.0);
        let params = exe.params.get("foo").unwrap().len();
        assert_eq!(params, 11);
    }
}

#[cfg(test)]
#[cfg(feature = "manual-tests")]
mod describe_qpu_for_id {
    use assert2::let_assert;
    use std::num::NonZeroU16;

    use crate::compiler::quilc::CompilerOpts;
    use crate::compiler::rpcq;
    use crate::qpu;
    use crate::{client::Qcs, Executable};

    fn quilc_client() -> rpcq::Client {
        let qcs = Qcs::load();
        let endpoint = qcs.get_config().quilc_url();
        rpcq::Client::new(endpoint).unwrap()
    }

    #[tokio::test]
    async fn it_refreshes_auth_token() {
        // Default config has no auth, so it should try to refresh
        let mut exe = Executable::from_quil("")
            .with_qcs_client(Qcs::load())
            .with_quilc_client(Some(quilc_client()));
        let result = exe.qpu_for_id("blah").await;
        let Err(err) = result else {
            panic!("Expected an error!");
        };
        let result_string = format!("{err:?}");
        assert!(result_string.contains("refresh_token"));
    }

    #[tokio::test]
    async fn it_loads_cached_version() {
        let mut exe = Executable::from_quil("").with_quilc_client(Some(quilc_client()));
        let shots = NonZeroU16::new(17).expect("value is non-zero");
        exe.shots = shots;
        exe.qpu = Some(
            qpu::Execution::new(
                "".into(),
                shots,
                "Aspen-M-3".into(),
                exe.qcs_client(),
                exe.quilc_client.clone(),
                CompilerOpts::default(),
            )
            .await
            .unwrap(),
        );
        // Load config with no credentials to prevent creating a new Execution if it tries
        let mut exe = exe.with_qcs_client(Qcs::default());

        assert!(exe.qpu_for_id("Aspen-M-3").await.is_ok());
    }

    #[tokio::test]
    async fn it_creates_new_after_shot_change() {
        let original_shots = NonZeroU16::new(23).expect("value is non-zero");
        let mut exe = Executable::from_quil("")
            .with_quilc_client(Some(quilc_client()))
            .with_shots(original_shots);
        let qpu = exe.qpu_for_id("Aspen-9").await.unwrap();

        assert_eq!(qpu.shots, original_shots);

        // Cache so we can verify cache is not used.
        exe.qpu = Some(qpu);
        let new_shots = NonZeroU16::new(32).expect("value is non-zero");
        exe = exe.with_shots(new_shots);
        let qpu = exe.qpu_for_id("Aspen-9").await.unwrap();

        assert_eq!(qpu.shots, new_shots);
    }

    #[tokio::test]
    async fn it_creates_new_for_new_qpu_id() {
        let mut exe = Executable::from_quil("").with_quilc_client(Some(quilc_client()));
        let qpu = exe.qpu_for_id("Aspen-9").await.unwrap();

        assert_eq!(qpu.quantum_processor_id, "Aspen-9");

        // Cache so we can verify cache is not used.
        exe.qpu = Some(qpu);
        // Load config with no credentials to prevent creating the new Execution (which would fail anyway)
        let mut exe = exe.with_qcs_client(Qcs::default());
        let result = exe.qpu_for_id("Aspen-8").await;

        let_assert!(Err(crate::executable::Error::Unexpected(err)) = result);
        assert!(err.contains("NoRefreshToken"));
        assert!(exe.qpu.is_none());
    }
}
