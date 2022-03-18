use std::time::Duration;

use eyre::{eyre, Report, Result, WrapErr};
use reqwest::StatusCode;

use qcs_api::apis::configuration::Configuration as ApiConfig;
use qcs_api::models::{CreateEngagementRequest, EngagementWithCredentials, Error as ErrorPayload};

use crate::configuration::Configuration;

/// Try to get an engagement for a QPU
pub(super) async fn get(
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
    let mut local_var_req_builder =
        local_var_client.request(reqwest::Method::POST, local_var_uri_str.as_str());

    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder =
            local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }
    if let Some(ref local_var_token) = local_var_configuration.bearer_access_token {
        local_var_req_builder = local_var_req_builder.bearer_auth(local_var_token.clone());
    };
    local_var_req_builder = local_var_req_builder.json(&create_engagement_request);

    let local_var_req = local_var_req_builder
        .build()
        .wrap_err("Failed to build request")?;
    let local_var_resp = local_var_client
        .execute(local_var_req)
        .await
        .wrap_err("Failed to send request to QCS")?;

    let local_var_status = local_var_resp.status();

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        local_var_resp
            .json::<EngagementWithCredentials>()
            .await
            .wrap_err("Response is not a valid JSON")
            .map_err(Error::from)
    } else if local_var_status == StatusCode::SERVICE_UNAVAILABLE {
        let seconds = local_var_resp
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .ok_or_else(|| eyre!("Service is unavailable but no Retry-After is specified"))?
            .to_str()
            .wrap_err("Failed to parse Retry-After header")?
            .parse::<u64>()
            .wrap_err("Failed to parse Retry-After header")?;
        Err(Error::QuantumProcessorUnavailable(Duration::from_secs(
            seconds,
        )))
    } else {
        let local_var_entity: ErrorPayload =
            local_var_resp.json().await.wrap_err("Cannot parse error")?;
        Err(Error::Authorization(local_var_entity))
    }
}

#[derive(thiserror::Error, Debug)]
pub(super) enum Error {
    #[error("the QPU is unavailable, try again after {}", .0.as_secs())]
    QuantumProcessorUnavailable(Duration),
    #[error("error gathering QPU authorization")]
    Authorization(ErrorPayload),
    #[error(transparent)]
    General(#[from] Report),
}
