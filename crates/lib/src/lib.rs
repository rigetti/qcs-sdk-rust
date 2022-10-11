#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)] // This should be enforced by cargo-deny
#![deny(missing_docs)]
#![allow(clippy::missing_errors_doc)]
#![forbid(unsafe_code)]

//! This crate is the primary Rust API for interacting with Rigetti products. Specifically, this
//! crate allows you to run Quil programs against real QPUs or a QVM
//! using [`Executable`].

pub use executable::{Error, Executable, JobHandle, Service};
pub use execution_data::Qvm;
pub use register_data::RegisterData;

pub mod api;
mod executable;
mod execution_data;
pub mod qpu;
mod qvm;
mod register_data;
