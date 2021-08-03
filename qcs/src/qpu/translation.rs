use eyre::{Result, WrapErr};

use qcs_api::apis::translation_api as translation;
use qcs_api::models::{
    TranslateNativeQuilToEncryptedBinaryRequest, TranslateNativeQuilToEncryptedBinaryResponse,
};

use crate::configuration::Configuration;

use crate::qpu::rewrite_arithmetic::RewrittenQuil;

pub(crate) async fn translate(
    quil: RewrittenQuil,
    shots: u16,
    quantum_processor_id: &str,
    config: &Configuration,
) -> Result<TranslateNativeQuilToEncryptedBinaryResponse> {
    let translation_request = TranslateNativeQuilToEncryptedBinaryRequest {
        num_shots: shots.into(),
        quil: quil.into(),
        settings_timestamp: None,
    };
    translation::translate_native_quil_to_encrypted_binary(
        config.as_ref(),
        quantum_processor_id,
        translation_request,
    )
    .await
    .wrap_err("While translating native quil to binary")
}
