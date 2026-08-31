use super::structured_query;

structured_query!(pub TYPEREF_QUERY / TyperefQuery / TyperefMatch / TyperefLitrl {
    q: Option,
    t: _,
    r: [RefQuery],
}, r#"
    (_
        (type_qualifier)? @q
        type: [(primitive_type) (type_identifier) (qualified_identifier)] @t
        declarator: (_) @r
    ) @root
"#);

structured_query!(pub REF_QUERY / RefQuery / RefMatch / RefLitrl {
    r: _
}, r#"
    [
        (reference_declarator
            [("&") ("&&")] @r
            (identifier)
        )
        (pointer_declarator
            ("*") @r
            declarator: [(pointer_declarator) (identifier)]
        )
    ] @root
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
            r: vec![],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(TyperefMatch::from_node(root), expected);
    }

    #[test]
    fn ref1() {
        let code = "Session& x = y;";
        let expected = [TyperefLitrl {
            q: None,
            t: "Session",
            r: vec![RefLitrl { r: "&" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(TyperefMatch::from_node(root), expected);
    }

    #[test]
    fn ref2() {
        let code = "Session&& x = y;";
        let expected = [TyperefLitrl {
            q: None,
            t: "Session",
            r: vec![RefLitrl { r: "&&" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(TyperefMatch::from_node(root), expected);
    }

    #[test]
    fn ptr1() {
        let code = "Session* x = y;";
        let expected = [TyperefLitrl {
            q: None,
            t: "Session",
            r: vec![RefLitrl { r: "*" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(TyperefMatch::from_node(root), expected);
    }

    #[test]
    fn ptr2() {
        let code = "Session** x = y;";
        let expected = [TyperefLitrl {
            q: None,
            t: "Session",
            r: vec![RefLitrl { r: "*" }, RefLitrl { r: "*" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(TyperefMatch::from_node(root), expected);
    }

    #[test]
    fn ptr3() {
        let code = "Session*** x = y;";
        let expected = [TyperefLitrl {
            q: None,
            t: "Session",
            r: vec![RefLitrl { r: "*" }, RefLitrl { r: "*" }, RefLitrl { r: "*" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(TyperefMatch::from_node(root), expected);
    }
}
