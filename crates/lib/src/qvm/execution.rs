use std::str::FromStr;
use std::{collections::HashMap, num::NonZeroU16};

use quil_rs::Program;

use crate::{executable::Parameters, qvm::run_program};

use super::{http::AddressRequest, Error, QvmResultData};
use super::{Client, QvmOptions};

/// Contains all the info needed to execute on a QVM a single time, with the ability to be reused for
/// faster subsequent runs.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Execution {
    program: Program,
}

impl Execution {
    /// Construct a new [`Execution`] from Quil. Immediately parses the Quil and returns an error if
    /// there are any problems.
    pub(crate) fn new(quil: &str) -> Result<Self, Error> {
        let program = Program::from_str(quil).map_err(Error::Parsing)?;
        Ok(Self { program })
    }

    /// Run on a QVM.
    ///
    /// QVM must be available at `config.qvm_url`.
    ///
    /// # Arguments
    ///
    /// 1. `shots`: The number of times the program should run.
    /// 2. `addresses`: A mapping of memory region names to an [`AddressRequest`] describing what
    ///    values should be returned for that address.
    /// 3. `register`: The name of the register containing results that should be read out from QVM.
    /// 4. `params`: Values to substitute for parameters in Quil.
    /// 5. `config`: A configuration object containing the connection URL of QVM.
    ///
    /// Returns: [`ExecutionResult`].
    ///
    /// # Errors
    ///
    /// All errors are returned in a human-readable format using `eyre` since usually they aren't
    /// recoverable at runtime and should just be logged for handling manually.
    ///
    /// ## QVM Connection Errors
    ///
    /// QVM must be running and accessible for this function to succeed. The address can be defined by
    /// the `<profile>.applications.pyquil.qvm_url` setting in your QCS `settings.toml`. More info on
    /// configuration in [`crate::configuration`].
    ///
    /// ## Parameter Errors
    ///
    /// Missing parameters, extra parameters, or parameters of the wrong type will all cause errors.
    pub(crate) async fn run<C: Client + ?Sized>(
        &self,
        shots: NonZeroU16,
        addresses: HashMap<String, AddressRequest>,
        params: &Parameters,
        client: &C,
    ) -> Result<QvmResultData, Error> {
        run_program(
            &self.program,
            shots,
            addresses,
            params,
            None,
            None,
            None,
            client,
            &QvmOptions::default(),
        )
        .await
    }
}

#[cfg(test)]
mod describe_execution {
    use std::{collections::HashMap, num::NonZeroU16};

    use super::{Execution, Parameters};
    use crate::{client::Qcs, qvm};

    async fn qvm_client() -> qvm::http::HttpClient {
        let qcs = Qcs::load().await;
        qvm::http::HttpClient::from(&qcs)
    }

    #[tokio::test]
    async fn it_errs_on_excess_parameters() {
        let exe = Execution::new("DECLARE ro BIT").unwrap();

        let mut params = Parameters::new();
        params.insert("doesnt_exist".into(), vec![0.0]);

        let result = exe
            .run(
                NonZeroU16::new(1).expect("value is non-zero"),
                HashMap::new(),
                &params,
                &qvm_client().await,
            )
            .await;
        if let Err(e) = result {
            assert!(e.to_string().contains("doesnt_exist"));
        } else {
            panic!("Expected an error but got none.");
        }
    }

    #[tokio::test]
    async fn it_errors_when_any_param_is_the_wrong_size() {
        let exe = Execution::new("DECLARE ro BIT[2]").unwrap();

        let mut params = Parameters::new();
        params.insert("ro".into(), vec![0.0]);

        let result = exe
            .run(
                NonZeroU16::new(1).expect("value is non-zero"),
                HashMap::new(),
                &params,
                &qvm_client().await,
            )
            .await;
        if let Err(e) = result {
            let err_string = e.to_string();
            assert!(err_string.contains("ro"));
            assert!(err_string.contains('1'));
            assert!(err_string.contains('2'));
        } else {
            panic!("Expected an error but got none.");
        }
    }
}
