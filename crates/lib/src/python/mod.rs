//! Python bindings for the qcs-sdk crate.
//!
//! While this package can be used directly,
//! [PyQuil](https://pypi.org/project/pyquil/) offers more functionality
//! and a higher-level interface for building and executing Quil programs.

use std::sync::OnceLock;

use numpy::Complex32;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList, PyTuple, PyType, PyTypeMethods},
    wrap_pymodule,
};

#[cfg(feature = "stubs")]
use pyo3_stub_gen::{
    define_stub_info_gatherer,
    derive::{gen_stub_pyfunction, gen_stub_pymethods},
};

use crate::{
    client::Qcs,
    compiler,
    python::{
        executable::{ExeParameter, PyExecutable, PyJobHandle},
        execution_data::PyRegisterMatrix,
    },
    qpu::{self, QpuResultData},
    qvm::{self, QvmResultData},
    ExecutionData, RegisterData, RegisterMap, ResultData, Service,
};

pub(crate) mod client;
pub(crate) mod errors;
pub(crate) mod executable;
pub(crate) mod execution_data;
pub(crate) mod nonzero;
pub(crate) mod register_data;

pub(crate) use nonzero::NonZeroU16;

static PY_RESET_LOGGING_HANDLE: OnceLock<pyo3_log::ResetHandle> = OnceLock::new();

#[pymodule]
#[pyo3(name = "_qcs_sdk")]
fn init_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    match pyo3_log::try_init() {
        Ok(reset_handle) => {
            // Ignore the error if the handle is already set.
            let _ = PY_RESET_LOGGING_HANDLE.set(reset_handle);
        }
        Err(e) => eprintln!("Failed to initialize the qcs_sdk logger: {e}"),
    }

    let py = m.py();

    m.add("QcsSdkError", py.get_type::<errors::QcsSdkError>())?;
    m.add("ExecutionError", py.get_type::<errors::ExecutionError>())?;
    m.add(
        "RegisterMatrixConversionError",
        py.get_type::<errors::RegisterMatrixConversionError>(),
    )?;

    m.add_class::<ExecutionData>()?;
    m.add_class::<ResultData>()?;
    m.add_class::<RegisterMap>()?;
    m.add_class::<PyRegisterMatrix>()?;
    m.add_class::<PyExecutable>()?;
    m.add_class::<ExeParameter>()?;
    m.add_class::<PyJobHandle>()?;
    m.add_class::<Service>()?;
    m.add_class::<RegisterData>()?;
    m.add_class::<Qcs>()?;

    m.add_function(wrap_pyfunction!(reset_logging, m)?)?;
    m.add_function(wrap_pyfunction!(gather_diagnostics, m)?)?;

    m.add_wrapped(wrap_pymodule!(client::init_module))?;
    m.add_wrapped(wrap_pymodule!(compiler::python::init_module))?;
    m.add_wrapped(wrap_pymodule!(qpu::python::init_module))?;
    m.add_wrapped(wrap_pymodule!(qvm::python::init_module))?;
    client::init_module(m)?;
    compiler::python::init_module(m)?;
    qpu::python::init_module(m)?;
    qvm::python::init_module(m)?;

    // Fix __qualname__ for complex enums so they can be pickled
    fix_complex_enums!(
        py,
        RegisterData,
        ResultData,
        qpu::ReadoutValues,
        qpu::result_data::MemoryValues,
        qpu::api::ConnectionStrategy,
        qvm::http::AddressRequest
    );

    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    let sys = PyModule::import(py, "sys")?;
    let sys_modules: Bound<'_, PyDict> = sys.getattr("modules")?.cast_into()?;

    pyo3_tracing_subscriber::add_submodule("qcs_sdk", "_tracing_subscriber", py, m)?;
    // let submodule = m.getattr("_tracing_subscriber")?;
    // sys_modules.set_item("qcs_sdk._tracing_subscriber.subscriber", submodule.getattr("subscriber")?)?;
    // sys_modules.set_item("qcs_sdk._tracing_subscriber.layers", submodule.getattr("layers")?)?;
    // sys_modules.set_item("qcs_sdk._tracing_subscriber.common", submodule.getattr("common")?)?;
    // sys_modules.set_item("qcs_sdk._tracing_subscriber", submodule)?;

    let submodule = m.getattr("client")?;
    sys_modules.set_item("qcs_sdk.client", submodule)?;

    let submodule = m.getattr("compiler")?;
    sys_modules.set_item("qcs_sdk.compiler.quilc", submodule.getattr("quilc")?)?;
    sys_modules.set_item("qcs_sdk.compiler", submodule)?;

    let submodule = m.getattr("qpu")?;
    sys_modules.set_item("qcs_sdk.qpu.api", submodule.getattr("api")?)?;
    sys_modules.set_item("qcs_sdk.qpu.isa", submodule.getattr("isa")?)?;
    sys_modules.set_item("qcs_sdk.qpu.translation", submodule.getattr("translation")?)?;
    {
        let submodule = submodule.getattr("experimental")?;
        sys_modules.set_item(
            "qcs_sdk.qpu.experimental.random",
            submodule.getattr("random")?,
        )?;
        sys_modules.set_item("qcs_sdk.qpu.experimental", submodule)?;
    }
    sys_modules.set_item("qcs_sdk.qpu", submodule)?;

    let submodule = m.getattr("qvm")?;
    sys_modules.set_item("qcs_sdk.qvm.api", submodule.getattr("api")?)?;
    sys_modules.set_item("qcs_sdk.qvm", submodule)?;

    Ok(())
}

/// Fix the `__qualname__` on PyO3's "complex enums" so that they can be pickled.
///
/// Essentially, this runs the following Python code:
///
/// ```python
/// import inspect
/// issubclass = lambda cls: inspect.isclass(cls) and issubclass(cls, typ)
/// for name, cls in inspect.getmembers(typ, issubclass):
///     cls.__qualname__ = f"{prefix}.{name}"
/// ```
///
/// # In a Pickle
///
/// PyO3 processes `enum`s with non-unit variants by creating a Python class for the enum,
/// then creating a class for each variant, subclassed from the main enum class.
/// The subclasses end up as attributes on the main enum class,
/// which enables syntax like `q = Qubit.Fixed(0)`;
/// however, they're given qualified names that use `_` as a seperator instead of `.`,
/// e.g. we get `Qubit.Fixed(0).__qualname__ == "Qubit_Fixed"`
/// rather than `Qubit.Fixed`, as we would if we had written the inner class ourselves.
/// As a consequence, attempting to `pickle` an instance of it
/// will raise an error complaining that `quil.instructions.Qubit_Fixed` can't be found.
///
/// There are a handful of ways of making this work,
/// but modifying the `__qualname__` seems not only simple, but correct.
///
/// # See Also
///
/// - PyO3's Complex Enums: https://pyo3.rs/v0.25.1/class#complex-enums
/// - Issue regarding `__qualname__`: https://github.com/PyO3/pyo3/issues/5270
/// - Python's `inspect`: https://docs.python.org/3/library/inspect.html#inspect.getmembers
pub(crate) fn fix_enum_qual_names<'py>(typ: &Bound<'py, PyType>) -> PyResult<()> {
    let py = typ.py();
    let inspect = PyModule::import(py, "inspect")?;
    let isclass = inspect.getattr("isclass")?;
    let get_members = inspect.getattr("getmembers")?;

    let prefix = typ.qualname()?;
    let prefix = prefix.as_borrowed();
    let prefix = prefix.to_str()?;

    let inner: Bound<'_, PyList> = get_members.call((typ, isclass), None)?.cast_into()?;
    for item in &inner {
        let item = item.cast::<PyTuple>()?;

        let cls = item.get_borrowed_item(1)?;
        if cls.cast()?.is_subclass(typ)? {
            // See https://pyo3.rs/v0.25.1/types#borroweda-py-t for info on `get_borrowed_item`.
            let name = item.get_borrowed_item(0)?;
            let fixed_name = format!("{prefix}.{}", name.cast()?.to_str()?);
            cls.setattr(pyo3::intern!(py, "__qualname__"), fixed_name)?;
        }
    }

    Ok(())
}

/// Fix the `__qualname__` on a list of complex enums so that they can be pickled.
/// See [`fix_enum_qual_names`] for more information.
///
/// The first argument should be a `Python<'py>` instance;
/// all others should be names of `#[pyclass]`-annotated `enum`s with non-unit variants.
///
/// # Example
///
/// ```ignore
/// use pyo3;
/// use pyo3_stub_gen::derive::gen_stub_pyclass_complex_enum;
///
/// #[pyo3::pymodule(name = "place", module = "some", submodule)]
/// fn init_some_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
///   let py = m.py();
///
///   m.add_class::<Foo>()?;
///   m.add_class::<Bar>()?;
///
///   fix_complex_enums!(py, Foo, Bar);
/// }
///
/// #[gen_stub_pyclass_complex_enum]
/// #[pyo3::pyclass(module = "some.place", eq, frozen, hash, get_all)]
/// pub enum Foo {
///     Integer(i64),
///     Real(f64),
/// }
///
/// #[gen_stub_pyclass_complex_enum]
/// #[pyo3::pyclass(module = "some.place", eq, frozen, hash, get_all)]
/// pub enum Bar {
///     Integer(i64),
///     Real(f64),
/// }
/// ```
macro_rules! fix_complex_enums {
    ($py:expr, $($name:path),* $(,)?) => {
        {
            let py = $py;
            $($crate::python::fix_enum_qual_names(&py.get_type::<$name>())?;)*
        }
    };
}

/// Add a `__repr__` method that returns the Rust type's `Debug` string.
macro_rules! impl_repr {
    ($name: ident) => {
        #[cfg_attr(feature = "stubs", pyo3_stub_gen::derive::gen_stub_pymethods)]
        #[pyo3::pymethods]
        impl $name {
            fn __repr__(&self) -> String {
                format!("{self:?}")
            }
        }
    };
}

/// Spawn and block on a future using the pyo3 tokio runtime.
/// Useful for returning a synchronous `PyResult`.
///
///
/// When used like the following:
/// ```rs
/// async fn say_hello(name: String) -> String {
///     format!("hello {name}")
/// }
///
/// #[pyo3(name="say_hello")]
/// pub fn py_say_hello(name: String) -> PyResult<String> {
///     py_sync!(say_hello(name))
/// }
/// ```
///
/// Becomes the associated "synchronous" python call:
/// ```py
/// assert say_hello("Rigetti") == "hello Rigetti"
/// ```
macro_rules! py_sync {
    ($py: ident, $body: expr) => {{
        $py.detach(|| {
            let runtime = ::pyo3_async_runtimes::tokio::get_runtime();
            let handle = runtime.spawn($body);

            runtime.block_on(async {
                tokio::select! {
                    result = handle => result.map_err(|err| ::pyo3::exceptions::PyRuntimeError::new_err(err.to_string()))?,
                    signal_err = async {
                        // A 100ms loop delay is a bit arbitrary, but seems to
                        // balance CPU usage and SIGINT responsiveness well enough.
                        let delay = ::std::time::Duration::from_millis(100);
                        loop {
                            ::pyo3::Python::attach(|py| {
                                py.check_signals()
                            })?;
                            ::tokio::time::sleep(delay).await;
                        }
                    } => signal_err,
                }
            })
        })
    }};
}

/// This is a reimplementation of [`rigetti_pyo3::py_function_sync_async`] that ensures
/// Opentelemetry contexts are propagated. This could be replaced with the [`rigetti_pyo3`]
/// implementation when https://github.com/rigetti/rigetti-pyo3/issues/59 is resolved (i.e.
/// this would allow us to use [`rigetti_pyo3::py_function_sync_async`] on sync functions
/// that return a future).
macro_rules! py_function_sync_async {
    (
        $(#[$meta: meta])+
        $pub:vis async fn $name:ident($($(#[$arg_meta: meta])*$arg: ident : $kind: ty),* $(,)?) $(-> $ret: ty)? $body: block
    ) => {
        ::paste::paste! {
        async fn [< $name _impl >]($($arg: $kind,)*) $(-> $ret)? {
            $body
        }

        $(#[$meta])+
        #[allow(clippy::too_many_arguments)]
        #[pyo3(name = $name "")]
        $pub fn [< py_ $name >](py: ::pyo3::Python<'_> $(, $(#[$arg_meta])*$arg: $kind)*) $(-> $ret)? {
            use opentelemetry::trace::FutureExt;
            $crate::python::py_sync!(py, [< $name _impl >]($($arg),*).with_current_context())
        }

        $(#[$meta])+
        #[pyo3(name = $name "_async")]
        #[allow(clippy::too_many_arguments)]
        $pub fn [< py_ $name _async >]<'py>(py: ::pyo3::Python<'py> $(, $(#[$arg_meta])*$arg: $kind)*) -> ::pyo3::PyResult<Bound<'py, ::pyo3::PyAny>> {
            use opentelemetry::trace::FutureExt;
            ::pyo3_async_runtimes::tokio::future_into_py(
                py, [< $name _impl >]($($arg),*).with_current_context())
        }
        }
    };
}

pub(crate) use fix_complex_enums;
pub(crate) use impl_repr;
pub(crate) use py_function_sync_async;
pub(crate) use py_sync;

#[cfg(feature = "stubs")]
define_stub_info_gatherer!(stub_info);

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk"))]
#[pyfunction]
fn reset_logging() {
    if let Some(handle) = PY_RESET_LOGGING_HANDLE.get() {
        handle.reset();
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pyfunction(module = "qcs_sdk"))]
#[pyfunction]
#[pyo3(name = "_gather_diagnostics")]
fn gather_diagnostics(py: Python<'_>) -> PyResult<String> {
    py_sync!(py, async { Ok(crate::diagnostics::get_report().await) })
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl RegisterData {
    #[new]
    fn __new__(values: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(values) = values.extract::<Vec<Vec<i8>>>() {
            Ok(Self::I8(values))
        } else if let Ok(values) = values.extract::<Vec<Vec<f64>>>() {
            Ok(Self::F64(values))
        } else if let Ok(values) = values.extract::<Vec<Vec<i16>>>() {
            Ok(Self::I16(values))
        } else if let Ok(values) = values.extract::<Vec<Vec<Complex32>>>() {
            Ok(Self::Complex32(values))
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "expected a list of lists of integers, reals, or complex numbers",
            ))
        }
    }
}

#[cfg_attr(not(feature = "stubs"), optipy::strip_pyo3(only_stubs))]
#[cfg_attr(feature = "stubs", gen_stub_pymethods)]
#[pymethods]
impl ResultData {
    #[new]
    fn __new__(values: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(data) = values.extract::<QvmResultData>() {
            Ok(Self::Qvm(data))
        } else if let Ok(data) = values.extract::<QpuResultData>() {
            Ok(Self::Qpu(data))
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "expected QVM or QPU result data",
            ))
        }
    }
}
