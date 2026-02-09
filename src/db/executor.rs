use crate::db::storage_engine::StorageEngine;
use crate::db::schema::Table;
use std::collections::HashMap;
use crate::db::query::{QueryPlan, Identifier};

pub struct ExecutionEngine {
    storage_engine: StorageEngine,
}

#[derive(Debug)]
pub enum ExecutionError {
    TableNotFound(String),
}

impl ExecutionEngine {
    pub fn new(storage_engine: StorageEngine) -> Self {
        ExecutionEngine { storage_engine }
    }

    pub fn execute(&self, query_plan: QueryPlan) -> Result<Vec<HashMap<String, String>>, ExecutionError> {
        // from storage engine
        let table = self.storage_engine
            .get_table(&query_plan.table)
            .ok_or(ExecutionError::TableNotFound(query_plan.table.clone()))?;

        let mut results: Vec<HashMap<String, String>> = Vec::new();

        for row in table.rows.values() {
            let mut result: HashMap<String, String> = HashMap::new();
            
            // (projection)
            for identifier in &query_plan.projection {
                result.insert(
                    identifier.0.clone(),
                    row.data.get(&identifier.0).unwrap_or(&"".to_string()).clone()
                );
            }
            results.push(result);
        }
        
        Ok(results)
    }
}