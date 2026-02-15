#[derive(Debug, Clone)]
pub struct Identifier(pub String);

#[derive(Debug, Clone)]
pub struct Filter {
    pub column: String,
    pub value: String,
}

#[derive(Debug)]
pub struct QueryPlan {
    pub table: String,
    pub projection: Vec<Identifier>,
    pub filter: Option<Filter>,
}