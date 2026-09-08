use crate::{AbiConfig, AbiExceptionConfig, ApiFnDef, Context, query::{FnDef, structured_query}};
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

pub static CPP_EXPECTED_CTX: LazyLock<Context> = LazyLock::new(|| {
    Context::from_file(CRATE_DIR.join("src/test/output.cpp")).expect("Failed to parse input")
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

pub static H_EXPECTED: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    ExpectedHeader::Match::from_node(H_EXPECTED_CTX.root()).into_iter().map(|m| {
        (m.name.as_str().expect("Bad name"), m.full.as_str().expect("Bad body"))
    }).collect()
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

pub static CPP_EXPECTED: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    ExpectedCpp::Match::from_node(CPP_EXPECTED_CTX.root()).into_iter().map(|m| {
        (m.name.as_str().expect("Bad name"), m.full.as_str().expect("Bad body"))
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

pub static H_ACTUAL: LazyLock<HashMap<&'static str, String>> = LazyLock::new(|| {
    PROCESSED_ACTUAL.iter().map(|(name, def)| {
        (*name, def.header().to_string())
    }).collect()
});

pub static CPP_ACTUAL: LazyLock<HashMap<&'static str, String>> = LazyLock::new(|| {
    PROCESSED_ACTUAL.iter().map(|(name, def)| {
        (*name, def.code(&CONFIG).to_string())
    }).collect()
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

#[test]
fn config_parsed() {
    assert_eq!(
        PARSED_EXPECTED.get("config").expect("No expected config"),
        PARSED_ACTUAL.get("config").expect("No actual config"),
    );
}

#[test]
fn config_interpreted() {
    assert_eq!(
        INTERPRETED_EXPECTED.get("config").expect("No expected config"),
        INTERPRETED_ACTUAL.get("config").expect("No actual config"),
    );
}

#[test]
fn config_processed() {
    assert_eq!(
        PROCESSED_EXPECTED.get("config").expect("No expected config"),
        PROCESSED_ACTUAL.get("config").expect("No actual config"),
    );
}

#[test]
fn config_h() {
    assert_eq!(
        H_EXPECTED.get("myclass_config").expect("No expected config"),
        H_ACTUAL.get("config").expect("No actual config"),
    );
}

#[test]
fn config_c() {
    assert_eq!(
        CPP_EXPECTED.get("myclass_config").expect("No expected config"),
        CPP_ACTUAL.get("config").expect("No actual config"),
    );
}