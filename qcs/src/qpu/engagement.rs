use eyre::{Result, WrapErr};

use qcs_api::apis::engagements_api as api;
use qcs_api::models::{CreateEngagementRequest, EngagementWithCredentials};

use crate::configuration::Configuration;

/// Try to get an engagement for a QPU
pub(crate) async fn get(
    quantum_processor_id: Option<String>,
    config: &Configuration,
) -> Result<EngagementWithCredentials> {
    let request = CreateEngagementRequest {
        account_id: None,
        account_type: None,
        endpoint_id: None,
        quantum_processor_id,
        tags: None,
    };
    api::create_engagement(config.as_ref(), request)
        .await
        .wrap_err("While creating an engagement")
}
