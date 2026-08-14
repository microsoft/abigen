use std::sync::LazyLock;
use tree_sitter::Query;

const FN_DEF_STR: &'static str = r#"
    (function_declarator
        declarator: (identifier) @fn_name
    )
"#;

pub static FN_DEF_QUERY: LazyLock<Query> = LazyLock::new(|| {
    Query::new(&crate::LANG, FN_DEF_STR).expect("Failed to compile fn_def")
});