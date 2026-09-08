use crate::{AbiConfig, AbiExceptionConfig, ApiFnDef, Context, ContextNode, query::{FnDef, structured_query}};
use std::{collections::{HashMap, HashSet}, fmt::Debug, hash::Hash, path::PathBuf, sync::LazyLock};

mod parsed;
pub use parsed::*;
mod interpreted;
pub use interpreted::*;
mod processed;
pub use processed::*;

pub static CRATE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
});

pub static INPUT_CTX: LazyLock<Context> = LazyLock::new(|| {
    Context::from_file(CRATE_DIR.join("src/test/input.h")).expect("Failed to parse input")
});

pub static H_EXPECTED_CTX: LazyLock<Context> = LazyLock::new(|| {
    Context::from_file(CRATE_DIR.join("src/test/output.h")).expect("Failed to parse input")
});

structured_query! {
    name: ExpectedHeader,
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
    ExpectedHeader::Match::from_node(H_EXPECTED_CTX.root()).into_iter().map(|m| {
        (m.name.as_str().expect("Bad name"), m.full)
    }).collect()
});

pub static CPP_EXPECTED_CTX: LazyLock<Context> = LazyLock::new(|| {
    Context::from_file(CRATE_DIR.join("src/test/output.cpp")).expect("Failed to parse input")
});

structured_query! {
    name: ExpectedCpp,
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
    ExpectedCpp::Match::from_node(CPP_EXPECTED_CTX.root()).into_iter().map(|m| {
        (m.name.as_str().expect("Bad name"), m.full)
    }).collect()
});

pub static PARSED_ACTUAL: LazyLock<HashMap<&'static str, FnDef::Match<'static>>> = LazyLock::new(|| {
    FnDef::Match::from_node(INPUT_CTX.root()).into_iter().map(|m| {
        (m.decl.name.as_str().expect("Bad name"), m)
    }).collect()
});

pub static INTERPRETED_ACTUAL: LazyLock<HashMap<&'static str, ApiFnDef>> = LazyLock::new(|| {
    PARSED_ACTUAL.iter().map(|(name, m)| {
        (*name, ApiFnDef::try_from(m).expect("Failed to interpret input"))
    }).collect()
});

pub static PROCESSED_ACTUAL: LazyLock<HashMap<&'static str, ApiFnDef>> = LazyLock::new(|| {
    INTERPRETED_ACTUAL.iter().map(|(name, def)| {
        (*name, def.clone().to_c(&CONFIG))
    }).collect()
});

// pub static H_ACTUAL: LazyLock<HashMap<&'static str, ContextNode<'static>>> = LazyLock::new(|| {
//     PROCESSED_ACTUAL.iter().map(|(name, def)| {
//         (*name, def.header().to_string())
//     }).collect()
// });

// pub static CPP_ACTUAL: LazyLock<HashMap<&'static str, ContextNode<'static>>> = LazyLock::new(|| {
//     PROCESSED_ACTUAL.iter().map(|(name, def)| {
//         (*name, def.code(&CONFIG).to_string())
//     }).collect()
// });

pub static CONFIG: LazyLock<AbiConfig> = LazyLock::new(|| AbiConfig {
    exception: AbiExceptionConfig {
        value_type: "Result".to_string(),
        success_value: "Result::Success".to_string(),
        nullref_value: "Result::ArgumentNull".to_string(),
        generic_value: "Result::Unknown".to_string(),
        custom_values: std::collections::HashMap::new(),
    },
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
    ($name:ident) => {
        #[cfg(test)]
        mod $name {
            #[allow(unused)]
            use super::*;

            #[test]
            fn parsed() {
                assert_eq!(
                    PARSED_EXPECTED.get(stringify!($name)).expect(concat!("No expected ", stringify!($name))),
                    PARSED_ACTUAL.get(stringify!($name)).expect(concat!("No actual ", stringify!($name))),
                );
            }

            #[test]
            fn interpreted() {
                assert_eq!(
                    INTERPRETED_EXPECTED.get(stringify!($name)).expect(concat!("No expected ", stringify!($name))),
                    INTERPRETED_ACTUAL.get(stringify!($name)).expect(concat!("No actual ", stringify!($name))),
                );
            }

            #[test]
            fn processed() {
                assert_eq!(
                    PROCESSED_EXPECTED.get(stringify!($name)).expect(concat!("No expected ", stringify!($name))),
                    PROCESSED_ACTUAL.get(stringify!($name)).expect(concat!("No actual ", stringify!($name))),
                );
            }

            // #[test]
            // fn h() {
            //     assert_eq!(
            //         H_EXPECTED.get(concat!("myclass_", stringify!($name))).expect(concat!("No expected ", stringify!($name))),
            //         H_ACTUAL.get(stringify!($name)).expect(concat!("No actual ", stringify!($name))),
            //     );
            // }

            // #[test]
            // fn cpp() {
            //     assert_eq!(
            //         CPP_EXPECTED.get(concat!("myclass_", stringify!($name))).expect(concat!("No expected ", stringify!($name))),
            //         CPP_ACTUAL.get(stringify!($name)).expect(concat!("No actual ", stringify!($name))),
            //     );
            // }
        }
    };
);

gen_test!(config);
gen_test!(set_config);
