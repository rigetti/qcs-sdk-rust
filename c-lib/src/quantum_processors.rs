use std::ffi::CString;
use std::{mem, ptr};

use http::StatusCode;
use libc::c_char;
use thiserror::Error;

use qcs_api::apis::quantum_processors_api;
use qcs_api::{apis, models};
use qcs_util::{get_configuration, ConfigError};

/// Return a comma-separated list of available quantum processors
///
/// # Safety
///
/// In order to safely operate this function:
///
/// 1. The return value of this function __must__ be passed into [`free_quantum_processors`] in
///     order to deallocate the memory.
///
#[no_mangle]
pub unsafe extern "C" fn list_quantum_processors() -> ListQuantumProcessorResponse {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(_) => {
            return ListQuantumProcessorResponse::failure(
                ListQuantumProcessorsResult::CouldNotQueryQCS,
            );
        }
    };
    let response = rt.block_on(get_config_and_list_quantum_processors());
    match response {
        Ok(resp) => resp.into(),
        Err(e) => {
            let result = match e {
                ListError::ApiError(apis::Error::ResponseError(apis::ResponseContent {
                    status: StatusCode::UNAUTHORIZED,
                    content: _,
                    entity: _,
                })) => ListQuantumProcessorsResult::Unauthorized,
                _ => ListQuantumProcessorsResult::CouldNotQueryQCS,
            };
            ListQuantumProcessorResponse::failure(result)
        }
    }
}

/// This function exists to deallocate the memory that was allocated by a call to [`list_quantum_processors`]
///
/// # Safety
///
/// The `response` passed in here must be a valid [`ListQuantumProcessorResponse`] as created by
/// [`list_quantum_processors`].
#[no_mangle]
pub unsafe extern "C" fn free_quantum_processors(response: ListQuantumProcessorResponse) {
    let rust_resp: models::ListQuantumProcessorsResponse = response.into();
    drop(rust_resp);
}

async fn get_config_and_list_quantum_processors(
) -> Result<models::ListQuantumProcessorsResponse, ListError> {
    let configuration = get_configuration().await?;
    Ok(quantum_processors_api::list_quantum_processors(&configuration, None, None).await?)
}

type ApiError = quantum_processors_api::ListQuantumProcessorsError;

#[derive(Error, Debug)]
enum ListError {
    #[error("Error loading config")]
    ConfigError(#[from] ConfigError),
    #[error("Error communicating with")]
    ApiError(#[from] apis::Error<ApiError>),
}

#[repr(u8)]
/// The available result codes from running [`list_quantum_processors`]
pub enum ListQuantumProcessorsResult {
    Success = 0,
    CouldNotQueryQCS = 1,
    Unauthorized = 2,
}

#[repr(C)]
/// Represents the information of a single available processor
pub struct QuantumProcessor {
    /// Unique identifier for a Processor.
    id: *mut c_char,
}

#[repr(C)]
/// The response from [`list_quantum_processors`], contains an array of strings.
pub struct ListQuantumProcessorResponse {
    /// The result code of the function call. Anything other than [`ListQuantumProcessorsResult::Success`]
    /// will result in a null `processors`.
    result: ListQuantumProcessorsResult,
    /// Array of all available processors. This will be NULL if `result` is not Success
    processors: *mut QuantumProcessor,
    /// The length of the array to use for iterating.
    len: usize,
    /// The total capacity of the array in case you'd like to modify it.
    cap: usize,
}

impl ListQuantumProcessorResponse {
    fn failure(result: ListQuantumProcessorsResult) -> Self {
        Self {
            result,
            processors: ptr::null_mut(),
            len: 0,
            cap: 0,
        }
    }
}

impl From<models::ListQuantumProcessorsResponse> for ListQuantumProcessorResponse {
    fn from(resp: models::ListQuantumProcessorsResponse) -> Self {
        let mut processors: Vec<QuantumProcessor> = resp
            .quantum_processors
            .into_iter()
            .filter_map(|proc| CString::new(proc.id).ok())
            .map(|c_string| QuantumProcessor {
                id: c_string.into_raw(),
            })
            .collect();
        processors.shrink_to_fit();

        let len = processors.len();
        let cap = processors.capacity();
        let out_ptr = processors.as_mut_ptr();
        mem::forget(processors);

        Self {
            result: ListQuantumProcessorsResult::Success,
            processors: out_ptr,
            len,
            cap,
        }
    }
}

impl From<ListQuantumProcessorResponse> for models::ListQuantumProcessorsResponse {
    fn from(resp: ListQuantumProcessorResponse) -> Self {
        let ListQuantumProcessorResponse {
            result: _,
            processors: ptr,
            len,
            cap,
        } = resp;

        let processors: Vec<QuantumProcessor> = unsafe { Vec::from_raw_parts(ptr, len, cap) };

        let strings = unsafe {
            processors
                .into_iter()
                .filter_map(|proc| CString::from_raw(proc.id).into_string().ok())
                .map(|id| models::QuantumProcessor { id })
        }
        .collect();
        models::ListQuantumProcessorsResponse {
            next_page_token: None,
            quantum_processors: strings,
        }
    }
}
