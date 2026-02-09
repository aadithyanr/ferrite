use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Table {
    pub columns: Vec<String>,
    pub rows: HashMap<usize, Row>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct Row {
    pub data: HashMap<String, String>  // public?
}

impl Row {
    pub fn new() -> Self {
        Row { data: HashMap::new() }
    }
    
    pub fn set(&mut self, column: String, value: String) {
        self.data.insert(column, value);
    }
}