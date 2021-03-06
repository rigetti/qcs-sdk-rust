/*
 * Rigetti QCS API
 *
 * # Introduction  This is the documentation for the Rigetti QCS HTTP API.  You can find out more about Rigetti at [https://rigetti.com](https://rigetti.com), and also interact with QCS via the web at [https://qcs.rigetti.com](https://qcs.rigetti.com).  This API is documented in **OpenAPI format** and so is compatible with the dozens of language-specific client generators available [here](https://github.com/OpenAPITools/openapi-generator) and elsewhere on the web.  # Principles  This API follows REST design principles where appropriate, and otherwise an HTTP RPC paradigm. We adhere to the Google [API Improvement Proposals](https://google.aip.dev/general) where reasonable to provide a consistent, intuitive developer experience. HTTP response codes match their specifications, and error messages fit a common format.  # Authentication  All access to the QCS API requires OAuth2 authentication provided by Okta. You can request access [here](https://www.rigetti.com/get-quantum). Once you have a user account, you can download your access token from QCS [here](https://qcs.rigetti.com/auth/token).   That access token is valid for 24 hours after issuance. The value of `access_token` within the JSON file is the token used for authentication (don't use the entire JSON file).  Authenticate requests using the `Authorization` header and a `Bearer` prefix:  ``` curl --header \"Authorization: Bearer eyJraW...Iow\" ```  # Quantum Processor Access  Access to the quantum processors themselves is not yet provided directly by this HTTP API, but is instead performed over ZeroMQ/[rpcq](https://gitlab.com/rigetti/rpcq). Until that changes, we suggest using [pyquil](https://gitlab.com/rigetti/pyquil) to build and execute quantum programs via the Legacy API.  # Legacy API  Our legacy HTTP API remains accessible at https://forest-server.qcs.rigetti.com, and it shares a source of truth with this API's services. You can use either service with the same user account and means of authentication. We strongly recommend using the API documented here, as the legacy API is on the path to deprecation.
 *
 * The version of the OpenAPI document: 2020-07-31
 * Contact: support@rigetti.com
 * Generated by: https://openapi-generator.tech
 */

/// Endpoint : An Endpoint is the entry point for remote access to a QuantumProcessor.

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Endpoint {
    /// Network address at which the endpoint is locally reachable
    #[serde(rename = "address")]
    pub address: String,
    /// Whether the endpoint is operating as intended
    #[serde(rename = "healthy")]
    pub healthy: bool,
    /// Unique, opaque identifier for the endpoint
    #[serde(rename = "id")]
    pub id: String,
    /// Whether the endpoint serves simulated or substituted data for testing purposes
    #[serde(rename = "mock")]
    pub mock: bool,
    /// Public identifier for a quantum processor [example: Aspen-1]
    #[serde(rename = "quantumProcessorId")]
    pub quantum_processor_id: String,
}

impl Endpoint {
    /// An Endpoint is the entry point for remote access to a QuantumProcessor.
    pub fn new(
        address: String,
        healthy: bool,
        id: String,
        mock: bool,
        quantum_processor_id: String,
    ) -> Endpoint {
        Endpoint {
            address,
            healthy,
            id,
            mock,
            quantum_processor_id,
        }
    }
}
