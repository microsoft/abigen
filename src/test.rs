use crate::{
    AbiConfig, Api, ApiFnDef, Context, ContextNode, Template,
    query::{FnDef, structured_query},
};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    path::PathBuf,
    sync::LazyLock,
};

mod expected_interpreted;
mod expected_parsed;

pub static CRATE_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));

pub static CONFIG: LazyLock<AbiConfig> = LazyLock::new(|| {
    let buf = std::fs::read(CRATE_DIR.join("src/test/config.toml")).expect("Failed to read config");
    let c: AbiConfig = toml::from_slice(&buf).expect("Failed to parse config");
    c.with_defaults()
});

structured_query! {
    /// Find function declarations in a header file
    name: HeaderDecl,
    fields: {
        /// The name of the function being declared
        name: _,
        /// The node of the overall declaration
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

structured_query! {
    /// Find function definitions in a header file
    name: CodeDef,
    fields: {
        /// The name of the function being declared
        name: _,
        /// The node of the overall declaration
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

pub mod input {
    use super::*;

    pub static CTX: LazyLock<Context> = LazyLock::new(|| {
        Context::from_file(CRATE_DIR.join("src/test/input.h")).expect("Failed to parse input")
    });

    pub static TEMPLATE_H_CTX: LazyLock<Context> = LazyLock::new(|| {
        Context::from_file(CRATE_DIR.join("src/test/input_template.h"))
            .expect("Failed to parse template header")
    });

    pub static TEMPLATE_H: LazyLock<Template> = LazyLock::new(|| {
        Template::from_buffer(TEMPLATE_H_CTX.as_str()).expect("Failed to interpret template header")
    });

    pub static TEMPLATE_CPP_CTX: LazyLock<Context> = LazyLock::new(|| {
        Context::from_file(CRATE_DIR.join("src/test/input_template.cpp"))
            .expect("Failed to parse template impl")
    });

    pub static TEMPLATE_CPP: LazyLock<Template> = LazyLock::new(|| {
        Template::from_buffer(TEMPLATE_CPP_CTX.as_str()).expect("Failed to interpret template impl")
    });
}

pub mod expected {
    use super::*;
    pub use expected_interpreted::*;
    pub use expected_parsed::*;

    pub static H_TEMPLATED: LazyLock<Context> = LazyLock::new(|| {
        Context::from_file(CRATE_DIR.join("src/test/expected.h")).expect("Failed to parse input")
    });

    pub static H_MAP: LazyLock<HashMap<&'static str, ContextNode<'static>>> = LazyLock::new(|| {
        HeaderDecl::Match::from_node(H_TEMPLATED.root())
            .into_iter()
            .map(|m| (m.name.as_str(), m.full))
            .collect()
    });

    pub static CPP_TEMPLATED: LazyLock<Context> = LazyLock::new(|| {
        Context::from_file(CRATE_DIR.join("src/test/expected.cpp")).expect("Failed to parse input")
    });

    pub static CPP_MAP: LazyLock<HashMap<&'static str, ContextNode<'static>>> =
        LazyLock::new(|| {
            CodeDef::Match::from_node(CPP_TEMPLATED.root())
                .into_iter()
                .map(|m| (m.name.as_str(), m.full))
                .collect()
        });
}

pub mod actual {
    use super::*;

    pub static PARSED: LazyLock<HashMap<&'static str, FnDef::Match<'static>>> =
        LazyLock::new(|| {
            FnDef::Match::from_node(input::CTX.root())
                .into_iter()
                .map(|m| (m.decl.name.as_str(), m))
                .collect()
        });

    pub static INTERPRETED: LazyLock<HashMap<&'static str, ApiFnDef>> = LazyLock::new(|| {
        PARSED
            .iter()
            .map(|(name, m)| (*name, ApiFnDef::from(m)))
            .collect()
    });

    pub static API: LazyLock<Api> =
        LazyLock::new(|| Api::from_buffer(input::CTX.as_str()).expect("Failed to populate API"));

    pub static H_TEMPLATED: LazyLock<Context> =
        LazyLock::new(|| Context::from(input::TEMPLATE_H.header(&API, &CONFIG).to_string()));

    pub static H_FULL: LazyLock<Context> =
        LazyLock::new(|| Context::from(API.header(&CONFIG).to_string()));

    pub static H_MAP: LazyLock<HashMap<&'static str, ContextNode<'static>>> = LazyLock::new(|| {
        HeaderDecl::Match::from_node(H_FULL.root())
            .into_iter()
            .map(|m| (m.name.as_str(), m.full))
            .collect()
    });

    pub static CPP_TEMPLATED: LazyLock<Context> =
        LazyLock::new(|| Context::from(input::TEMPLATE_CPP.code(&API, &CONFIG).to_string()));

    pub static CPP_FULL: LazyLock<Context> = LazyLock::new(|| {
        let api = Api::from_buffer(input::CTX.as_str()).expect("Failed to populate API");
        Context::from(api.code(&CONFIG).to_string())
    });

    pub static CPP_MAP: LazyLock<HashMap<&'static str, ContextNode<'static>>> =
        LazyLock::new(|| {
            CodeDef::Match::from_node(CPP_FULL.root())
                .into_iter()
                .map(|m| (m.name.as_str(), m.full))
                .collect()
        });

    #[test]
    #[ignore]
    fn print_h() {
        println!("{}", H_FULL.as_str());
        panic!()
    }

    #[test]
    #[ignore]
    fn print_cpp() {
        println!("{}", CPP_FULL.as_str());
        panic!()
    }
}

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
fn header() {
    actual::H_TEMPLATED
        .root()
        .diff(&expected::H_TEMPLATED.root())
        .expect("Mismatch");
}

#[test]
fn code() {
    actual::CPP_TEMPLATED
        .root()
        .diff(&expected::CPP_TEMPLATED.root())
        .expect("Mismatch");
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
                    expected::PARSED.get(stringify!($cpp_name)).expect(concat!("No expected ", stringify!($cpp_name))),
                    actual::PARSED.get(stringify!($cpp_name)).expect(concat!("No actual ", stringify!($cpp_name))),
                );
            }

            #[test]
            fn interpreted() {
                assert_eq!(
                    expected::INTERPRETED.get(stringify!($cpp_name)).expect(concat!("No expected ", stringify!($cpp_name))),
                    actual::INTERPRETED.get(stringify!($cpp_name)).expect(concat!("No actual ", stringify!($cpp_name))),
                );
            }

            #[test]
            fn h() {
                actual::H_MAP.get(stringify!($c_name))
                    .expect(concat!("No actual ", stringify!($cpp_name)))
                    .diff(
                        expected::H_MAP.get(stringify!($c_name))
                            .expect(concat!("No expected ", stringify!($cpp_name)))
                    ).expect("Mismatch");
            }

            #[test]
            fn cpp() {
                actual::CPP_MAP.get(stringify!($c_name))
                    .expect(concat!("No actual ", stringify!($cpp_name)))
                    .diff(
                        expected::CPP_MAP.get(stringify!($c_name))
                            .expect(concat!("No expected ", stringify!($cpp_name)))
                    ).expect("Mismatch");
            }
        }
    };
);

gen_test!(do_something, x_y_myclass_subclass_do_something);
gen_test!(config, x_y_myclass_config);
gen_test!(set_config, x_y_myclass_set_config);
