//! This module contains all the functionality for running Quil programs on a real QPU. Specifically,
//! the [`Execution`] struct in this module.

use self::client::OpenApiClientError;
use qcs_api_client_openapi::{
    apis::quantum_processors_api::{
        get_instruction_set_architecture, GetInstructionSetArchitectureError,
    },
    models::InstructionSetArchitecture,
};

pub mod client;
mod execution;
pub mod quilc;
pub(crate) mod rewrite_arithmetic;
pub(crate) mod rpcq;
pub(crate) mod runner;
pub(crate) mod translation;

pub use client::Qcs;
pub(crate) use execution::{Error as ExecutionError, Execution};

/// Query QCS for the ISA of the provided `quantum_processor_id`.
///
/// # Errors
///
/// 1. Problem communicating with QCS
/// 2. Unauthenticated
/// 3. Expired token
pub async fn get_isa(
    quantum_processor_id: &str,
    client: &Qcs,
) -> Result<InstructionSetArchitecture, IsaError> {
    get_instruction_set_architecture(&client.get_openapi_client(), quantum_processor_id)
        .await
        .map_err(OpenApiClientError::RequestFailed)
}

/// Error raised due to failure to get an ISA
pub type IsaError = OpenApiClientError<GetInstructionSetArchitectureError>;
