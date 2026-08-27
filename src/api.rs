use std::path::Path;

use crate::{Context, ContextError, query};

pub struct ApiHeader {
    pub functions: Vec<ApiFnDef>,
}

impl ApiHeader {
    pub fn from_file(file: &Path) -> Result<Self, ContextError> {
        todo!()
    }
}

pub struct ApiFnDef {}

impl From<query::fn_def::FnDefMatch<'_>> for ApiFnDef {
    fn from(value: query::fn_def::FnDefMatch) -> Self {
        todo!()
    }
}

pub struct ApiFnArg {}

impl From<query::fn_arg::FnArgMatch<'_>> for ApiFnArg {
    fn from(value: query::fn_arg::FnArgMatch) -> Self {
        todo!()
    }
}

pub struct ApiTyperef {
    q: Option<String>,
    t: String,
    r: Option<String>,
}

impl From<query::typeref::TyperefMatch<'_>> for ApiTyperef {
    fn from(value: query::typeref::TyperefMatch<'_>) -> Self {
        todo!()
    }
}
