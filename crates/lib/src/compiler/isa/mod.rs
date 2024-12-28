use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;

use itertools::{Either, Itertools};
use serde::{Deserialize, Serialize};

use edge::{convert_edges, Edge, Id};
use qcs_api_client_openapi::models::InstructionSetArchitecture;
use qubit::{FrbSim1q, Qubit};

mod edge;
mod operator;
mod qubit;

/// Restructuring of an [`InstructionSetArchitecture`] for sending to quilc
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct Compiler {
    #[serde(rename = "1Q")]
    qubits: HashMap<String, Qubit>,
    #[serde(rename = "2Q")]
    edges: HashMap<String, Edge>,
}

impl TryFrom<InstructionSetArchitecture> for Compiler {
    type Error = Error;

    fn try_from(isa: InstructionSetArchitecture) -> Result<Self, Error> {
        let architecture = isa.architecture;
        let mut qubits = Qubit::from_nodes(&architecture.nodes);

        let mut edges = convert_edges(&architecture.edges)?;

        let site_ops = isa
            .instructions
            .iter()
            .flat_map(|op| op.sites.iter().map(move |site| (op, site)));
        let frb_sim_1q = FrbSim1q::try_from(isa.benchmarks)?;

        for (op, site) in site_ops {
            match (&op.node_count, &site.node_ids.len()) {
                (Some(1), 1) => {
                    let id = &site.node_ids[0];
                    let qubit = qubits
                        .get_mut(id)
                        .ok_or_else(|| Error::QubitDoesNotExist(String::from(&op.name), *id))?;
                    qubit.add_operation(&op.name, &site.characteristics, &frb_sim_1q)?;
                }
                (Some(2), 2) => {
                    let id = Id::try_from(&site.node_ids)?;
                    let edge = edges
                        .get_mut(&id)
                        .ok_or_else(|| Error::EdgeDoesNotExist(String::from(&op.name), id))?;
                    edge.add_operation(&op.name, &site.characteristics)?;
                }
                (node_count, node_ids) => {
                    return Err(Error::IncorrectNodes(
                        (*node_count, *node_ids),
                        String::from(&op.name),
                        site.node_ids.clone(),
                    ))
                }
            }
        }

        let (qubits, dead_qubits): (_, HashSet<_>) = qubits
            .into_iter()
            .partition_map(|(id, q)| {
                if q.has_valid_operations() {
                    Either::Left((id.to_string(), q))
                } else {
                    Either::Right(id)
                }
            });
        let edges = edges
            .into_iter()
            .filter(|(_, e)| e.has_valid_operations() && !e.qubits().any(|q| dead_qubits.contains(q)))
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        Ok(Self { qubits, edges })
    }
}

/// All the errors that can occur from within this module
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Operation {0} is defined for Qubit {1} but that Qubit does not exist")]
    QubitDoesNotExist(String, i64),
    #[error("Operation {0} is defined for Edge {1} but that Edge does not exist")]
    EdgeDoesNotExist(String, Id),
    #[error(
        "The number of nodes for an operation and site_operation must be (1, 1) or (2, 2). \
                        Got {0:?} while parsing operation {1} at site {2:?}"
    )]
    IncorrectNodes((Option<i64>, usize), String, Vec<i64>),
    #[error(transparent)]
    Qubit(#[from] qubit::Error),
    #[error(transparent)]
    Edge(#[from] edge::Error),
}

#[cfg(test)]
mod describe_compiler_isa {
    use std::{convert::TryFrom, fs::{self, read_to_string}};

    use float_cmp::{approx_eq, F64Margin};
    use qcs_api_client_openapi::models::{InstructionSetArchitecture, Node};
    use serde_json::Value;

    use super::Compiler;

    /// Compare two JSON values and make sure they are equivalent while allowing for some precision
    /// loss in numbers.
    ///
    /// Panics if there is any inequality.
    fn assert_json_is_equivalent<'a>(
        expected: &'a Value,
        actual: &'a Value,
    ) {
        assert_json_is_equivalent_inner(expected, actual, "");
    }

    fn assert_json_is_equivalent_inner<'a>(
        expected: &'a Value,
        actual: &'a Value,
        path: &str,
    ) {
        match (expected, actual) {
            (Value::Number(expected_num), Value::Number(actual_num)) => {
                if !expected_num.is_f64() || !actual_num.is_f64() {
                    assert_eq!(expected_num, actual_num, "path '{}': non-f64 numeric inequality: expected: {}, actual: {}", path, expected_num, actual_num);
                } else {
                    let expected_f64 = expected_num.as_f64().unwrap();
                    let actual_f64 = actual_num.as_f64().unwrap();
                    assert!(approx_eq!(
                        f64,
                        expected_f64,
                        actual_f64,
                        F64Margin {
                            ulps: 1,
                            epsilon: 0.000_000_1
                        }
                    ), "path '{}': numeric inequality out of range: expected: {}, actual: {}", path, expected_f64, actual_f64);
                }
            }
            (Value::Object(expected_map), Value::Object(actual_map)) => {
                
                let mut expected_key_missing_from_actual = None;
                for (key, expected_value) in expected_map {
                    let actual_value = actual_map.get(key);
                    if actual_value.is_none() {
                        expected_key_missing_from_actual = Some(key);
                        break;
                    }
                    assert_json_is_equivalent_inner(expected_value, actual_value.unwrap(), &(format!("{}.{}", path, key)));
                }
                assert!(expected_key_missing_from_actual.is_none(), "path '{}': expected map has key not in actual map: {}", path, expected_key_missing_from_actual.unwrap());
                for (key, _) in actual_map {
                    assert!(expected_map.contains_key(key), "path '{}': actual map has key not in expected map: {}", path, key);
                }
            }
            (Value::Array(expected_array), Value::Array(actual_array)) => {
                assert!(expected_array.len() == actual_array.len(), "expected array has more elements than actual array");
                for (index, (expected_value, actual_value)) in expected_array.iter().zip(actual_array).enumerate() {
                    assert_json_is_equivalent_inner(expected_value, actual_value, &(format!("{}[{}]", path, index)));
                }
            }
            (expected, actual) => assert_eq!(expected, actual, "path '{}': inequality: expected: {:?}, actual: {:?}", path, expected, actual),
        };
    }

    #[test]
    fn it_correctly_converts_aspen_8() {
        let input = read_to_string("tests/qcs-isa-Aspen-8.json")
            .expect("Could not read Aspen 8 input data");
        let expected_json = read_to_string("tests/compiler-isa-Aspen-8.json")
            .expect("Could not read Aspen 8 output data");
        let qcs_isa: InstructionSetArchitecture =
            serde_json::from_str(&input).expect("Could not deserialize Aspen-8 input");
        let expected: Value =
            serde_json::from_str(&expected_json).expect("Could not deserialize Aspen-8 output");

        let compiler_isa =
            Compiler::try_from(qcs_isa).expect("Could not convert ISA to CompilerIsa");

        assert!(
            !compiler_isa.edges.contains_key("31-32"),
            "edge with Qubit 31 is excluded from the CompilerIsa"
        );
        assert!(
            !compiler_isa.qubits.contains_key("31"),
            "Qubit 31 is excluded from the CompilerIsa"
        );

        let serialized =
            serde_json::to_value(compiler_isa).expect("Unable to serialize CompilerIsa");

        assert_json_is_equivalent(&expected, &serialized);
    }

    #[test]
    fn compiler_excludes_qubits_with_no_operations() {
        let input = read_to_string("tests/qcs-isa-edges-without-gates.json")
            .expect("Could not read ISA with edges without gates");
        let qcs_isa: InstructionSetArchitecture =
            serde_json::from_str(&input).expect("Could not deserialize ISA with edges without gates");

        assert!(
            qcs_isa.architecture.nodes.contains(&Node::new(31)),
            "Qubit 31 is in the source ISA"
        );
        assert!(
            qcs_isa.architecture.edges.iter().any(|e| e.node_ids.contains(&31)),
            "edge with Qubit 31 is in the source ISA"
        );

        let compiler_isa = Compiler::try_from(qcs_isa)
            .expect("Could not convert ISA with edges without gates to CompilerIsa");

        assert!(
            !compiler_isa.qubits.contains_key("31"),
            "Qubit 31 should not be in the CompilerIsa"
        );
        assert!(
            !compiler_isa.edges.contains_key("31-32"),
            "edge with Qubit 31 should not be in the CompilerIsa"
        );

        let input_without_dead = read_to_string("tests/qcs-isa-excluding-dead-edges.json")
            .expect("Could not read ISA that excludes dead edges");
        let isa_without_dead: InstructionSetArchitecture =
            serde_json::from_str(&input_without_dead)
                .expect("Could not read ISA that excludes dead edges");
        let compiler_isa_excluding_dead = Compiler::try_from(isa_without_dead)
            .expect("Could not convert ISA with manually excluded dead edges to CompilerIsa");

        let serialized =
            serde_json::to_value(compiler_isa).expect("Unable to serialize CompilerIsa");
        let serialized_without_dead = serde_json::to_value(compiler_isa_excluding_dead)
            .expect("Unable to serialize CompilerIsa");

        assert_json_is_equivalent(&serialized, &serialized_without_dead);
    }
}
