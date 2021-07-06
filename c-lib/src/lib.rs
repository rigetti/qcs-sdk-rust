#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// C doesn't have namespaces, so exported functions should contain the module name
#![allow(clippy::module_name_repetitions)]

pub use quantum_processors::free_quantum_processors;
pub use quantum_processors::list_quantum_processors;

mod quantum_processors;

//
// #[repr(u8)]
// pub enum GetInstructionSetArchitectureResponse {
//     Success = 0,
//     InvalidUTF8 = 1,
//     CouldNotQueryQCS = 2,
// }
//
// #[repr(C)]
// pub struct InstructionSetArchitecture {}
//
// /// Given the string of a `quantum_processor_id` (one of the values retrieved from
// /// `list_quantum_processors`), return the instruction set architecture.
// ///
// /// ## SAFETY
// /// In order to safely operate this function, the following must be guaranteed by the caller:
// /// 1. quantum_processor_id is a valid, null-terminated string which is less than isize::MAX in length.
// /// 2. The quantum_processor_id pointer will remain valid for the duration of this function.
// /// 3. The memory backing quantum_processor_id will not change for the duration of this function.
// #[no_mangle]
// extern "C" fn get_instruction_set_architecture(
//     quantum_processor_id: *mut c_char,
// ) -> GetInstructionSetArchitectureResponse {
//     let configuration = get_config();
//     let rt = match tokio::runtime::Runtime::new() {
//         Ok(runtime) => runtime,
//         Err(_) => return GetInstructionSetArchitectureResponse::CouldNotQueryQCS,
//     };
//     // SAFETY: There is no guarantee of the validity/lifetime of the pointer provided, so this
//     // will always be unsafe. It's up to the caller to prevent memory issues here.
//     let c_str = unsafe { CStr::from_ptr(quantum_processor_id) };
//     let quantum_processor_id = match c_str.to_str() {
//         Ok(val) => val,
//         Err(_) => return GetInstructionSetArchitectureResponse::InvalidUTF8,
//     };
//
//     let fut = qcs_api::apis::quantum_processors_api::get_instruction_set_architecture(
//         &configuration,
//         quantum_processor_id,
//     );
//     let response = match rt.block_on(fut) {
//         Ok(resp) => resp,
//         Err(_) => return GetInstructionSetArchitectureResponse::CouldNotQueryQCS,
//     };
// }
