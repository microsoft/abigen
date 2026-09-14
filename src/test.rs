use crate::{AbiConfig, AbiExceptionConfig, Api, ApiFnDef, Context, ContextNode, Template, query::{FnDef, structured_query}};
use std::{collections::{HashMap, HashSet}, fmt::Debug, hash::Hash, path::PathBuf, sync::LazyLock};

mod expected_parsed;
pub use expected_parsed::*;
mod expected_interpreted;
pub use expected_interpreted::*;
mod expected_processed;
pub use expected_processed::*;

pub static CRATE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
});

pub static CONFIG: LazyLock<AbiConfig> = LazyLock::new(|| AbiConfig {
    exception: AbiExceptionConfig {
        value_type: "Result".to_string(),
        success_value: "Result::Success".to_string(),
        nullref_value: "Result::ArgumentNull".to_string(),
        generic_value: "Result::Unknown".to_string(),
        custom_values: std::collections::HashMap::new(),
    },
});

pub static INPUT_CTX: LazyLock<Context> = LazyLock::new(|| {
    Context::from_file(CRATE_DIR.join("src/test/input.h")).expect("Failed to parse input")
});

pub static TEMPLATE_H_CTX: LazyLock<Context> = LazyLock::new(|| {
    Context::from_file(CRATE_DIR.join("src/test/input_template.h")).expect("Failed to parse template header")
});

pub static TEMPLATE_H: LazyLock<Template> = LazyLock::new(|| {
    Template::from_buffer(TEMPLATE_H_CTX.as_str()).expect("Failed to interpret template header")
});

pub static TEMPLATE_CPP_CTX: LazyLock<Context> = LazyLock::new(|| {
    Context::from_file(CRATE_DIR.join("src/test/input_template.cpp")).expect("Failed to parse template impl")
});

pub static TEMPLATE_CPP: LazyLock<Template> = LazyLock::new(|| {
    Template::from_buffer(TEMPLATE_H_CTX.as_str()).expect("Failed to interpret template impl")
});

pub static H_EXPECTED_CTX: LazyLock<Context> = LazyLock::new(|| {
    Context::from_file(CRATE_DIR.join("src/test/expected.h")).expect("Failed to parse input")
});

pub static CPP_EXPECTED_CTX: LazyLock<Context> = LazyLock::new(|| {
    Context::from_file(CRATE_DIR.join("src/test/expected.cpp")).expect("Failed to parse input")
});

pub static H_ACTUAL_CTX: LazyLock<Context> = LazyLock::new(|| {
    let api = Api::from_buffer(INPUT_CTX.as_str()).expect("Failed to populate API");
    Context::from(format!("{}", api.to_c(&CONFIG).header(&CONFIG)))
});

pub static CPP_ACTUAL_CTX: LazyLock<Context> = LazyLock::new(|| {
    let api = Api::from_buffer(INPUT_CTX.as_str()).expect("Failed to populate API");
    Context::from(format!("{}", api.to_c(&CONFIG).code(&CONFIG)))
});

pub static H_ACTUAL: LazyLock<HashMap<&'static str, ContextNode<'static>>> = LazyLock::new(|| {
    HeaderDecl::Match::from_node(H_ACTUAL_CTX.root()).into_iter().map(|m| {
        (m.name.as_str(), m.full)
    }).collect()
});

pub static CPP_ACTUAL: LazyLock<HashMap<&'static str, ContextNode<'static>>> = LazyLock::new(|| {
    CodeDef::Match::from_node(CPP_ACTUAL_CTX.root()).into_iter().map(|m| {
        (m.name.as_str(), m.full)
    }).collect()
});

structured_query! {
    name: HeaderDecl,
    fields: {
        name: _,
        full: _,
    },
    query: r#"
        (declaration
            declarator: (function_declarator
                declarator: (identifier) @name
            )
        ) @full
    "#,
}

pub static H_EXPECTED: LazyLock<HashMap<&'static str, ContextNode<'static>>> = LazyLock::new(|| {
    HeaderDecl::Match::from_node(H_EXPECTED_CTX.root()).into_iter().map(|m| {
        (m.name.as_str(), m.full)
    }).collect()
});

structured_query! {
    name: CodeDef,
    fields: {
        name: _,
        full: _,
    },
    query: r#"
        (function_definition
            declarator: (function_declarator
                declarator: (identifier) @name
            )
        ) @full
    "#,
}

pub static CPP_EXPECTED: LazyLock<HashMap<&'static str, ContextNode<'static>>> = LazyLock::new(|| {
    CodeDef::Match::from_node(CPP_EXPECTED_CTX.root()).into_iter().map(|m| {
        (m.name.as_str(), m.full)
    }).collect()
});

pub static PARSED_ACTUAL: LazyLock<HashMap<&'static str, FnDef::Match<'static>>> = LazyLock::new(|| {
    FnDef::Match::from_node(INPUT_CTX.root()).into_iter().map(|m| {
        (m.decl.name.as_str(), m)
    }).collect()
});

pub static INTERPRETED_ACTUAL: LazyLock<HashMap<&'static str, ApiFnDef>> = LazyLock::new(|| {
    PARSED_ACTUAL.iter().map(|(name, m)| {
        (*name, ApiFnDef::from(m))
    }).collect()
});

pub static PROCESSED_ACTUAL: LazyLock<HashMap<&'static str, ApiFnDef>> = LazyLock::new(|| {
    INTERPRETED_ACTUAL.iter().map(|(name, def)| {
        (*name, def.clone().to_c(&CONFIG))
    }).collect()
});

pub fn collection_eq<A, E>(
    actual: impl IntoIterator<Item = A>,
    expected: impl IntoIterator<Item = E>,
) where
    A: Debug,
    E: Debug + PartialEq + Eq + Hash + PartialEq<A>,
{
    let mut expected: HashSet<_> = expected.into_iter().collect();

    let mut unexpected = vec![];
    for item in actual {
        let expected = expected.extract_if(|i| i == &item).next();
        if expected.is_none() {
            unexpected.push(item);
        }
    }
    let unmatched: Vec<_> = expected.into_iter().collect();

    assert_eq!(unmatched, unexpected, "expected (left) != actual (right)");
}

macro_rules! gen_test(
    ($cpp_name:ident, $c_name:ident) => {
        #[cfg(test)]
        mod $cpp_name {
            #[allow(unused)]
            use super::*;

            #[test]
            fn parsed() {
                assert_eq!(
                    PARSED_EXPECTED.get(stringify!($cpp_name)).expect(concat!("No expected ", stringify!($cpp_name))),
                    PARSED_ACTUAL.get(stringify!($cpp_name)).expect(concat!("No actual ", stringify!($cpp_name))),
                );
            }

            #[test]
            fn interpreted() {
                assert_eq!(
                    INTERPRETED_EXPECTED.get(stringify!($cpp_name)).expect(concat!("No expected ", stringify!($cpp_name))),
                    INTERPRETED_ACTUAL.get(stringify!($cpp_name)).expect(concat!("No actual ", stringify!($cpp_name))),
                );
            }

            #[test]
            fn processed() {
                assert_eq!(
                    PROCESSED_EXPECTED.get(stringify!($cpp_name)).expect(concat!("No expected ", stringify!($cpp_name))),
                    PROCESSED_ACTUAL.get(stringify!($cpp_name)).expect(concat!("No actual ", stringify!($cpp_name))),
                );
            }

            #[test]
            fn h() {
                H_EXPECTED.get(stringify!($c_name))
                    .expect(concat!("No expected ", stringify!($cpp_name)))
                    .diff(
                        H_ACTUAL.get(stringify!($c_name))
                            .expect(concat!("No actual ", stringify!($cpp_name)))
                    ).expect("Mismatch");
            }

            #[test]
            fn cpp() {
                CPP_EXPECTED.get(stringify!($c_name))
                    .expect(concat!("No expected ", stringify!($cpp_name)))
                    .diff(
                        CPP_ACTUAL.get(stringify!($c_name))
                            .expect(concat!("No actual ", stringify!($cpp_name)))
                    ).expect("Mismatch");
            }
        }
    };
);

gen_test!(config, myclass_config);
gen_test!(set_config, myclass_set_config);
