use serde::{Deserialize, Serialize};

pub(crate) fn wildcard(node_id: Option<i32>) -> Operator {
    let arg = node_id.map_or_else(|| Argument::String("_".to_owned()), Argument::Int);
    Operator::Gate {
        operator: "_".to_string(),
        duration: PERFECT_DURATION,
        fidelity: PERFECT_FIDELITY,
        parameters: vec![Parameter::String("_".to_owned())],
        arguments: vec![arg],
    }
}

/// Represents a single operation that can be performed on a Qubit or Edge
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "operator_type")]
#[serde(rename_all = "lowercase")]
pub(crate) enum Operator {
    Gate {
        operator: String,
        duration: f64,
        fidelity: f64,
        parameters: Vec<Parameter>,
        arguments: Vec<Argument>,
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
    use crate::qpu::quilc::isa::operator::{Argument, Operator, Parameter};

    /// This test copies some JSON data from the pyQuil ISA integration test to
    /// validate that [`Operator::Gate`] is serialized correctly.
    #[test]
    fn it_serializes_gates_like_pyquil() {
        let gate_op = Operator::Gate {
            arguments: vec![Argument::Int(1)],
            duration: 0.5,
            fidelity: 0.5,
            operator: "RZ".to_string(),
            parameters: vec![Parameter::String("_".to_owned())],
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
    /// [`Operator::Measure`] is serialized correctly.
    #[test]
    fn it_serializes_measurements_like_pyquil() {
        let measure = Operator::Measure {
            duration: 0.5,
            fidelity: 0.5,
            qubit: 1,
            operator: "MEASURE".to_string(),
            target: Some("_".to_string()),
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
    /// [`Operator::Measure`] is serialized correctly.
    #[test]
    fn it_serializes_measurements_with_null_targets_like_pyquil() {
        let measure = Operator::Measure {
            duration: 0.5,
            fidelity: 0.5,
            qubit: 1,
            operator: "MEASURE".to_string(),
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub(crate) enum Parameter {
    String(String),
    Float(f64),
}

#[cfg(test)]
mod test_parameters_deser {
    use super::Parameter;

    #[test]
    fn it_deserializes_parameters() {
        let s = r##"[""]"##;
        serde_json::from_str::<Vec<Parameter>>(s).expect("yo");
    }
}

#[cfg(test)]
mod describe_parameters {
    use crate::qpu::quilc::isa::operator::Parameter;

    #[test]
    fn it_serializes_underscore_as_list_of_strings() {
        let p = Parameter::String("_".to_owned());
        let serialized = serde_json::to_value(p).expect("Could not serialize");
        let expected = serde_json::json!("_");
        assert_eq!(expected, serialized);
    }

    #[test]
    fn it_serializes_one_float_as_list_of_numbers() {
        let p = Parameter::Float(1.0);
        let serialized = serde_json::to_value(p).expect("Could not serialize");
        let expected = serde_json::json!(1.0);
        assert_eq!(expected, serialized);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub(crate) enum Argument {
    Int(i32),
    String(String),
}

#[cfg(test)]
mod describe_arguments {
    use crate::qpu::quilc::isa::operator::Argument;

    #[test]
    fn it_serializes_underscores_as_list_of_strings() {
        let p = vec![Argument::String("_".to_owned())];
        let serialized = serde_json::to_value(p).expect("Could not serialize");
        let expected = serde_json::json!(["_"]);
        assert_eq!(expected, serialized);
    }

    #[test]
    fn it_serializes_one_int_as_list_of_numbers() {
        let p = vec![Argument::Int(1)];
        let serialized = serde_json::to_value(p).expect("Could not serialize");
        let expected = serde_json::json!([1]);
        assert_eq!(expected, serialized);
    }
}

pub(crate) const PERFECT_FIDELITY: f64 = 1.0;
pub(crate) const PERFECT_DURATION: f64 = 1.0 / 100.0;
