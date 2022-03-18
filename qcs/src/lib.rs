#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)] // This should be enforced by cargo-deny
#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! This crate is the primary Rust API for interacting with Rigetti products. Specifically, this
//! crate allows you to run Quil programs against real QPUs or a QVM
//! using [`Executable`].

pub use executable::{Error, Executable};
pub use execution_result::ExecutionResult;

pub mod configuration;
mod executable;
mod execution_result;
mod qpu;
mod qvm;
