#[derive(Debug, Clone, Default)]
pub struct ApiFn {
    pub name: String,
    pub ns: Vec<String>,
    pub ret: String,
    pub args: Vec<ApiFnArg>,
}

#[derive(Debug, Clone, Default)]
pub struct ApiFnArg {
    pub typ: String,
    pub name: String,
}
