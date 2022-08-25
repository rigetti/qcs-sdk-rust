use std::time::Duration;

use reqwest::StatusCode;

use qcs_api::apis::configuration::Configuration as ApiConfig;
use qcs_api::models::{CreateEngagementRequest, EngagementWithCredentials, Error as ErrorPayload};

use crate::configuration::Configuration;

/// Try to get an engagement for a QPU
pub(crate) async fn get(
    quantum_processor_id: String,
    config: &Configuration,
) -> Result<EngagementWithCredentials, Error> {
    let request = CreateEngagementRequest {
        account_id: None,
        account_type: None,
        endpoint_id: None,
        quantum_processor_id: Some(quantum_processor_id),
        tags: None,
    };
    create_engagement(config.as_ref(), request).await
}

/// Create a new engagement using the specified parameters.
///
/// # Manual Implementation
///
/// This function is derived from [`qcs-api`] but implemented manually because of limitations in
/// that generator. Specifically, they provide no access to headers.
async fn create_engagement(
    configuration: &ApiConfig,
    create_engagement_request: CreateEngagementRequest,
) -> Result<EngagementWithCredentials, Error> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/v1/engagements", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.post(local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.clone());
    };
    local_var_req_builder = local_var_req_builder.json(&create_engagement_request);

    let local_var_req = local_var_req_builder.build().map_err(Error::Internal)?;
    let local_var_resp = local_var_client
        .execute(local_var_req)
        .await
        .map_err(Error::Connection)?;

    let local_var_status = local_var_resp.status();

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        local_var_resp
            .json::<EngagementWithCredentials>()
            .await
            .map_err(Error::Schema)
    } else if local_var_status == StatusCode::SERVICE_UNAVAILABLE {
        let retry_after = local_var_resp
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map_or(DEFAULT_RETRY_AFTER, Duration::from_secs);
        Err(Error::QuantumProcessorUnavailable(retry_after))
    } else if local_var_status == StatusCode::UNAUTHORIZED
        || local_var_status == StatusCode::FORBIDDEN
    {
        Err(Error::Unauthorized)
    } else {
        let local_var_entity: ErrorPayload = local_var_resp.json().await.map_err(Error::Schema)?;
        Err(Error::Unknown(local_var_entity))
    }
}

const DEFAULT_RETRY_AFTER: Duration = Duration::from_secs(15 /* minutes */ * 60);

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("the QPU is unavailable, try again after {} seconds", .0.as_secs())]
    QuantumProcessorUnavailable(Duration),
    #[error("Received an unauthorized response, try refreshing the token")]
    Unauthorized,
    #[error("Could not understand a response from QCS, this is likely a bug in this library")]
    Schema(#[source] reqwest::Error),
    #[error("Received an unknown error from QCS: {}", .0.message)]
    Unknown(ErrorPayload),
    #[error("A bug in this library prevented the request from being sent")]
    Internal(#[source] reqwest::Error),
    #[error("Trouble connecting to QCS, check your network connection")]
    Connection(#[source] reqwest::Error),
}
