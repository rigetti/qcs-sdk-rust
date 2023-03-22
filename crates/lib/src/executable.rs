//! This module contains the public-facing API for executing programs. [`Executable`] is the how
//! users will interact with QCS, quilc, and QVM.

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use qcs_api_client_common::configuration::LoadError;
use qcs_api_client_common::ClientConfiguration;

use crate::compiler::quilc::CompilerOpts;
use crate::execution_data::{self, ResultData};
use crate::qpu::api::{JobId, JobTarget};
use crate::qpu::client::Qcs;
use crate::qpu::rewrite_arithmetic;
use crate::qpu::ExecutionError;
use crate::{qpu, qvm};
use quil_rs::program::ProgramError;
use quil_rs::Program;

/// The builder interface for executing Quil programs on QVMs and QPUs.
///
/// # Example
///
/// ```rust
/// use qcs_api_client_common::ClientConfiguration;
/// use qcs::Executable;
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
///     let mut result = Executable::from_quil(PROGRAM).with_config(ClientConfiguration::default()).with_shots(4).execute_on_qvm().await.unwrap();
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
#[derive(Debug, Clone)]
pub struct Executable<'executable, 'execution> {
    quil: Arc<str>,
    shots: u16,
    readout_memory_region_names: Option<Vec<Cow<'executable, str>>>,
    params: Parameters,
    compile_with_quilc: bool,
    compiler_options: CompilerOpts,
    config: Option<ClientConfiguration>,
    client: Option<Arc<Qcs>>,
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
    pub fn from_quil<Quil: Into<Arc<str>>>(quil: Quil) -> Self {
        Self {
            quil: quil.into(),
            shots: 1,
            readout_memory_region_names: None,
            params: Parameters::new(),
            compile_with_quilc: true,
            compiler_options: CompilerOpts::default(),
            config: None,
            client: None,
            qpu: None,
            qvm: None,
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
    /// use qcs_api_client_common::ClientConfiguration;
    /// use qcs::Executable;
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
    ///     let mut result = Executable::from_quil(PROGRAM)
    ///         .with_config(ClientConfiguration::default()) // Unnecessary if you have ~/.qcs/settings.toml
    ///         .read_from("first")
    ///         .read_from("second")
    ///         .execute_on_qvm()
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
    /// use qcs_api_client_common::ClientConfiguration;
    /// use qcs::Executable;
    ///
    /// const PROGRAM: &str = "DECLARE theta REAL[2]";
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut exe = Executable::from_quil(PROGRAM)
    ///         .with_config(ClientConfiguration::default()) // Unnecessary if you have ~/.qcs/settings.toml
    ///         .read_from("theta");
    ///
    ///     for theta in 0..2 {
    ///         let theta = theta as f64;
    ///         let mut result = exe
    ///             .with_parameter("theta", 0, theta)
    ///             .with_parameter("theta", 1, theta * 2.0)
    ///             .execute_on_qvm().await.unwrap();
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
    pub fn with_config(mut self, config: ClientConfiguration) -> Self {
        self.config = Some(config);
        self
    }
}

/// The [`Result`] from executing on a QPU or QVM.
pub type ExecutionResult = Result<execution_data::ExecutionData, Error>;

impl Executable<'_, '_> {
    /// Specify a number of times to run the program for each execution. Defaults to 1 run or "shot".
    #[must_use]
    pub fn with_shots(mut self, shots: u16) -> Self {
        self.shots = shots;
        self
    }

    /// If set, the Executable will be compiled using `quilc` prior to compilation on QCS. If not set, the program
    /// is treated as native quil and will not be sent to `quilc`.
    #[must_use]
    pub fn compile_with_quilc(mut self, compile: bool) -> Self {
        self.compile_with_quilc = compile;
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
    pub async fn execute_on_qvm(&mut self) -> ExecutionResult {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            quil = %self.quil,
            num_shots = %self.shots,
            "running Executable on QVM",
        );

        let config = self.get_config().await?;

        let mut qvm = if let Some(qvm) = self.qvm.take() {
            qvm
        } else {
            qvm::Execution::new(&self.quil)?
        };
        let result = qvm
            .run(self.shots, self.get_readouts(), &self.params, &config)
            .await;
        self.qvm = Some(qvm);
        result
            .map_err(Error::from)
            .map(|registers| execution_data::ExecutionData {
                result_data: ResultData::Qvm(registers),
                duration: None,
            })
    }

    async fn get_config(&mut self) -> Result<ClientConfiguration, Error> {
        if let Some(config) = &self.config {
            Ok(config.clone())
        } else {
            let config = ClientConfiguration::load_default().await?;
            self.config = Some(config.clone());
            Ok(config)
        }
    }

    /// Load `self.client` if not yet loaded, then return a reference to it.
    async fn get_client(&mut self) -> Result<Arc<Qcs>, Error> {
        if let Some(client) = &self.client {
            Ok(client.clone())
        } else {
            let config = self.get_config().await?;
            let client = Arc::new(Qcs::with_config(config));
            self.client = Some(client.clone());
            Ok(client)
        }
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
            self.get_client().await?,
            self.compile_with_quilc,
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
    ) -> ExecutionResult
    where
        S: Into<Cow<'execution, str>>,
    {
        let job_handle = self
            .submit_to_qpu_with_endpoint(quantum_processor_id, endpoint_id)
            .await?;
        self.retrieve_results(job_handle).await
    }

    /// Compile the program and execute it on a QCS endpoint, waiting for results.
    ///
    /// # Arguments
    /// 1. `quantum_processor_id`: The name of the QPU to translate the program for on.
    ///     This parameter affects the lifetime of the [`Executable`].
    ///     The [`Executable`] will only live as long as the last parameter passed into this function.
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
    pub async fn execute_on_qpu<S>(&mut self, quantum_processor_id: S) -> ExecutionResult
    where
        S: Into<Cow<'execution, str>>,
    {
        let quantum_processor_id = quantum_processor_id.into();

        #[cfg(feature = "tracing")]
        tracing::debug!(
            quil = %self.quil,
            num_shots = %self.shots,
            %quantum_processor_id,
            "running Executable on QPU",
        );

        let job_handle = self.submit_to_qpu(quantum_processor_id).await?;
        self.retrieve_results(job_handle).await
    }

    /// Compile and submit the program to a QPU, but do not wait for execution to complete.
    ///
    /// Call [`Executable::retrieve_results`] to wait for execution to complete and retrieve the
    /// results.
    ///
    /// # Errors
    ///
    /// See [`Executable::execute_on_qpu`].
    pub async fn submit_to_qpu<S>(
        &mut self,
        quantum_processor_id: S,
    ) -> Result<JobHandle<'execution>, Error>
    where
        S: Into<Cow<'execution, str>>,
    {
        let quantum_processor_id = quantum_processor_id.into();

        #[cfg(feature = "tracing")]
        tracing::debug!(
            quil = %self.quil,
            num_shots = %self.shots,
            %quantum_processor_id,
            "submitting Executable to QPU",
        );

        let job_handle = self
            .qpu_for_id(quantum_processor_id)
            .await?
            .submit(&self.params)
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
    ) -> Result<JobHandle<'execution>, Error>
    where
        S: Into<Cow<'execution, str>>,
    {
        let job_handle = self
            .qpu_for_id(quantum_processor_id)
            .await?
            .submit_to_endpoint_id(&self.params, endpoint_id.into())
            .await?;
        Ok(job_handle)
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
    /// The requested QPU was not found. Either the QPU does not exist or you do not have access to it.
    #[error("The requested QPU was not found.")]
    QpuNotFound,
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
    Quil(#[from] ProgramError<Program>),
    /// There was a problem when compiling the Quil program.
    #[error("There was a problem compiling the Quil program: {0}")]
    Compilation(String),
    /// There was a problem when translating the Quil program.
    #[error("There was a problem translating the Quil program: {0}")]
    Translation(String),
    /// There was a problem when rewriting parameter arithmetic in the Quil program.
    #[error("There was a problem rewriting parameter arithmetic in the Quil program: {0}")]
    RewriteArithmetic(#[from] rewrite_arithmetic::Error),
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
            ExecutionError::IsaError(v) => Self::Unexpected(format!("{v:?}")),
            ExecutionError::ReadoutParse(v) => Self::Unexpected(format!("{v:?}")),
            ExecutionError::Quil(e) => Self::Quil(e),
            ExecutionError::Compilation { details } => Self::Compilation(details),
            ExecutionError::RewriteArithmetic(e) => Self::RewriteArithmetic(e),
            ExecutionError::Substitution(message) => Self::Substitution(message),
        }
    }
}

impl From<qvm::Error> for Error {
    fn from(err: qvm::Error) -> Self {
        match err {
            qvm::Error::QvmCommunication { .. } => Self::Connection(Service::Qvm),
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
}

impl<'a> JobHandle<'a> {
    #[must_use]
    pub(crate) fn new<S>(
        job_id: JobId,
        quantum_processor_id: S,
        endpoint_id: Option<S>,
        readout_map: HashMap<String, String>,
    ) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            job_id,
            quantum_processor_id: quantum_processor_id.into(),
            endpoint_id: endpoint_id.map(Into::into),
            readout_map,
        }
    }

    /// The string representation of the QCS Job ID. Useful for debugging.
    #[must_use]
    pub fn job_id(&self) -> JobId {
        self.job_id.clone()
    }

    /// The execution target of the QCS Job, either the quantum processor or an expicit endpoint.
    #[must_use]
    pub fn job_target(&self) -> JobTarget {
        self.endpoint_id.as_ref().map_or_else(
            || JobTarget::QuantumProcessorId(self.quantum_processor_id.to_string()),
            |endpoint_id| JobTarget::EndpointId(endpoint_id.to_string()),
        )
    }

    /// The readout map from source readout memory locations to the
    /// filter pipeline node which publishes the data.
    #[must_use]
    pub fn readout_map(&self) -> &HashMap<String, String> {
        &self.readout_map
    }
}

#[cfg(test)]
mod describe_get_config {
    use qcs_api_client_openapi::common::ClientConfiguration;

    use crate::Executable;

    #[tokio::test]
    async fn it_resizes_params_dynamically() {
        let mut exe = Executable::from_quil("").with_config(ClientConfiguration::default());
        let foo_len = |exe: &mut Executable<'_, '_>| exe.params.get("foo").unwrap().len();

        exe.with_parameter("foo", 0, 0.0);
        assert_eq!(foo_len(&mut exe), 1);

        exe.with_parameter("foo", 10, 10.0);
        assert_eq!(foo_len(&mut exe), 11);
    }

    #[tokio::test]
    #[cfg(feature = "manual-tests")]
    async fn it_returns_cached_values() {
        let mut exe = Executable::from_quil("");
        let config = ClientConfiguration::builder()
            .set_quilc_url(String::from("test"))
            .build()
            .unwrap();
        exe.config = Some(config.clone());
        let gotten = exe.get_config().await.unwrap_or_default();
        assert_eq!(gotten.quilc_url(), config.quilc_url());
    }
}

#[cfg(test)]
#[cfg(feature = "manual-tests")]
mod describe_qpu_for_id {
    use std::sync::Arc;

    use qcs_api_client_common::ClientConfiguration;

    use crate::compiler::quilc::CompilerOpts;
    use crate::{
        qpu::{self, Qcs},
        Executable,
    };

    #[tokio::test]
    async fn it_refreshes_auth_token() {
        let mut exe = Executable::from_quil("");
        // Default config has no auth, so it should try to refresh
        exe.config = Some(ClientConfiguration::default());
        let result = exe.qpu_for_id("blah").await;
        let Err(err) = result else {
            panic!("Expected an error!");
        };
        let result_string = format!("{err:?}");
        assert!(result_string.contains("refresh_token"));
    }

    #[tokio::test]
    async fn it_loads_cached_version() {
        let mut exe = Executable::from_quil("");
        let shots = 17;
        exe.shots = shots;
        exe.qpu = Some(
            qpu::Execution::new(
                "".into(),
                shots,
                "Aspen-M-3".into(),
                Arc::new(Qcs::with_config(exe.get_config().await.unwrap_or_default())),
                exe.compile_with_quilc,
                CompilerOpts::default(),
            )
            .await
            .unwrap(),
        );
        // Load config with no credentials to prevent creating a new Execution if it tries
        exe.config = Some(ClientConfiguration::default());

        assert!(exe.qpu_for_id("Aspen-M-3").await.is_ok());
    }

    #[tokio::test]
    async fn it_creates_new_after_shot_change() {
        let original_shots = 23;
        let mut exe = Executable::from_quil("").with_shots(original_shots);
        let qpu = exe.qpu_for_id("Aspen-9").await.unwrap();

        assert_eq!(qpu.shots, original_shots);

        // Cache so we can verify cache is not used.
        exe.qpu = Some(qpu);
        let new_shots = 32;
        exe = exe.with_shots(new_shots);
        let qpu = exe.qpu_for_id("Aspen-9").await.unwrap();

        assert_eq!(qpu.shots, new_shots);
    }

    #[tokio::test]
    async fn it_creates_new_for_new_qpu_id() {
        let mut exe = Executable::from_quil("");
        let qpu = exe.qpu_for_id("Aspen-9").await.unwrap();

        assert_eq!(qpu.quantum_processor_id, "Aspen-9");

        // Cache so we can verify cache is not used.
        exe.qpu = Some(qpu);
        // Load config with no credentials to prevent creating the new Execution (which would fail anyway)
        exe.config = Some(ClientConfiguration::default());
        let result = exe.qpu_for_id("Aspen-8").await;

        assert!(matches!(result, Err(_)));
        assert!(matches!(exe.qpu, None));
    }
}
