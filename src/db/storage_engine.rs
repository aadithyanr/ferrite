use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::db::schema::{Table, Row};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct StorageEngine {
    tables: HashMap<String, Table>
}

impl StorageEngine {
    pub fn new() -> Self {
        StorageEngine {  // Use curly braces, not parentheses
            tables: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<String>) {
        self.tables.insert(
            name.to_string(),  // Remove "k:"
            Table {  // Use curly braces
                columns, 
                rows: HashMap::new(),
            }
        );
    }

    pub fn insert_row(&mut self, table_name: &str, row: Row) -> Result<usize, String> {  // Fixed "fc" typo and return type
        if let Some(table) = self.tables.get_mut(table_name) {
            let row_id = table.rows.len();
            table.rows.insert(row_id, row);
            Ok(row_id)
        } else {
            Err(format!("Table {} not found", table_name))
        }
    }

    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<(), String> {  // Return Result
        buffer.clear();
        let serialized = bincode::encode_to_vec(&self.tables, bincode::config::standard())
            .map_err(|e| format!("Serialization error: {:?}", e))?;
        buffer.extend_from_slice(&serialized);
        Ok(())
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, String> {  // Takes &[u8], not &mut
        let (tables, _) = bincode::decode_from_slice(buffer, bincode::config::standard())
            .map_err(|e| format!("Deserialization error: {:?}", e))?;
        Ok(StorageEngine { tables })
    }
}