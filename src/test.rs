use crate::{
    AbiConfig, Api, ApiClassDef, Context, ContextNode, Template,
    query::{ClassDef, structured_query},
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
    name: HeaderDecl,
    fields: {
        decl: (FnDecl),
        full: _,
    },
    query: r#"
        (declaration
            declarator: (_) @decl
        ) @full
    "#
}

structured_query! {
    /// Find function definitions in a header file
    name: CodeDef,
    fields: {
        /// The name of the function being declared
        def: (FnDecl),
        /// The node of the overall declaration
        full: _,
    },
    query: r#"
        (function_definition
            declarator: (_) @def
        ) @full
    "#,
}

structured_query! {
    name: FnDecl,
    fields: {
        name: _,
    },
    query: r#"
        (function_declarator
            declarator: (identifier) @name
        )
    "#
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
            .map(|m| (m.decl.name.as_str(), m.full))
            .collect()
    });

    pub static CPP_TEMPLATED: LazyLock<Context> = LazyLock::new(|| {
        Context::from_file(CRATE_DIR.join("src/test/expected.cpp")).expect("Failed to parse input")
    });

    pub static CPP_MAP: LazyLock<HashMap<&'static str, ContextNode<'static>>> =
        LazyLock::new(|| {
            CodeDef::Match::from_node(CPP_TEMPLATED.root())
                .into_iter()
                .map(|m| (m.def.name.as_str(), m.full))
                .collect()
        });
}

pub mod actual {
    use super::*;
    use crate::TypeName;

    /// Maps the unqualified class name to its query structure
    pub static PARSED: LazyLock<HashMap<&'static str, ClassDef::Match<'static>>> =
        LazyLock::new(|| {
            ClassDef::Match::from_node(input::CTX.root())
                .into_iter()
                .map(|m| (m.name.as_str(), m))
                .collect()
        });

    /// Maps the fully qualified class name to its api structure
    pub static INTERPRETED: LazyLock<HashMap<TypeName, ApiClassDef>> = LazyLock::new(|| {
        PARSED
            .values()
            .map(|m| {
                let m = ApiClassDef::from(m);
                (m.name.clone(), m)
            })
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
            .map(|m| (m.decl.name.as_str(), m.full))
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
                .map(|m| (m.def.name.as_str(), m.full))
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
    ($($class_part:ident)::+, $method:ident) => {
        #[cfg(test)]
        mod $method {
            #[allow(unused)]
            use super::*;

            #[test]
            fn parsed() {
                let name = $crate::TypeName::from_iter([$(stringify!($class_part)),+]);
                let uqname = name.iter().last().unwrap();
                let expected_class = expected::PARSED.get(uqname).unwrap_or_else(|| panic!("No expected '{uqname}'"));
                let actual_class = actual::PARSED.get(uqname).unwrap_or_else(|| panic!("No actual '{uqname}'"));
                
                if stringify!($method) == "new" {
                    collection_eq(&actual_class.ctor, &expected_class.ctor);
                } else {
                    collection_eq(
                        actual_class.methods.iter().filter(|m| m.decl.name.as_str() == stringify!($method)),
                        expected_class.methods.iter().filter(|m| m.decl.name == stringify!($method)),
                    );
                }
            }

            #[test]
            fn interpreted() {
                let name = $crate::TypeName::from_iter([$(stringify!($class_part)),+]);
                let expected_class = expected::INTERPRETED.get(&name)
                    .expect(concat!("No expected ", stringify!($($class_part)::+)));
                let actual_class = actual::INTERPRETED.get(&name)
                    .expect(concat!("No actual ", stringify!($($class_part)::+)));

                collection_eq(
                    actual_class.methods.iter().filter(|m| m.role.as_str() == stringify!($method)),
                    expected_class.methods.iter().filter(|m| m.role.as_str() == stringify!($method)),
                );
            }

            #[test]
            fn h() {
                use $crate::format::fn_wrap;

                // find expected names
                let full_name = $crate::TypeName::from_iter([$(stringify!($class_part)),+, stringify!($method)]);
                let full_name = fn_wrap!(full_name.snake_case).to_string();
                let expected_names: HashSet<_> = expected::H_MAP.keys().filter_map(|k| {
                    if k.starts_with(&full_name) {
                        Some(*k)
                    } else {
                        None
                    }
                }).collect();

                // find actual names
                let class = actual::API.classes.iter().find(|c| {
                    c.name == $crate::TypeName::from_iter([$(stringify!($class_part)),+])
                }).expect("No actual class");
                let method = stringify!($method);
                let actual_names: HashSet<_> = class.methods.iter()
                    .filter_map(|m| {
                        if m.role.as_str() == method {
                            Some(fn_wrap!(m.name_c, class.name_collisions.contains(&m.role)).to_string())
                        } else {
                            None
                        }
                    }).collect();

                collection_eq(&expected_names, &actual_names);

                let mut errors = 0;
                for name in expected_names {
                    let r = actual::H_MAP.get(name)
                        .unwrap_or_else(|| panic!("No actual {name}"))
                        .diff(
                            expected::H_MAP.get(name)
                                .unwrap_or_else(|| panic!("No expected {name}"))
                        );
                    if let Err(e) = r {
                        eprintln!("Error in {name}:\n{e}\n");
                        errors += 1;
                    }
                }

                if errors > 0 {
                    panic!("Mismatch");
                }
            }

            #[test]
            fn cpp() {
                use $crate::format::fn_wrap;

                // find expected names
                let full_name = $crate::TypeName::from_iter([$(stringify!($class_part)),+, stringify!($method)]);
                let full_name = fn_wrap!(full_name.snake_case).to_string();
                let expected_names: HashSet<_> = expected::CPP_MAP.keys().filter_map(|k| {
                    if k.starts_with(&full_name) {
                        Some(*k)
                    } else {
                        None
                    }
                }).collect();

                // find actual names
                let class = actual::API.classes.iter().find(|c| {
                    c.name == $crate::TypeName::from_iter([$(stringify!($class_part)),+])
                }).expect("No actual class");
                let method = stringify!($method);
                let actual_names: HashSet<_> = class.methods.iter()
                    .filter_map(|m| {
                        if m.role.as_str() == method {
                            Some(fn_wrap!(m.name_c, class.name_collisions.contains(&m.role)).to_string())
                        } else {
                            None
                        }
                    }).collect();

                collection_eq(&expected_names, &actual_names);

                let mut errors = 0;
                for name in expected_names {
                    let r = actual::CPP_MAP.get(name)
                        .unwrap_or_else(|| panic!("No actual {name}"))
                        .diff(
                            expected::CPP_MAP.get(name)
                                .unwrap_or_else(|| panic!("No expected {name}"))
                        );
                    if let Err(e) = r {
                        eprintln!("Error in {name}:\n{e}\n");
                        errors += 1;
                    }
                }

                if errors > 0 {
                    panic!("Mismatch");
                }
            }
        }
    };
);

gen_test!(x::y::MyClass::SubClass, do_something);
gen_test!(x::y::MyClass, new);
gen_test!(x::y::MyClass, destroy);
gen_test!(x::y::MyClass, config);
gen_test!(x::y::MyClass, set_config);
