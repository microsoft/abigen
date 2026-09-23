use crate::{
    ApiFnDef, ApiFnRole, TypeName, query::{self, f, k},
};
use std::{cmp::Ordering, collections::HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiClassDef {
    pub name: TypeName,
    pub methods: HashSet<ApiFnDef>,
}

impl From<&query::ClassDef::Match<'_>> for ApiClassDef {
    fn from(value: &query::ClassDef::Match) -> Self {
        let mut fqns = TypeName::from_iter([value.name.as_str().to_string()]);
        let mut parent = value.name.parent().unwrap();
        while let Some(p) = parent.parent() {
            parent = p;
            if parent.node.kind_id() == *k::CLASS
                && let Some(name) = parent.child_by_field_id(*f::NAME)
            {
                // class definition
                fqns.insert(0, name.as_str().to_string());
            } else if parent.node.kind_id() == *k::NS
                && let Some(name) = parent.child_by_field_id(*f::NAME)
            {
                // namespace definition
                if name.node.kind_id() == *k::NESTED_NS {
                    // multi-part namespace name, i.e. x::y
                    for (i, part) in name.named_children(&mut name.walk()).enumerate() {
                        fqns.insert(i, part.as_str().to_string());
                    }
                } else {
                    // single-part namespace name
                    fqns.insert(0, name.as_str().to_string());
                }
            }
        }

        let mut methods: HashSet<_> = value.ctor.iter()
            .map(ApiFnDef::from)
            .chain(value.methods.iter().map(ApiFnDef::from))
            .collect();

        if value.ctor.len() > 0 {
            methods.insert(ApiFnDef {
                role: ApiFnRole::Destructor,
                args: vec![],
                noexcept: true,
                is_const: false,
            });
        }
        
        Self {
            name: fqns,
            methods,
        }
    }
}

impl Ord for ApiClassDef {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for ApiClassDef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for ApiClassDef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
