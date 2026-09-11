use crate::query::structured_query;

structured_query! {
    name: pub Template,
    fields: {
        start: TemplateStart,
        end: TemplateEnd,
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
        let m = Template::Match::from_node(crate::test::TEMPLATE_H_CTX.root());
        let m = if m.len() == 1 {
            m.into_iter().next().unwrap()
        } else {
            panic!("Too many matches!");
        };

        assert_eq!(Template::Literal {
            start: TemplateStart::Literal {
                f: "/** Generated header - DO NOT MODIFY\r\n * @abigen:start\r\n */",
            },
            end: TemplateEnd::Literal {
                f: "// @abigen:end\r",
            },
        }, m);
    }
}