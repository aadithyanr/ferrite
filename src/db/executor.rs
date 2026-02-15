use crate::db::storage_engine::StorageEngine;
use crate::db::schema::Table;
use std::collections::HashMap;
use crate::db::query::{QueryPlan, Identifier, Filter};

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
        let table = self.storage_engine
            .get_table(&query_plan.table)
            .ok_or(ExecutionError::TableNotFound(query_plan.table.clone()))?;

        let row_ids = if let Some(filter) = &query_plan.filter {
            // try to use index
            if let Some(ids) = self.storage_engine.search_index(
                &query_plan.table,
                &filter.column,
                &filter.value
            ) {
                ids
            } else {
                // fallback to table scan
                self.table_scan_with_filter(table, filter)
            }
        } else {
            // no filter, scan all
            table.rows.keys().copied().collect()
        };

        let mut results: Vec<HashMap<String, String>> = Vec::new();
        
        for row_id in row_ids {
            if let Some(row) = table.rows.get(&row_id) {
                let mut result: HashMap<String, String> = HashMap::new();
                
                for identifier in &query_plan.projection {
                    result.insert(
                        identifier.0.clone(),
                        row.data.get(&identifier.0).unwrap_or(&"".to_string()).clone()
                    );
                }
                
                results.push(result);
            }
        }

        Ok(results)
    }

    fn table_scan_with_filter(&self, table: &Table, filter: &Filter) -> Vec<usize> {
        table.rows
            .iter()
            .filter(|(_, row)| {
                row.data.get(&filter.column)
                    .map(|v| v == &filter.value)
                    .unwrap_or(false)
            })
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn explain(&self, query_plan: &QueryPlan) -> String {
        let mut plan = String::from("query plan:\n");
        
        if let Some(filter) = &query_plan.filter {
            if self.storage_engine.search_index(&query_plan.table, &filter.column, &filter.value).is_some() {
                plan.push_str(&format!("  - index scan: {}.{}\n", query_plan.table, filter.column));
                plan.push_str("  - estimated cost: 1 row\n");
            } else {
                plan.push_str(&format!("  - table scan: {}\n", query_plan.table));
                plan.push_str("  - estimated cost: unknown\n");
            }
        } else {
            plan.push_str(&format!("  - table scan: {}\n", query_plan.table));
        }
        
        plan
    }
}