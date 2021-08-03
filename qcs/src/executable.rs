use std::collections::HashMap;

use eyre::{eyre, Result, WrapErr};

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
///     // Here we indicate to `qcs` that the `"ro"` register contains the data we'd like in our `ExecutionResult`
///     let result: ExecutionResult = Executable::from_quil(PROGRAM).with_shots(4).execute_on_qvm().await.unwrap();
///     // We know it's i8 because we declared the memory as `BIT` in Quil.
///     let data = result.into_i8().unwrap();
///     // In this case, we ran the program for 4 shots, so we know the length is 4.
///     assert_eq!(data.len(), 4);
///     for shot in data {
///         // Each shot will contain all the memory, in order, for the vector (or "register") we
///         // requested the results of. In this case, "ro".
///         assert_eq!(shot.len(), 2);
///         // In the case of this particular program, we know ro[0] should equal ro[1]
///         assert_eq!(shot[0], shot[1]);
///     }
/// }
///
/// ```
pub struct Executable<'executable, 'execution> {
    quil: &'executable str,
    shots: u16,
    register: &'executable str,
    params: Parameters<'executable>,
    config: Option<Configuration>,
    qpu: Option<qpu::Execution<'execution>>,
    qvm: Option<qvm::Execution>,
}

pub(crate) type ParamName<'a> = &'a str;
pub(crate) type Parameters<'a> = HashMap<ParamName<'a>, Vec<f64>>;

impl<'executable> Executable<'executable, '_> {
    /// Create an `Executable` from a string containing a  [quil](https://github.com/quil-lang/quil)
    /// program.
    #[must_use]
    pub fn from_quil(quil: &'executable str) -> Self {
        Self {
            quil,
            shots: 1,
            register: "ro",
            params: Parameters::new(),
            config: None,
            qpu: None,
            qvm: None,
        }
    }

    /// Specify the memory region or "register" to return results from. This must correspond to a
    /// `DECLARE` statement in the provided Quil program.
    #[must_use]
    pub fn read_from(mut self, register: &'executable str) -> Self {
        self.register = register;
        self
    }

    /// Sets a concrete value for [parametric compilation].
    ///
    /// [parametric compilation]: https://pyquil-docs.rigetti.com/en/stable/basics.html?highlight=parametric#parametric-compilation
    ///
    /// # Errors
    ///
    /// 1. Quil could not be parsed to search for parameters.
    /// 2. `param_name` was not declared in the Quil program (i.e. missing `DECLARE` statement or misspelled name)
    /// 3. Tried to set an incorrect number of values for the allocated memory (e.g. `DECLARE mem BIT[2]` but passed `vec![0.0]`.
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

impl Executable<'_, '_> {
    /// Specify a number of times to run the program in this execution. Defaults to 1 run or "shot".
    #[must_use]
    pub fn with_shots(mut self, shots: u16) -> Self {
        self.shots = shots;
        self
    }

    /// Execute on a QVM which must be available at `config.qvm_url`.
    ///
    /// # Warning
    ///
    /// This function is `async` because of the HTTP client under the hood, but it will block your
    /// thread waiting on the RPCQ-based functions.
    ///
    /// # Returns
    ///
    /// [`ExecutionResult`].
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
    /// configuration in [`crate::configuration`].
    ///
    /// ## Execution Errors
    ///
    /// A number of errors could occur if `program` is malformed.
    pub async fn execute_on_qvm(&mut self) -> Result<ExecutionResult> {
        let config = self.take_or_load_config().await;
        let mut qvm = if let Some(qvm) = self.qvm.take() {
            qvm
        } else {
            qvm::Execution::new(self.quil)?
        };
        let result = qvm
            .run(self.shots, self.register, &self.params, &config)
            .await;
        self.qvm = Some(qvm);
        self.config = Some(config);
        result
    }

    async fn take_or_load_config(&mut self) -> Configuration {
        if let Some(config) = self.config.take() {
            config
        } else {
            Configuration::load()
                .await
                .unwrap_or_else(|_| Configuration::default())
        }
    }
}

impl<'execution> Executable<'_, 'execution> {
    async fn qpu_for_id(&mut self, id: &'execution str) -> Result<qpu::Execution<'execution>> {
        if let Some(qpu) = self.qpu.take() {
            if qpu.quantum_processor_id == id && qpu.shots == self.shots {
                return Ok(qpu);
            }
        }
        let mut config = self.take_or_load_config().await;
        let result = match qpu::Execution::new(self.quil, self.shots, id, &config).await {
            Ok(result) => Ok(result),
            Err(qpu::Error::Qcs(_)) => {
                config = config
                    .refresh()
                    .await
                    .wrap_err("When refreshing authentication token")?;
                qpu::Execution::new(self.quil, self.shots, id, &config).await
            }
            err => err,
        };
        self.config = Some(config);
        result.wrap_err_with(|| eyre!("When executing on {}", id))
    }

    /// Execute on a real QPU
    ///
    /// # Arguments
    /// 1. `quantum_processor_id`: The name of the QPU to run on.
    ///
    /// # Warning
    ///
    /// This function is `async` because of the HTTP client under the hood, but it will block your
    /// thread waiting on the RPCQ-based functions.
    ///
    /// # Returns
    ///
    /// [`ExecutionResult`].
    ///
    /// # Errors
    /// All errors are human readable by way of [`mod@eyre`]. Some common errors are:
    ///
    /// 1. You are not authenticated for QCS
    /// 1. Your credentials don't have an active reservation for the QPU you requested
    /// 1. [quilc] was not running.
    ///
    /// [quilc]: https://github.com/quil-lang/quilc
    pub async fn execute_on_qpu(
        &mut self,
        quantum_processor_id: &'execution str,
    ) -> Result<ExecutionResult> {
        let mut qpu = self.qpu_for_id(quantum_processor_id).await?;
        let mut config = self.take_or_load_config().await;

        let response = match qpu.run(&self.params, self.register, &config).await {
            Ok(result) => Ok(result),
            Err(qpu::Error::Qcs(_)) => {
                config = config
                    .refresh()
                    .await
                    .wrap_err("When refreshing authentication token")?;
                qpu.run(&self.params, self.register, &config).await
            }
            err => err,
        };

        self.qpu = Some(qpu);
        response.wrap_err_with(|| eyre!("When executing on {}", quantum_processor_id))
    }
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
            qpu::Execution::new("", shots, "Aspen-9", &exe.take_or_load_config().await)
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
