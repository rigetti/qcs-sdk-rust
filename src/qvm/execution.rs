use std::collections::HashMap;
use std::str::FromStr;

use quil_rs::{
    instruction::{ArithmeticOperand, Instruction, MemoryReference, Move},
    Program,
};

use crate::configuration::Configuration;
use crate::executable::Parameters;
use crate::RegisterData;

use super::{Request, Response};

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
    /// 2. `register`: The name of the register containing results that should be read out from QVM.
    /// 3. `params`: Values to substitute for parameters in Quil.
    /// 4. `config`: A configuration object containing the connection URL of QVM.
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
    pub(crate) async fn run(
        &mut self,
        shots: u16,
        readouts: &[&str],
        params: &Parameters,
        config: &Configuration,
    ) -> Result<HashMap<Box<str>, RegisterData>, Error> {
        if shots == 0 {
            return Err(Error::ShotsMustBePositive);
        }

        let memory = &self.program.memory_regions;

        let mut instruction_count = 0;
        for (name, values) in params {
            match memory.get(name.as_ref()) {
                Some(region) => {
                    if region.size.length != values.len() as u64 {
                        return Err(Error::RegionSizeMismatch {
                            name: name.clone(),
                            declared: region.size.length,
                            parameters: values.len(),
                        });
                    }
                }
                None => {
                    return Err(Error::RegionNotFound { name: name.clone() });
                }
            }
            for (index, value) in values.iter().enumerate() {
                self.program.instructions.insert(
                    0,
                    Instruction::Move(Move {
                        destination: ArithmeticOperand::MemoryReference(MemoryReference {
                            name: name.to_string(),
                            index: index as u64,
                        }),
                        source: ArithmeticOperand::LiteralReal(*value),
                    }),
                );
                instruction_count += 1;
            }
        }
        let result = self.execute(shots, readouts, config).await;
        for _ in 0..instruction_count {
            self.program.instructions.remove(0);
        }
        result
    }

    async fn execute(
        &self,
        shots: u16,
        readouts: &[&str],
        config: &Configuration,
    ) -> Result<HashMap<Box<str>, RegisterData>, Error> {
        let request = Request::new(&self.program.to_string(true), shots, readouts);

        let client = reqwest::Client::new();
        let response = client
            .post(&config.qvm_url)
            .json(&request)
            .send()
            .await
            .map_err(|source| Error::QvmCommunication {
                qvm_url: config.qvm_url.clone(),
                source,
            })?;

        match response.json::<Response>().await {
            Err(source) => Err(Error::QvmCommunication {
                qvm_url: config.qvm_url.clone(),
                source,
            }),
            Ok(Response::Success(response)) => Ok(response
                .registers
                .into_iter()
                .map(|(key, value)| (key.into_boxed_str(), value))
                .collect()),
            Ok(Response::Failure(response)) => Err(Error::Qvm {
                message: response.status,
            }),
        }
    }
}

/// All of the errors that can occur when running a Quil program on QVM.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Error parsing Quil program: {0}")]
    Parsing(String),
    #[error("Shots must be a positive integer.")]
    ShotsMustBePositive,
    #[error("Declared region {name} has size {declared} but parameters have size {parameters}.")]
    RegionSizeMismatch {
        name: Box<str>,
        declared: u64,
        parameters: usize,
    },
    #[error("Could not find region {name} for parameter. Are you missing a DECLARE instruction?")]
    RegionNotFound { name: Box<str> },
    #[error("Could not communicate with QVM at {qvm_url}")]
    QvmCommunication {
        qvm_url: String,
        source: reqwest::Error,
    },
    #[error("QVM reported a problem running your program: {message}")]
    Qvm { message: String },
}

#[cfg(test)]
mod describe_execution {
    use super::{Configuration, Execution, Parameters};

    #[tokio::test]
    async fn it_errs_on_excess_parameters() {
        let mut exe = Execution::new("DECLARE ro BIT").unwrap();

        let mut params = Parameters::new();
        params.insert("doesnt_exist".into(), vec![0.0]);

        let result = exe.run(1, &[], &params, &Configuration::default()).await;
        if let Err(e) = result {
            assert!(e.to_string().contains("doesnt_exist"));
        } else {
            panic!("Expected an error but got none.");
        }
    }

    #[tokio::test]
    async fn it_errors_when_any_param_is_the_wrong_size() {
        let mut exe = Execution::new("DECLARE ro BIT[2]").unwrap();

        let mut params = Parameters::new();
        params.insert("ro".into(), vec![0.0]);

        let result = exe.run(1, &[], &params, &Configuration::default()).await;
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
