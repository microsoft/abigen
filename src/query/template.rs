use crate::query::structured_query;

structured_query! {
    name: pub Template,
    fields: {
        start: (TemplateStart),
        end: (TemplateEnd),
    },
    query: r#"
        (translation_unit) @start @end
    "#,
}

structured_query! {
    name: pub TemplateStart,
    fields: {
        f: _,
    },
    query: r#"
        ((comment) @c (#match? @c "@abigen:start")) @f
    "#,
}

structured_query! {
    name: pub TemplateEnd,
    fields: {
        f: _,
    },
    query: r#"
        ((comment) @c (#match? @c "@abigen:end")) @f
    "#,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn header() {
        let m = Template::Match::from_node(crate::test::input::TEMPLATE_H_CTX.root());
        let m = if m.len() == 1 {
            m.into_iter().next().unwrap()
        } else {
            panic!("Too many matches!");
        };

        assert!(m.start.f.as_str().contains("@abigen:start"));
        assert!(m.end.f.as_str().contains("@abigen:end"));
    }
}
