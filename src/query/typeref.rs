use super::structured_query;

structured_query!{
    name: pub Ref,
    fields: { r: _ },
    query: r#"
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
    "#,
}

structured_query!{
    name: pub Typeref,
    fields: {
        q: Option,
        t: _,
        r: [Ref],
    },
    query: r#"
        (_
            (type_qualifier)? @q
            type: [(primitive_type) (type_identifier) (qualified_identifier)] @t
            declarator: (_) @r
        ) @root
    "#,
}


#[cfg(test)]
mod test {
    use crate::{Context, print_node, query::StructuredMatch};

    use super::*;

    #[test]
    fn simple() {
        let code = "int x = 0;";
        let expected = [Typeref::Literal {
            q: None,
            t: "int",
            r: vec![],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ref1() {
        let code = "Session& x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![Ref::Literal { r: "&" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ref2() {
        let code = "Session&& x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![Ref::Literal { r: "&&" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ptr1() {
        let code = "Session* x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![Ref::Literal { r: "*" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ptr2() {
        let code = "Session** x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![Ref::Literal { r: "*" }, Ref::Literal { r: "*" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }

    #[test]
    fn ptr3() {
        let code = "Session*** x = y;";
        let expected = [Typeref::Literal {
            q: None,
            t: "Session",
            r: vec![Ref::Literal { r: "*" }, Ref::Literal { r: "*" }, Ref::Literal { r: "*" }],
        }];

        let ctx = Context::from_buffer(code).expect("Failed to parse snippet");
        let root = ctx.root();
        print_node(root, 15, 50, true);
        super::super::test::collection_eq(Typeref::Match::from_node(root), expected);
    }
}
