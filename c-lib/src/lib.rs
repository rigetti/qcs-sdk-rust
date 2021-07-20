#![deny(clippy::all)]
#![deny(clippy::pedantic)]
// C doesn't have namespaces, so exported functions should contain the module name
#![allow(clippy::module_name_repetitions)]

pub use crate::qvm::{free_qvm_response, run_program_on_qvm, QVMResponse};

mod qvm;
