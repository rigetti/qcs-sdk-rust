use enum_as_inner::EnumAsInner;
use ndarray::Array2;
use num::complex::Complex32;
use serde::{Deserialize, Serialize};

/// Data resulting from [`Executable::execute_on_qvm`](`crate::Executable::execute_on_qvm`)
///
/// This represents a single vector (or "register") of typed memory across some number of shots.
/// The register corresponds to the usage of a `DECLARE` instruction in Quil, and the name of that
/// register should be provided with [`Executable::read_from`](`crate::Executable::read_from`).
///
/// There is a variant of this enum for each type of data that a register could hold.
/// Any variant of an instance of `ExecutionResult` will contain a `Vec` with one entry for each shot,
/// where each entry represents the entire register.
///
/// # Usage
///
/// Typically you will already know what type of data the `ExecutionResult` _should_ have, so you can
/// use the [`mod@enum_as_inner`] methods (e.g. [`RegisterData::into_i8`]) in order to
/// convert any variant type to its inner data.
#[derive(Clone, Debug, Deserialize, EnumAsInner, PartialEq, Serialize)]
#[serde(untagged)]
pub enum RegisterData {
    /// Corresponds to the Quil `BIT` or `OCTET` types.
    I8(Array2<i8>),
    /// Corresponds to the Quil `REAL` type.
    F64(Array2<f64>),
    /// Corresponds to the Quil `INTEGER` type.
    I16(Array2<i16>),
    /// Results containing complex numbers.
    #[serde(skip)]
    Complex32(Array2<Complex32>),
}
