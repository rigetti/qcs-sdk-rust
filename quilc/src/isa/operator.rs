use std::collections::HashSet;

use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

/// Contains all the operators for a single Site ([`Qubit`] or [`Edge`]) organized to allow
/// deduplication by name
#[derive(Debug)]
pub(crate) struct OperatorMap(HashSet<String>, Vec<Operator>);

impl OperatorMap {
    pub(crate) fn new() -> Self {
        Self(HashSet::new(), Vec::new())
    }

    /// Add a new batch of operators with a given name if no operators with that name have been
    /// added yet.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of the operators being added.
    /// * `operators`: The list of operators to add.
    ///
    /// returns: true if the operators were inserted, false if they weren't (due to duplicates).
    pub(crate) fn add(&mut self, name: String, operators: Vec<Operator>) -> bool {
        if !self.0.insert(name) {
            return false;
        }
        self.1.extend(operators);
        true
    }
}

impl Serialize for OperatorMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(&self.1)
    }
}

#[cfg(test)]
mod describe_operator_map {
    use super::*;

    #[test]
    fn it_serializes_as_a_list_of_operators() {
        let mut map = OperatorMap::new();

        let rx_1 = Operator::Gate {
            operator: "RX".to_string(),
            duration: 50.0,
            fidelity: 1.0,
            parameters: Parameters::Float(0.0),
            arguments: Arguments::Int(1),
        };
        let rx_2 = Operator::Gate {
            operator: "RX".to_string(),
            duration: 50.0,
            fidelity: 0.9,
            parameters: Parameters::Float(0.9),
            arguments: Arguments::Int(1),
        };
        map.add("RX".to_string(), vec![rx_1.clone(), rx_2.clone()]);
        let rz = Operator::Gate {
            operator: "RZ".to_string(),
            duration: 0.01,
            fidelity: 0.9,
            parameters: Parameters::String("_".to_string()),
            arguments: Arguments::Int(1),
        };
        map.add("RZ".to_string(), vec![rz.clone()]);
        let serialized = serde_json::to_value(&map).expect("Could not serialize OperatorMap");
        let expected = serde_json::to_value(vec![rx_1, rx_2, rz])
            .expect("Could not serialize vec of operators");
        assert_eq!(serialized, expected);
    }

    #[test]
    fn it_skips_duplicate_operators_with_the_same_name() {
        let mut map = OperatorMap::new();

        let rz = Operator::Gate {
            operator: "RZ".to_string(),
            duration: 0.01,
            fidelity: 0.9,
            parameters: Parameters::String("_".to_string()),
            arguments: Arguments::Int(1),
        };
        map.add("RZ".to_string(), vec![rz.clone()]);
        map.add("RZ".to_string(), vec![rz.clone()]);
        let serialized = serde_json::to_value(&map).expect("Could not serialize OperatorMap");
        let expected =
            serde_json::to_value(vec![rz]).expect("Could not serialize vec of operators");
        assert_eq!(serialized, expected);
    }
}

/// Represents a single operation that can be performed on a Qubit or Edge
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "operator_type")]
#[serde(rename_all = "lowercase")]
pub(crate) enum Operator {
    Gate {
        // TODO: Try to make this &str to reduce allocations
        operator: String,
        duration: f64,
        fidelity: f64,
        // TODO: Try to make these arrays or slices reduce alloc?
        parameters: Parameters,
        arguments: Arguments,
    },
    Measure {
        operator: String,
        duration: f64,
        fidelity: f64,
        qubit: i32,
        target: Option<String>,
    },
}

#[cfg(test)]
mod describe_operator {
    use super::*;

    /// This test copies some JSON data from the pyQuil ISA integration test to validate that
    /// Operator::Gate is serialized correctly.
    #[test]
    fn it_serializes_gates_like_pyquil() {
        let gate_op = Operator::Gate {
            arguments: Arguments::Int(1),
            duration: 0.5,
            fidelity: 0.5,
            operator: String::from("RZ"),
            parameters: Parameters::String(String::from("_")),
        };
        let expected = serde_json::json!({
            "arguments": [1],
            "duration": 0.5,
            "fidelity": 0.5,
            "operator": "RZ",
            "operator_type": "gate",
            "parameters": ["_"]
        });
        let serialized =
            serde_json::to_value(&gate_op).expect("Could not serialize Operation::Gate");
        assert_eq!(serialized, expected);
    }

    /// This test copies some JSON data from the pyQuil ISA integration test to validate that
    /// Operator::Measure is serialized correctly.
    #[test]
    fn it_serializes_measurements_like_pyquil() {
        let measure = Operator::Measure {
            duration: 0.5,
            fidelity: 0.5,
            qubit: 1,
            operator: String::from("MEASURE"),
            target: Some(String::from("_")),
        };
        let expected = serde_json::json!({
            "duration": 0.5,
            "fidelity": 0.5,
            "operator": "MEASURE",
            "operator_type": "measure",
            "qubit": 1,
            "target": "_"
        });
        let serialized =
            serde_json::to_value(&measure).expect("Could not serialize Operation::Gate");
        assert_eq!(serialized, expected);
    }

    /// This test copies some JSON data from the pyQuil ISA integration test to validate that
    /// Operator::Measure is serialized correctly.
    #[test]
    fn it_serializes_measurements_with_null_targets_like_pyquil() {
        let measure = Operator::Measure {
            duration: 0.5,
            fidelity: 0.5,
            qubit: 1,
            operator: String::from("MEASURE"),
            target: None,
        };
        let expected = serde_json::json!({
            "duration": 0.5,
            "fidelity": 0.5,
            "operator": "MEASURE",
            "operator_type": "measure",
            "qubit": 1,
            "target": null
        });
        let serialized =
            serde_json::to_value(&measure).expect("Could not serialize Operation::Gate");
        assert_eq!(serialized, expected);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Parameters {
    String(String),
    Float(f64),
    Empty,
}

impl Serialize for Parameters {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::String(element) => serializer.collect_seq([element]),
            Self::Float(element) => serializer.collect_seq([element]),
            Self::Empty => serializer.collect_seq(Vec::new() as Vec<i32>),
        }
    }
}

#[cfg(test)]
mod describe_parameters {
    use super::*;

    #[test]
    fn it_serializes_one_string_as_list_of_strings() {
        let p = Parameters::String(String::from("Hello Serde"));
        let serialized = serde_json::to_value(p).expect("Could not serialize");
        let expected = serde_json::json!(["Hello Serde"]);
        assert_eq!(expected, serialized);
    }

    #[test]
    fn it_serializes_one_float_as_list_of_numbers() {
        let p = Parameters::Float(1.0);
        let serialized = serde_json::to_value(p).expect("Could not serialize");
        let expected = serde_json::json!([1.0]);
        assert_eq!(expected, serialized);
    }

    #[test]
    fn it_serializes_empty_as_sequence() {
        let p = Parameters::Empty;
        let serialized = serde_json::to_value(p).expect("Could not serialize");
        let expected = serde_json::json!([]);
        assert_eq!(expected, serialized);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Arguments {
    Int(i32),
    Strings(Vec<String>),
}

impl Serialize for Arguments {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Strings(strings) => serializer.collect_seq(strings),
            Self::Int(int) => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element(int)?;
                seq.end()
            }
        }
    }
}

#[cfg(test)]
mod describe_arguments {
    use super::*;

    #[test]
    fn it_serializes_strings_as_list_of_strings() {
        let p = Arguments::Strings(vec!["First".to_string(), "Second".to_string()]);
        let serialized = serde_json::to_value(p).expect("Could not serialize");
        let expected = serde_json::json!(["First", "Second"]);
        assert_eq!(expected, serialized);
    }

    #[test]
    fn it_serializes_one_int_as_list_of_numbers() {
        let p = Arguments::Int(1);
        let serialized = serde_json::to_value(p).expect("Could not serialize");
        let expected = serde_json::json!([1]);
        assert_eq!(expected, serialized);
    }
}

pub(crate) const PERFECT_FIDELITY: f64 = 1.0;
pub(crate) const PERFECT_DURATION: f64 = 1.0 / 100.0;
