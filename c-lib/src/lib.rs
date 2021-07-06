#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// C doesn't have namespaces, so exported functions should contain the module name
#![allow(clippy::module_name_repetitions)]

pub use quantum_processors::free_quantum_processors;
pub use quantum_processors::list_quantum_processors;
pub use qvm::{free_qvm_response, run_program_on_qvm, QVMResponse, QVMStatus};

mod quantum_processors;
mod qvm;
