//! This module contains the public-facing API for executing programs. [`Executable`] is the how
//! users will interact with QCS, quilc, and QVM.

use std::collections::HashMap;
use std::time::Duration;

use eyre::{eyre, Report, Result, WrapErr};

use crate::configuration::Configuration;
use crate::{qpu, qvm, ExecutionResult};

/// The builder interface for executing Quil programs on QVMs and QPUs.
///
/// # Example
///
/// ```rust
/// use qcs::{Executable, ExecutionResult};
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
///     let mut result = Executable::from_quil(PROGRAM).with_shots(4).execute_on_qvm().await.unwrap();
///     // We know it's i8 because we declared the memory as `BIT` in Quil.
///     // "ro" is the only source read from by default if you don't specify a .read_from()
///     let data = result.remove("ro").expect("Did not receive ro data").into_i8().unwrap();
///     // In this case, we ran the program for 4 shots, so we know the length is 4.
///     assert_eq!(data.len(), 4);
///     for shot in data {
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
pub struct Executable<'executable, 'execution> {
    quil: &'executable str,
    shots: u16,
    readout_memory_region_names: Option<Vec<&'executable str>>,
    params: Parameters<'executable>,
    skip_quilc: bool,
    config: Option<Configuration>,
    qpu: Option<qpu::Execution<'execution>>,
    qvm: Option<qvm::Execution>,
}

pub(crate) type Parameters<'a> = HashMap<&'a str, Vec<f64>>;

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
    pub fn from_quil(quil: &'executable str) -> Self {
        Self {
            quil,
            shots: 1,
            readout_memory_region_names: None,
            params: Parameters::new(),
            skip_quilc: false,
            config: None,
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
    ///         .read_from("first")
    ///         .read_from("second")
    ///         .execute_on_qvm()
    ///         .await
    ///         .unwrap();
    ///     let first = result
    ///         .remove("first")
    ///         .expect("Did not receive first buffer")
    ///         .into_f64()
    ///         .expect("Received incorrect data type for first");
    ///     let second = result
    ///         .remove("second")
    ///         .expect("Did not receive second buffer")
    ///         .into_f64()
    ///         .expect("Received incorrect data type for second");
    ///     assert_eq!(first[0][0], 3.141);
    ///     assert_eq!(second[0][0], 1.234);
    /// }
    /// ```
    #[must_use]
    pub fn read_from(mut self, register: &'executable str) -> Self {
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
    /// use qcs::Executable;
    ///
    /// const PROGRAM: &str = "DECLARE theta REAL[2]";
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut exe = Executable::from_quil(PROGRAM)
    ///         .read_from("theta");
    ///     
    ///     for theta in 0..2 {
    ///         let theta = theta as f64;
    ///         let mut result = exe
    ///             .with_parameter("theta", 0, theta)
    ///             .with_parameter("theta", 1, theta * 2.0)
    ///             .execute_on_qvm().await.unwrap();
    ///         let data = result.remove("theta").expect("Could not read theta").into_f64().unwrap();
    ///         assert_eq!(data[0][0], theta);
    ///         assert_eq!(data[0][1], theta * 2.0);
    ///     }
    /// }
    /// ```
    ///
    /// [parametric compilation]: https://pyquil-docs.rigetti.com/en/stable/basics.html?highlight=parametric#parametric-compilation
    pub fn with_parameter(
        &mut self,
        param_name: &'executable str,
        index: usize,
        value: f64,
    ) -> &mut Self {
        let values = if let Some(values) = self.params.get_mut(param_name) {
            values
        } else {
            self.params.insert(param_name, vec![0.0; index]);
            if let Some(values) = self.params.get_mut(param_name) {
                values
            } else {
                unreachable!("Set in the line above")
            }
        };

        if index + 1 > values.len() {
            values.resize(index + 1, 0.0);
        }

        values[index] = value;

        self
    }
}

type ExecuteResult = Result<HashMap<Box<str>, ExecutionResult>, Error>;

impl Executable<'_, '_> {
    /// Specify a number of times to run the program for each execution. Defaults to 1 run or "shot".
    #[must_use]
    pub fn with_shots(mut self, shots: u16) -> Self {
        self.shots = shots;
        self
    }

    /// If set, the Executable is assumed to be native quil and wil skip calls to quilc.
    #[must_use]
    pub fn skip_quilc(mut self) -> Self {
        self.skip_quilc = true;
        self
    }

    fn get_readouts(&self) -> &[&str] {
        return self
            .readout_memory_region_names
            .as_ref()
            .map_or(&["ro"], Vec::as_slice);
    }

    /// Execute on a QVM which must be available at the configured URL (default <http://localhost:5000>).
    ///
    /// # Warning
    ///
    /// This function is `async` because of the HTTP client under the hood, but it will block your
    /// thread waiting on the RPCQ-based functions.
    ///
    /// # Returns
    ///
    /// A `HashMap<String, ExecutionResult>` where the key is the name of the register that was read from (e.g. "ro").
    ///
    /// # Errors
    ///
    /// All errors are returned in a human-readable format using [`mod@eyre`] since usually they aren't
    /// recoverable at runtime and should just be logged for handling manually.
    ///
    /// ## QVM Connection Errors
    ///
    /// QVM must be running and accessible for this function to succeed. The address can be defined by
    /// the `<profile>.applications.pyquil.qvm_url` setting in your QCS `settings.toml`. More info on
    /// configuration [here][`crate::configuration`].
    ///
    /// ## Execution Errors
    ///
    /// A number of errors could occur if `program` is malformed.
    pub async fn execute_on_qvm(&mut self) -> ExecuteResult {
        let config = self.take_or_load_config().await;
        let mut qvm = if let Some(qvm) = self.qvm.take() {
            qvm
        } else {
            qvm::Execution::new(self.quil)?
        };
        let result = qvm
            .run(self.shots, self.get_readouts(), &self.params, &config)
            .await;
        self.qvm = Some(qvm);
        self.config = Some(config);
        result.map_err(Error::from)
    }

    /// Remove and return `self.config` if set. Otherwise, load it from disk.
    async fn take_or_load_config(&mut self) -> Configuration {
        if let Some(config) = self.config.take() {
            config
        } else {
            Configuration::load().await.unwrap_or_else(|e| {
                log::error!("Got an error when loading config: {:#?}", e);
                Configuration::default()
            })
        }
    }
}

impl<'execution> Executable<'_, 'execution> {
    /// Remove and return `self.qpu` if it's set and still valid. Otherwise, create a new one.
    async fn qpu_for_id(&mut self, id: &'execution str) -> Result<qpu::Execution<'execution>> {
        if let Some(qpu) = self.qpu.take() {
            if qpu.quantum_processor_id == id && qpu.shots == self.shots {
                return Ok(qpu);
            }
        }
        let mut config = self.take_or_load_config().await;
        let result =
            match qpu::Execution::new(self.quil, self.shots, id, &config, self.skip_quilc).await {
                Ok(result) => Ok(result),
                Err(qpu::Error::Qcs { .. }) => {
                    config = config
                        .refresh()
                        .await
                        .wrap_err("When refreshing authentication token")?;
                    qpu::Execution::new(self.quil, self.shots, id, &config, self.skip_quilc).await
                }
                err => err,
            };
        self.config = Some(config);
        result.wrap_err_with(|| eyre!("When executing on {}", id))
    }

    /// Execute on a real QPU
    ///
    /// # Arguments
    /// 1. `quantum_processor_id`: The name of the QPU to run on. This parameter affects the
    ///     lifetime of the [`Executable`]. The [`Executable`] will only live as long as the last
    ///     parameter passed into this function.
    ///
    /// # Warning
    ///
    /// This function is `async` because of the HTTP client under the hood, but it will block your
    /// thread waiting on the RPCQ-based services.
    ///
    /// # Returns
    ///
    /// A `HashMap<String, ExecutionResult>` where the key is the name of the register that was read from (e.g. "ro").
    ///
    /// # Errors
    /// All errors are human readable by way of [`mod@eyre`]. Some common errors are:
    ///
    /// 1. You are not authenticated for QCS
    /// 1. Your credentials don't have an active reservation for the QPU you requested
    /// 1. [quilc] was not running.
    /// 1. The `quil` that this [`Executable`] was constructed with was invalid.
    /// 1. Missing parameters that should be filled with [`Executable::with_parameter`]
    ///
    /// [quilc]: https://github.com/quil-lang/quilc
    pub async fn execute_on_qpu(&mut self, quantum_processor_id: &'execution str) -> ExecuteResult {
        let mut qpu = self.qpu_for_id(quantum_processor_id).await?;
        let mut config = self.take_or_load_config().await;

        let readouts = self.get_readouts();

        let response = match qpu.run(&self.params, readouts, &config).await {
            Ok(result) => Ok(result),
            Err(qpu::Error::Qcs {
                retry_after: None, ..
            }) => {
                // If retry_after is set, don't retry now
                config = config
                    .refresh()
                    .await
                    .wrap_err("When refreshing authentication token")?;
                qpu.run(&self.params, readouts, &config).await
            }
            err => err,
        };

        self.qpu = Some(qpu);
        match response {
            Ok(result) => Ok(result),
            Err(qpu::Error::Qcs {
                source,
                retry_after,
            }) => {
                match retry_after {
                    Some(duration) => {
                        // retry_after could mean maintenance which requires recompile
                        self.qpu = None;
                        Err(Error::Retry {
                            source,
                            after: duration,
                        })
                    }
                    None => Err(source.into()),
                }
            }
            Err(quil_err @ qpu::Error::Quil { .. }) => Err(Error::from(
                eyre!(quil_err).wrap_err("When compiling for the QPU"),
            )),
        }
    }
}

/// The possible errors which can be returned by [`Executable::execute_on_qpu`] and
/// [`Executable::execute_on_qvm`]..
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error that is due to a temporary problem and should be retried after `after` [`Duration`].
    #[error("An error that is due to a temporary problem and should be retried.")]
    Retry {
        /// The error itself
        source: Report,
        /// The [`Duration`] to wait before retrying
        after: Duration,
    },
    /// An error which is due to a permanent problem and should not be retried.
    #[error("A fatal error that should not be retried.")]
    Fatal(#[from] Report),
}

#[cfg(test)]
#[cfg(feature = "manual-tests")]
mod describe_qpu_for_id {
    use super::*;

    #[tokio::test]
    async fn it_refreshes_auth_token() {
        let mut exe = Executable::from_quil("");
        // Default config has no auth, so it should try to refresh
        exe.config = Some(Configuration::default());
        let result = exe.qpu_for_id("blah").await;
        let err = if let Err(err) = result {
            err
        } else {
            panic!("Expected an error!");
        };
        let result_string = format!("{:?}", err);
        assert!(result_string.contains("refresh token"))
    }

    #[tokio::test]
    async fn it_loads_cached_version() {
        let mut exe = Executable::from_quil("");
        let shots = 17;
        exe.shots = shots;
        exe.qpu = Some(
            qpu::Execution::new(
                "",
                shots,
                "Aspen-9",
                &exe.take_or_load_config().await,
                exe.skip_quilc,
            )
            .await
            .unwrap(),
        );
        // Load config with no credentials to prevent creating a new Execution if it tries
        exe.config = Some(Configuration::default());

        assert!(exe.qpu_for_id("Aspen-9").await.is_ok());
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
        exe.config = Some(Configuration::default());
        let result = exe.qpu_for_id("Aspen-8").await;

        assert!(matches!(result, Err(_)));
        assert!(matches!(exe.qpu, None));
    }
}

#[cfg(test)]
#[cfg(feature = "manual-tests")]
mod describe_take_or_load_config {
    use super::*;

    #[tokio::test]
    async fn it_returns_cached_values() {
        let mut exe = Executable::from_quil("");
        let mut config = Configuration::default();
        config.quilc_url = String::from("test");
        exe.config = Some(config.clone());
        let gotten = exe.take_or_load_config().await;
        assert_eq!(gotten.quilc_url, config.quilc_url);
        assert!(matches!(exe.config, None));
    }
}
