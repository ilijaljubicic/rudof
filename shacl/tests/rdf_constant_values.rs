//! Constant values must reach the native constraint evaluator without losing RDF identity.

use rudof_rdf::rdf_core::RDFFormat;
use rudof_rdf::rdf_core::term::Object;
use rudof_rdf::rdf_impl::{OxigraphInMemory, ReaderMode};
use shacl::ir::{IRComponent, IRSchema};
use shacl::rdf::ShaclParser;
use shacl::validator::processor::{GraphValidation, ShaclProcessor};
use shacl::validator::store::Graph;
use shacl::validator::{ShaclConfig, ShaclValidationMode};

const VALUES: &[&str] = &[
    "ex:required",
    "\"required\"",
    "\"مرحبا\"@ar--rtl",
    "_:required",
    "<<( ex:s ex:p ex:o )>>",
    "<<( ex:s ex:p <<( ex:a ex:b ex:c )>> )>>",
];

fn graph(constraint: &str, constant: &str, data: &str) -> OxigraphInMemory {
    let parameter = if constraint == "in" {
        format!("({constant})")
    } else {
        constant.to_string()
    };
    let turtle = format!(
        "@prefix ex: <urn:example:> .\n\
         @prefix sh: <http://www.w3.org/ns/shacl#> .\n\
         ex:S a sh:PropertyShape ; sh:targetNode ex:focus ;\n\
           sh:path ex:value ; sh:{constraint} {parameter} .\n{data}"
    );
    OxigraphInMemory::from_str(&turtle, &RDFFormat::Turtle, None, &ReaderMode::Strict).expect("valid RDF fixture")
}

fn compile(graph: &OxigraphInMemory) -> IRSchema {
    let ast = ShaclParser::new(graph.clone())
        .parse()
        .expect("RDF constants must be accepted by the SHACL parser");
    IRSchema::compile(&ast).expect("native constraint compilation")
}

fn conforms(graph: OxigraphInMemory) -> bool {
    let schema = compile(&graph);
    let graph = Graph::try_from(graph).expect("native validation graph");
    GraphValidation::new(graph)
        .validate(&schema, &ShaclValidationMode::Native, &ShaclConfig::default())
        .expect("native constraint execution")
        .conforms()
}

#[test]
fn has_value_preserves_all_native_constant_terms() {
    for constant in VALUES {
        let graph = graph("hasValue", constant, "");
        let expected: Object = graph
            .quads()
            .find(|quad| quad.predicate.as_str() == "http://www.w3.org/ns/shacl#hasValue")
            .expect("constant parameter")
            .object
            .try_into()
            .expect("native RDF object");
        let schema = compile(&graph);
        let actual = schema
            .iter()
            .flat_map(|(_, shape)| shape.components())
            .find_map(|component| match component {
                IRComponent::HasValue(component) => Some(component.value()),
                _ => None,
            })
            .expect("native HasValue component");
        assert_eq!(actual, &expected, "constant identity changed: {constant}");
        assert_eq!(graph.quads().count(), 4, "inner triples were asserted: {constant}");
    }
}

#[test]
fn has_value_validates_matching_distinct_and_missing_values() {
    for constant in VALUES {
        assert!(
            conforms(graph("hasValue", constant, &format!("ex:focus ex:value {constant} ."))),
            "matching value rejected: {constant}"
        );
        assert!(
            !conforms(graph("hasValue", constant, "ex:focus ex:value ex:other .")),
            "distinct value accepted: {constant}"
        );
        assert!(
            !conforms(graph("hasValue", constant, "")),
            "missing value accepted: {constant}"
        );
    }
}

#[test]
fn has_value_distinguishes_blank_nodes_direction_and_nested_triples() {
    for (required, different) in [
        ("_:required", "_:other"),
        ("\"مرحبا\"@ar--rtl", "\"مرحبا\"@ar--ltr"),
        (
            "<<( ex:s ex:p <<( ex:a ex:b ex:c )>> )>>",
            "<<( ex:s ex:p <<( ex:a ex:b ex:other )>> )>>",
        ),
    ] {
        assert!(
            !conforms(graph("hasValue", required, &format!("ex:focus ex:value {different} ."))),
            "RDF identity was lost between {required} and {different}"
        );
    }
}

#[test]
fn in_preserves_and_validates_the_same_constants() {
    for constant in VALUES {
        assert!(
            conforms(graph("in", constant, &format!("ex:focus ex:value {constant} ."))),
            "matching sh:in value: {constant}"
        );
        assert!(!conforms(graph("in", constant, "ex:focus ex:value ex:other .")));
    }
}
