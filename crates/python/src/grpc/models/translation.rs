use qcs_api_client_grpc::services::translation::{
    translation_options::TranslationBackend, BackendV1Options, BackendV2Options, TranslationOptions,
};
use rigetti_pyo3::{py_wrap_data_struct, py_wrap_type, py_wrap_union_enum, create_init_submodule};

py_wrap_type! {
    #[derive(Default)]
    PyBackendV1Options(BackendV1Options) as "BackendV1Options";
}

py_wrap_type! {
    #[derive(Default)]
    PyBackendV2Options(BackendV2Options) as "BackendV2Options";
}

py_wrap_union_enum! {
    PyTranslationBackend(TranslationBackend) as "TranslationBackend" {
        v1: V1 => PyBackendV1Options,
        v2: V2 => PyBackendV2Options
    }
}

py_wrap_data_struct! {
    #[derive(Default)]
    PyTranslationOptions(TranslationOptions) as "TranslationOptions" {
        translation_backend: Option<TranslationBackend> => Option<PyTranslationBackend>
    }
}

create_init_submodule! {
    classes: [
        PyTranslationBackend,
        PyTranslationOptions,
        PyBackendV1Options,
        PyBackendV2Options
    ],
}