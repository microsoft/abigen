use crate::{AbiConfig, ApiFnArg, ApiFnDef, ApiTyperef, Delimited, RefType};
use std::fmt::{self, Display};

impl ApiFnDef {
    pub fn header(&self) -> impl Display {
        fmt::from_fn(|f| {
            write!(f, "{};", fmt::from_fn(|f| fmt_proto(f, self)))
        })
    }

    pub fn code(&self, config: &AbiConfig) -> impl Display {
        fmt::from_fn(|f| {
            write!(f, "{proto}\n{body}",
                proto = fmt::from_fn(|f| fmt_proto(f, self)),
                body = fmt::from_fn(|f| fmt_body(f, self, config)),
            )
        })
    }
}

const PRELUDE: &'static str = "#ifdef __cplusplus extern \"C\" #endif";

fn fmt_proto(f: &mut std::fmt::Formatter<'_>, item: &ApiFnDef) -> std::fmt::Result {
    let args: Delimited<',', _> = item.args.iter()
        .map(|a| {
            fmt::from_fn(|f| {
                fmt_arg(f, a)
            })
        }).collect();

    write!(f, "{PRELUDE} {ret} {name}({args})",
        ret = fmt::from_fn(|f| fmt_typ(f, &item.ret)),
        name = item.name,
    )
}

fn fmt_arg(f: &mut std::fmt::Formatter<'_>, item: &ApiFnArg) -> std::fmt::Result {
    write!(f, "{typ} {name}",
        typ = fmt::from_fn(|f| fmt_typ(f, &item.typ)),
        name = item.name,
    )
}

fn fmt_typ(f: &mut std::fmt::Formatter<'_>, item: &ApiTyperef) -> std::fmt::Result {
    write!(f, "{corst} {name} {rel}",
        corst = fmt::from_fn(|f| if item.is_const { write!(f, "const") } else { Ok(()) }),
        name = item.typ,
        rel = fmt::from_fn(|f| {
            match item.relation {
                RefType::OwnedReference => write!(f, "&&"),
                RefType::Reference => write!(f, "&"),
                RefType::Value => Ok(()),
                RefType::Pointer(depth) => {
                    for _ in 0..depth {
                        write!(f, "*")?;
                    }
                    Ok(())
                }
            }
        }),
    )
}

fn fmt_body(f: &mut std::fmt::Formatter<'_>, item: &ApiFnDef, config: &AbiConfig) -> std::fmt::Result {
    let nullchecks: Delimited<'\n', _> = item.args.iter()
        .map(|a| {
            fmt::from_fn(|f| {
                fmt_body_nullcheck(f, a, config)
            })
        }).collect();

    writeln!(f, r#"
        {{
            {nullchecks}
            {exec}
            return {success};
        }}"#,
        exec = fmt::from_fn(|f| {
            if item.noexcept {
                fmt_body_exec(f, item, config)
            } else {
                fmt_body_try_exec(f, item, config)
            }
        }),
        success = config.exception.success_value,
    )
}

fn fmt_body_nullcheck(f: &mut std::fmt::Formatter<'_>, item: &ApiFnArg, config: &AbiConfig) -> std::fmt::Result {
    if let RefType::Pointer(depth) = item.typ.relation {
        for i in 0..depth {
            write!(f, "if (")?;
            for _ in 0..i {
                write!(f, "*")?;
            }
            writeln!(f, "{name} == nullptr) {{ return {nullref_exception}; }}",
                name = item.name,
                nullref_exception = config.exception.nullref_value,
            )?;
        }
    }
    Ok(())
}

fn fmt_body_try_exec(f: &mut std::fmt::Formatter<'_>, item: &ApiFnDef, config: &AbiConfig) -> std::fmt::Result {
    let custom_exceptions: Delimited<'\n', _> = config.exception.custom_values.iter()
        .map(|(err, ret)| {
            fmt::from_fn(move |f| {
                write!(f, "catch ({err}) {{ return {ret}; }}")
            })
        }).collect();

    write!(f, r#"
        try {{ {exec} }}
        {custom_exceptions}
        catch (...) {{ return {generic_exception}; }}
    "#,
        exec = fmt::from_fn(|f| fmt_body_exec(f, item, config)),
        generic_exception = config.exception.generic_value,
    )
}

fn fmt_body_exec(f: &mut std::fmt::Formatter<'_>, item: &ApiFnDef, config: &AbiConfig) -> std::fmt::Result {
    if item.args[item.args.len()-1].name == "out" {
        // no "this", no "out"
        let other_args: Delimited<',', _> = item.args[1..item.args.len()-1].iter()
            .map(|a| {
                fmt::from_fn(|f| {
                    write!(f, "{}", a.name)
                })
            })
            .collect();

        write!(f, "*out = nullptr; *out = this->{name}({other_args});",
            name = item.name,
        )
    } else {
        // no "this"
        let args: Delimited<',', _> = item.args[1..].iter()
            .map(|a| {
                fmt::from_fn(|f| {
                    write!(f, "{}", a.name)
                })
            })
            .collect();

        write!(f, "this->{name}({args});",
            name = item.name,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Context, query::FnDef};

    #[test]
    fn header() {
        let ctx = Context::from_buffer(crate::test_util::CODE).expect("Failed to parse snippet");
        let root = ctx.root();
        let defs: Vec<_> = FnDef::Match::from_node(root)
            .into_iter()
            .map(|q| {
                ApiFnDef::try_from(q)
                    .expect("Failed to convert")
                    .to_c(&crate::test_util::CONFIG)
                    .header()
                    .to_string()
            })
            .collect();
        crate::test_util::collection_eq(
            defs,
            [
                "#ifdef __cplusplus extern \"C\" #endif Result myclass_config(const MyClass* this, Config** out);",
                "#ifdef __cplusplus extern \"C\" #endif Result myclass_set_config(MyClass* this, const Config* config);",
            ],
        );
    }

    #[test]
    fn code_simple() {
        todo!()
    }
}
