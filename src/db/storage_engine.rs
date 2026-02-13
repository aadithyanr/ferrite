use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::db::schema::{Table, Row};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct StorageEngine {
    tables: HashMap<String, Table>
}

impl StorageEngine {
    pub fn new() -> Self {
        StorageEngine {
            tables: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<String>) {
        self.tables.insert(
            name.to_string(),
            Table {
                columns, 
                rows: HashMap::new(),
            }
        );
    }

    pub fn insert_row(&mut self, table_name: &str, row: Row) -> Result<usize, String> {
        if let Some(table) = self.tables.get_mut(table_name) {
            let row_id = table.rows.len();
            table.rows.insert(row_id, row);
            Ok(row_id)
        } else {
            Err(format!("Table {} not found", table_name))
        }
    }

    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<(), String> {
        buffer.clear();
        let serialized = bincode::encode_to_vec(&self.tables, bincode::config::standard())
            .map_err(|e| format!("Serialization error: {:?}", e))?;
        buffer.extend_from_slice(&serialized);
        Ok(())
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, String> {
        let (tables, _) = bincode::decode_from_slice(buffer, bincode::config::standard())
            .map_err(|e| format!("Deserialization error: {:?}", e))?;
        Ok(StorageEngine { tables })
    }

    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }
}

// saving/loading the database to/from disk
pub struct FileSystem {
    pub storage_engine: StorageEngine,
    file_path: PathBuf
}

impl FileSystem {
    pub fn new(file_path: &str) -> Result<Self, String> {
        let mut storage_engine = StorageEngine::new();
        
        // file exists, load existing database
        if Path::new(file_path).exists() {
            storage_engine = Self::load_from_file(file_path)
                .map_err(|e| format!("Failed to load database: {:?}", e))?;
        }
        
        Ok(FileSystem {
            storage_engine,
            file_path: PathBuf::from(file_path)
        })
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<String>) -> Result<(), String> {
        self.storage_engine.create_table(name, columns);
        self.save()
    }

    pub fn insert_row(&mut self, table_name: &str, row: Row) -> Result<usize, String> {
        let row_id = self.storage_engine.insert_row(table_name, row)?;
        self.save()?;
        Ok(row_id)
    }

    fn save(&self) -> Result<(), String> {
        let file = File::create(&self.file_path)
            .map_err(|e| format!("Failed to create file: {:?}", e))?;
        let mut writer = BufWriter::new(file);
        let mut buffer = Vec::new();
        
        self.storage_engine.serialize(&mut buffer)?;
        writer.write_all(&buffer)
            .map_err(|e| format!("Failed to write to file: {:?}", e))?;
        
        Ok(())
    }

    fn load_from_file(file_path: &str) -> Result<StorageEngine, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open file: {:?}", e))?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        
        reader.read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file: {:?}", e))?;
        
        StorageEngine::deserialize(&buffer)
    }
}