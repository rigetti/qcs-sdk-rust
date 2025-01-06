//! This module contains a definition for the [`ExternedCall`] trait.
//! Implementations of this trait represent externed functions that
//! the QCS backend supports.
//!
//! Trait implementations support semantically meaningful interfaces
//! that may be converted into Quil [`quil_rs::instruction::Call`]
//! instructions as well as their associated `PRAGMA EXTERN`
//! instructions.
use std::convert::TryInto;

use quil_rs::quil::ToQuilError;

/// A trait for supporting `PRAGMA EXTERN` and [`quil_rs::instruction::Call`] instructions.
pub trait ExternedCall: Sized + TryInto<quil_rs::instruction::Call> {
    /// An error that may occur when building the signature.
    type Error: From<ToQuilError>;

    /// The name of the externed function.
    const NAME: &'static str;

    /// Build the function signature.
    fn build_signature(
    ) -> Result<quil_rs::instruction::ExternSignature, <Self as ExternedCall>::Error>;

    /// Build a `PRAGMA EXTERN` instruction for the externed function.
    fn pragma_extern() -> Result<quil_rs::instruction::Pragma, <Self as ExternedCall>::Error> {
        use quil_rs::quil::Quil;

        Ok(quil_rs::instruction::Pragma::new(
            quil_rs::instruction::RESERVED_PRAGMA_EXTERN.to_string(),
            vec![quil_rs::instruction::PragmaArgument::Identifier(
                Self::NAME.to_string(),
            )],
            Some(
                Self::build_signature()?
                    .to_quil()
                    .map_err(<Self as ExternedCall>::Error::from)?,
            ),
        ))
    }
}
