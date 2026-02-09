#[derive(Debug, Clone)]
pub struct Identifier(pub String);  

#[derive(Debug)]
pub struct QueryPlan {
    pub table: String,  
    pub projection: Vec<Identifier>  
}