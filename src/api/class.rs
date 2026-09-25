use crate::{
    AbiConfig, ApiFnDef, ApiFnRole, TypeName, query::{self, f, k},
};
use std::{cmp::Ordering, collections::{HashMap, HashSet}, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiClassDef {
    pub name: TypeName,
    pub methods: HashSet<ApiFnDef>,
    pub name_collisions: HashSet<ApiFnRole>,
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

        let mut methods: Vec<_> = value.ctor.iter()
            .map(ApiFnDef::from)
            .chain(value.methods.iter().map(ApiFnDef::from))
            .collect();

        if value.ctor.len() > 0 {
            methods.push(ApiFnDef {
                role: ApiFnRole::Destructor,
                noexcept: true,
                ..Default::default()
            });
        }

        Self::new(fqns, methods)
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

impl ApiClassDef {
    pub fn new(name: impl Into<TypeName>, methods: impl IntoIterator<Item=ApiFnDef>) -> Self {
        let mut s = Self {
            name: name.into(),
            methods: HashSet::new(),
            name_collisions: HashSet::new(),
        };

        for mut m in methods {
            m.set_implicit_fields(&s.name);
            s.methods.insert(m);
        }
        s.check_name_collisions();

        s
    }

    pub(crate) fn check_name_collisions(&mut self) {
        let mut counts = HashMap::new();
        for method in &self.methods {
            *counts.entry(&method.role).or_insert(0u32) += 1;
        }
        for (name, count) in counts {
            if self.name_collisions.contains(name) && count < 2 {
                self.name_collisions.remove(name);
            } else if count >= 2 {
                self.name_collisions.insert(name.clone());
            }
        }
    }

    pub(crate) fn opaque_struct(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "typedef struct ")?;
        self.name.pascal_case(f)?;
        write!(f, " ")?;
        self.name.pascal_case(f)?;
        write!(f, ";")
    }

    pub(crate) fn header_methods(&self, f: &mut fmt::Formatter, config: &AbiConfig) -> fmt::Result {
        let mut methods: Vec<_> = self.methods.iter().collect();
        methods.sort();

        for method in methods {
            method.header(f, config, self.name_collisions.contains(&method.role))?;
            writeln!(f)?;
        }

        Ok(())
    }

    pub(crate) fn code_methods(&self, f: &mut fmt::Formatter, config: &AbiConfig) -> fmt::Result {
        let mut methods: Vec<_> = self.methods.iter().collect();
        methods.sort();

        for method in methods {
            method.code(f, config, self.name_collisions.contains(&method.role))?;
            writeln!(f)?;
        }

        Ok(())
    }
}