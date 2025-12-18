//! Wrappers around [`std::num`]'s `NonZero*` with [`pyo3_stub_gen::PyStubType`] information
//! and more informative Python error messages.

macro_rules! make_nonzero {
    ($name:ident($ty:ty), $num:ty) => {
        /// A non-zero wrapper with [`pyo3_stub_gen::PyStubType`] stubs.
        #[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, ::pyo3::IntoPyObject)]
        pub struct $name(pub $ty);

        // PyO3 has a conversion we could derive from,
        // but it raises a TypeError that says "failed to extract field NonZeroU64.0".
        // By implementing it manually, an invalid value instead reads:
        // "ValueError: expected a positive value".
        impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for $name {
            type Error = ::pyo3::PyErr;

            fn extract(ob: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> Result<Self, Self::Error> {
                ob.extract::<$num>()
                    .ok()
                    .and_then(|value| ::std::convert::TryFrom::try_from(value).ok())
                    .map(Self)
                    .ok_or_else(|| {
                        ::pyo3::exceptions::PyValueError::new_err(concat!(
                            "expected a positive ",
                            stringify!($num)
                        ))
                    })
            }
        }

        #[cfg(feature = "stubs")]
        impl ::pyo3_stub_gen::PyStubType for $name {
            fn type_output() -> ::pyo3_stub_gen::TypeInfo {
                ::pyo3_stub_gen::TypeInfo::builtin("int")
            }
        }
    };
}

make_nonzero!(NonZeroU64(std::num::NonZeroU64), u64);
make_nonzero!(NonZeroU32(std::num::NonZeroU32), u32);
make_nonzero!(NonZeroU16(std::num::NonZeroU16), u16);
make_nonzero!(NonZeroU8(std::num::NonZeroU8), u8);
make_nonzero!(NonZeroUsize(std::num::NonZeroUsize), usize);

make_nonzero!(NonZeroI64(std::num::NonZeroI64), i64);
make_nonzero!(NonZeroI32(std::num::NonZeroI32), i32);
make_nonzero!(NonZeroI16(std::num::NonZeroI16), i16);
make_nonzero!(NonZeroI8(std::num::NonZeroI8), i8);
