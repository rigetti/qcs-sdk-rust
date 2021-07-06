/*
 * Rigetti QCS API
 *
 * # Introduction  This is the documentation for the Rigetti QCS HTTP API.  You can find out more about Rigetti at [https://rigetti.com](https://rigetti.com), and also interact with QCS via the web at [https://qcs.rigetti.com](https://qcs.rigetti.com).  This API is documented in **OpenAPI format** and so is compatible with the dozens of language-specific client generators available [here](https://github.com/OpenAPITools/openapi-generator) and elsewhere on the web.  # Principles  This API follows REST design principles where appropriate, and otherwise an HTTP RPC paradigm. We adhere to the Google [API Improvement Proposals](https://google.aip.dev/general) where reasonable to provide a consistent, intuitive developer experience. HTTP response codes match their specifications, and error messages fit a common format.  # Authentication  All access to the QCS API requires OAuth2 authentication provided by Okta. You can request access [here](https://www.rigetti.com/get-quantum). Once you have a user account, you can download your access token from QCS [here](https://qcs.rigetti.com/auth/token).   That access token is valid for 24 hours after issuance. The value of `access_token` within the JSON file is the token used for authentication (don't use the entire JSON file).  Authenticate requests using the `Authorization` header and a `Bearer` prefix:  ``` curl --header \"Authorization: Bearer eyJraW...Iow\" ```  # Quantum Processor Access  Access to the quantum processors themselves is not yet provided directly by this HTTP API, but is instead performed over ZeroMQ/[rpcq](https://gitlab.com/rigetti/rpcq). Until that changes, we suggest using [pyquil](https://gitlab.com/rigetti/pyquil) to build and execute quantum programs via the Legacy API.  # Legacy API  Our legacy HTTP API remains accessible at https://forest-server.qcs.rigetti.com, and it shares a source of truth with this API's services. You can use either service with the same user account and means of authentication. We strongly recommend using the API documented here, as the legacy API is on the path to deprecation.
 *
 * The version of the OpenAPI document: 2020-07-31
 * Contact: support@rigetti.com
 * Generated by: https://openapi-generator.tech
 */

/// Characteristic : A measured characteristic of an operation.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Characteristic {
    /// The error in the characteristic value, or None otherwise.
    #[serde(rename = "error", skip_serializing_if = "Option::is_none")]
    pub error: Option<f32>,
    /// The name of the characteristic.
    #[serde(rename = "name")]
    pub name: String,
    /// The list of architecture node ids for the site where the characteristic is measured, if that is different from the site of the enclosing operation. None if it is the same. The order of this or the enclosing node ids obey the definition of node symmetry from the enclosing operation.
    #[serde(rename = "node_ids", skip_serializing_if = "Option::is_none")]
    pub node_ids: Option<Vec<i32>>,
    /// The optional ordered list of parameter values used to generate the characteristic. The order matches the parameters in the enclosing operation, and so the lengths of these two lists must match.
    #[serde(rename = "parameter_values", skip_serializing_if = "Option::is_none")]
    pub parameter_values: Option<Vec<f32>>,
    /// The date and time at which the characteristic was measured.
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    /// The characteristic value measured.
    #[serde(rename = "value")]
    pub value: f32,
}

impl Characteristic {
    /// A measured characteristic of an operation.
    pub fn new(name: String, timestamp: String, value: f32) -> Characteristic {
        Characteristic {
            error: None,
            name,
            node_ids: None,
            parameter_values: None,
            timestamp,
            value,
        }
    }
}
