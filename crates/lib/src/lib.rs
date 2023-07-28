#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)] // This should be enforced by cargo-deny
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::too_many_arguments)]
#![forbid(unsafe_code)]
#![warn(future_incompatible)]
#![warn(rust_2018_compatibility, rust_2018_idioms)]
#![warn(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    bad_style,
    dead_code,
    deprecated_in_future,
    keyword_idents,
    improper_ctypes,
    let_underscore_drop,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    noop_method_call,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    pointer_structural_match,
    private_in_public,
    semicolon_in_expressions_from_macros,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_parens,
    unused_qualifications,
    while_true
)]

//! This crate is the primary Rust API for interacting with Rigetti products. Specifically, this
//! crate allows you to run Quil programs against real QPUs or a QVM
//! using [`Executable`].

pub use executable::{Error, Executable, ExecutionResult, JobHandle, Service};
pub use execution_data::{
    ExecutionData, RegisterMap, RegisterMatrix, RegisterMatrixConversionError, ResultData,
};
pub use register_data::RegisterData;

pub mod client;
pub mod compiler;
pub mod diagnostics;
mod executable;
mod execution_data;
pub mod qpu;
pub mod qvm;
mod register_data;

/// Build information about the crate and environment in which it was built.
pub mod build_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
