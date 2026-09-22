use crate::query::structured_query;

pub mod k {
    macro_rules! kind_id (
        ($vis:vis $name:ident, $kind:literal) => {
            $vis static $name: std::sync::LazyLock<u16> = std::sync::LazyLock::new(|| $crate::LANG.id_for_node_kind($kind, true));
        }
    );
    kind_id!(pub CLASS, "class_specifier");
    kind_id!(pub NS, "namespace_definition");
    kind_id!(pub NESTED_NS, "nested_namespace_specifier");
    kind_id!(pub QUALIFIED_ID, "qualified_identifier");
    kind_id!(pub NS_ID, "namespace_identifier");
    kind_id!(pub TYPE_ID, "type_identifier");
}

pub mod f {
    macro_rules! field_id (
        ($vis:vis $name:ident, $field:literal) => {
            $vis static $name: std::sync::LazyLock<u16> = std::sync::LazyLock::new(|| $crate::LANG.field_id_for_name($field).unwrap().into());
        }
    );
    field_id!(pub SCOPE, "scope");
    field_id!(pub NAME, "name");
}

structured_query! {
    name: pub ClassDef,
    fields: {
        name: _,
        -
        ctor: [=FnDecl],
        methods: [=FnDef],
    },
    query: r#"
        (class_specifier
            name: (type_identifier) @name
            body: (field_declaration_list (
                ((access_specifier) @variable.builtin (#eq? @variable.builtin "public"))
                .
                [(comment) (declaration) (field_declaration)]*
                .
                [
                    (field_declaration) @methods
                    (declaration
                        declarator: (function_declarator) @ctor
                    )
                ]
            ))
        )
    "#,
}

structured_query! {
    name: pub FnDef,
    fields: {
        ret_qual: ?,
        ret_typ: _,
        ret_refs: [FnRetRef],
        decl: (FnDecl),
    },
    query: r#"
        (field_declaration
            (type_qualifier ("const"))? @ret_qual
            type: [(primitive_type) (type_identifier) (qualified_identifier) (template_type)] @ret_typ
            declarator: (_) @ret_refs @decl
        )
    "#
}

structured_query! {
    name: pub FnRetRef,
    fields: {
        r: _,
    },
    query: r#"
        [
            (reference_declarator
                [("&") ("&&")] @r
                (function_declarator)
            )
            (pointer_declarator
                ("*") @r
                declarator: [(function_declarator) (pointer_declarator)]
            )
        ]
    "#,
}

structured_query! {
    name: pub FnDecl,
    fields: {
        name: _,
        args: [=FnArg],
        immut: ?,
        noexcept: ?,
    },
    query: r#"
        (function_declarator
            declarator: (field_identifier) @name
            parameters: (parameter_list
                (parameter_declaration)* @args
            )
            (type_qualifier ("const") @immut)?
            (noexcept ("noexcept") @noexcept)?
        )
    "#
}

structured_query! {
    name: pub FnArg,
    fields: {
        typ_qual: ?,
        typ: _,
        typ_refs: [FnArgRef],
        name: _,
    },
    query: r#"
        (parameter_declaration
            (type_qualifier ("const"))? @typ_qual
            type: [(primitive_type) (type_identifier) (qualified_identifier) (template_type)] @typ
            declarator: [
                (identifier) @name
                (reference_declarator
                    (identifier) @name
                )
            ] @typ_refs
        )
    "#
}

structured_query! {
    name: pub FnArgRef,
    fields: {
        r: _,
    },
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
        ]
    "#,
}
