//! This module contains all the functionality for running Quil programs on a real QPU. Specifically,
//! the [`Execution`] struct in this module.
use std::time::Duration;

use crate::client::{OpenApiClientError, Qcs, DEFAULT_HTTP_API_TIMEOUT};
use qcs_api_client_openapi::{
    apis::{
        quantum_processors_api::{
            self, get_instruction_set_architecture, GetInstructionSetArchitectureError,
        },
        Error as OpenApiError,
    },
    models::InstructionSetArchitecture,
};
use tokio::time::error::Elapsed;

pub mod api;
mod execution;
pub mod result_data;
pub mod translation;

pub(crate) use execution::{Error as ExecutionError, Execution};
#[allow(clippy::module_name_repetitions)]
pub use result_data::{QpuResultData, ReadoutValues};

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
) -> Result<InstructionSetArchitecture, GetIsaError> {
    #[cfg(feature = "tracing")]
    tracing::debug!(
        "getting instruction set architecture for {}",
        quantum_processor_id
    );

    get_instruction_set_architecture(&client.get_openapi_client(), quantum_processor_id)
        .await
        .map_err(OpenApiClientError::RequestFailed)
}

/// Error raised due to failure to get an ISA
pub type GetIsaError = OpenApiClientError<GetInstructionSetArchitectureError>;

/// API Errors encountered when trying to list available quantum processors.
#[derive(Debug, thiserror::Error)]
pub enum ListQuantumProcessorsError {
    /// Failed the http call
    #[error("Failed to list processors via API: {0}")]
    ApiError(#[from] OpenApiError<quantum_processors_api::ListQuantumProcessorsError>),

    /// Pagination did not finish before timeout
    #[error("API pagination did not finish before timeout.")]
    TimeoutError(#[from] Elapsed),
}

/// Query the QCS API for the names of all available quantum processors.
/// If `None`, the default `timeout` used is 10 seconds.
pub async fn list_quantum_processors(
    client: &Qcs,
    timeout: Option<Duration>,
) -> Result<Vec<String>, ListQuantumProcessorsError> {
    #[cfg(feature = "tracing")]
    tracing::debug!("listing quantum processors");

    let timeout = timeout.unwrap_or(DEFAULT_HTTP_API_TIMEOUT);

    tokio::time::timeout(timeout, async move {
        let mut quantum_processors = vec![];
        let mut page_token = None;

        loop {
            let result = quantum_processors_api::list_quantum_processors(
                &client.get_openapi_client(),
                Some(100),
                page_token.as_deref(),
            )
            .await?;

            let mut data = result
                .quantum_processors
                .into_iter()
                .map(|qpu| qpu.id)
                .collect::<Vec<_>>();
            quantum_processors.append(&mut data);

            page_token = result.next_page_token;
            if page_token.is_none() {
                break;
            }
        }

        Ok(quantum_processors)
    })
    .await?
}
