use super::structured_query;

structured_query!(pub TYPEREF_QUERY / TyperefQuery / TyperefMatch / TyperefLitrl {
    q: Option,
    t: _,
    r: Option,
}, r#"
    (_
        (type_qualifier)? @q
        type: (_) @t
        declarator: [
            (_
                ("&") @r
            )
            (_
                (_
                    ("&") @r
                )
            )
        ]?
    ) @root
"#);

#[cfg(test)]
mod test {
    use crate::{Context, print_node, query::StructuredMatch};

    use super::*;

    #[test]
    fn simple() {
        let code = "int x = 0;";
        let expected = [TyperefLitrl {
            q: None,
            t: "int",
            r: None,
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        super::super::test::collection_eq(TyperefMatch::from_node(root), expected);
    }

    #[test]
    fn complex() {
        let code = "const Session& x = y;";
        let expected = [TyperefLitrl {
            q: Some("const"),
            t: "Session",
            r: Some("&"),
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(TyperefMatch::from_node(root), expected);
    }
}
