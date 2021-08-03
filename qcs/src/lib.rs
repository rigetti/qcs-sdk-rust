#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! This crate is the primary Rust API for interacting with Rigetti products. Specifically, this
//! crate allows you to run Quil programs against real QPUs using [`qpu::run_program`] or a QVM
//! using [`qvm::run_program`].

pub use executable::Executable;
pub use execution_result::ExecutionResult;

pub mod configuration;
mod executable;
mod execution_result;
mod qpu;
mod qvm;
