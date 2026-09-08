use crate::query::structured_query;

structured_query! {
    name: pub Template,
    fields: {
        start: _,
        end: _,
    },
    query: r#"
        (translation_unit
            (comment)
            (_)*
            (comment)
        )
    "#,
}
