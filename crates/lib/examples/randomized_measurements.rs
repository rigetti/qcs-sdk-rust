//! This example demonstrates how to construct and then execute a program with
//! randomized measurements. Specifically, we will use a ZXZXZ unitary
//! decomposition to randomly draw from a
//! [tetrahedral unitary ensemble](https://en.wikipedia.org/wiki/Tetrahedral_symmetry).
use std::num::NonZeroU16;
use std::str::FromStr;

use itertools::Itertools;
use qcs::qpu::experimental::random::PrngSeedValue;
use qcs::qpu::experimental::randomized_measurements::{
    QubitRandomization, RandomizedMeasurements, UnitarySet,
};
use qcs::{qpu::api::ExecutionOptions, Executable};
use quil_rs::instruction::Measurement;
use quil_rs::quil::Quil;
use quil_rs::Program;

include!("../src/qpu/experimental/unitary_set_zxzxz.rs");

const BASE_PROGRAM: &str = r#"
DECLARE ro BIT[3]
RX(pi/2) 0
RX(pi/2) 1
RX(pi/2) 2
"#;

const NUM_SHOTS: u16 = 5_000;

const SEED_VALUES: [(u64, u64); 3] = [(0, 555_571_734), (1, 467_091_842), (2, 925_313_021)];

const LEADING_DELAY_SECONDS: f64 = 1e-5;

const DEFAULT_QUANTUM_PROCESSOR_ID: &str = "Ankaa-3";

fn quantum_processor_id() -> String {
    std::env::var("QUANTUM_PROCESSOR_ID")
        .unwrap_or_else(|_| DEFAULT_QUANTUM_PROCESSOR_ID.to_string())
}

#[tokio::main]
async fn main() {
    let program =
        Program::from_str(BASE_PROGRAM).expect("BASE_PROGRAM should be a valid Quil program");
    let measurements = (0..2)
        .map(|q| {
            Measurement::new(
                Qubit::Fixed(q),
                Some(MemoryReference::new("ro".to_string(), q)),
            )
        })
        .collect();
    let unitary_set =
        ZxzxzUnitarySet::tetrahedral().expect("tetrahedral unitary set should be valid");

    let leading_delay = Expression::Number(Complex64::new(LEADING_DELAY_SECONDS, 0.0));
    let randomized_measurements =
        RandomizedMeasurements::try_new(measurements, unitary_set, leading_delay)
            .expect("RandomizedMeasurements should be successfully created");
    let program_with_random_measurements = randomized_measurements
        .append_to_program(program)
        .expect("Program should be successfully appended with randomized measurements");
    let seed_values = SEED_VALUES
        .iter()
        .copied()
        .map(|(qubit, seed_value)| PrngSeedValue::try_new(seed_value).map(|seed| (qubit, seed)))
        .try_collect()
        .expect("prng seeds must be valid");
    let parameters = randomized_measurements
        .to_parameters(&seed_values)
        .expect("parameters should be successfully created");

    let mut exe = Executable::from_quil(
        program_with_random_measurements
            .to_quil()
            .expect("Program should be successfully converted to Quil"),
    )
    .with_shots(
        NonZeroU16::new(NUM_SHOTS)
            .unwrap_or_else(|| panic!("{} should be a valid number of shots", NUM_SHOTS)),
    );
    parameters.into_iter().for_each(|(name, values)| {
        values.into_iter().enumerate().for_each(|(index, value)| {
            exe.with_parameter(name.clone(), index, value);
        });
    });

    let job_handle = exe
        .submit_to_qpu(quantum_processor_id(), None, &ExecutionOptions::default())
        .await
        .expect("Program should be successfully submitted for execution");
    let _data = exe
        .retrieve_results(job_handle)
        .await
        .expect("Results should be successfully retrieved");
    println!("program constructed, translated, and executed successfully!");
}
