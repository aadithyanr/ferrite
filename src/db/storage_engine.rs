use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::db::schema::{Table, Row};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use crate::db::btree::BTreeIndex;
use crate::db::wal::{WAL, WALEntry};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageEngine {
    tables: HashMap<String, Table>,
    indexes: HashMap<String, HashMap<String, BTreeIndex>>, // table -> column -> index
}

impl StorageEngine {
    pub fn new() -> Self {
        StorageEngine {
            tables: HashMap::new(),
            indexes: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<String>) {
        self.tables.insert(
            name.to_string(),
            Table {
                columns: columns.clone(), 
                rows: HashMap::new(),
            }
        );
        self.indexes.insert(name.to_string(), HashMap::new());
    }

    pub fn create_index(&mut self, table_name: &str, column: &str) -> Result<(), String> {
        if !self.tables.contains_key(table_name) {
            return Err(format!("table {} not found", table_name));
        }

        let mut btree = BTreeIndex::new(4);
        
        // build index from existing data
        if let Some(table) = self.tables.get(table_name) {
            for (row_id, row) in &table.rows {
                if let Some(value) = row.data.get(column) {
                    btree.insert(value.clone(), *row_id);
                }
            }
        }

        self.indexes
            .entry(table_name.to_string())
            .or_insert_with(HashMap::new)
            .insert(column.to_string(), btree);

        Ok(())
    }

    pub fn insert_row(&mut self, table_name: &str, row: Row) -> Result<usize, String> {
        if let Some(table) = self.tables.get_mut(table_name) {
            let row_id = table.rows.len();
            
            // update indexes for this table
            if let Some(table_indexes) = self.indexes.get_mut(table_name) {
                for (column, index) in table_indexes.iter_mut() {
                    if let Some(value) = row.data.get(column) {
                        index.insert(value.clone(), row_id);
                    }
                }
            }
            
            table.rows.insert(row_id, row);
            Ok(row_id)
        } else {
            Err(format!("table {} not found", table_name))
        }
    }

    pub fn search_index(&self, table_name: &str, column: &str, key: &str) -> Option<Vec<usize>> {
        self.indexes
            .get(table_name)?
            .get(column)?
            .search(key)
            .map(|row_id| vec![row_id])
    }

    pub fn serialize(&self, buffer: &mut Vec<u8>) -> Result<(), String> {
        buffer.clear();
        
        #[derive(Serialize, Deserialize)]
        struct SerializedData {
            tables: HashMap<String, Table>,
            indexes: HashMap<String, HashMap<String, BTreeIndex>>,
        }
        
        let data = SerializedData {
            tables: self.tables.clone(),
            indexes: self.indexes.clone(),
        };
        
        let serialized = bincode::encode_to_vec(&data, bincode::config::standard())
            .map_err(|e| format!("serialization error: {:?}", e))?;
        buffer.extend_from_slice(&serialized);
        Ok(())
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self, String> {
        #[derive(Serialize, Deserialize)]
        struct SerializedData {
            tables: HashMap<String, Table>,
            indexes: HashMap<String, HashMap<String, BTreeIndex>>,
        }
        
        let (data, _): (SerializedData, _) = bincode::decode_from_slice(buffer, bincode::config::standard())
            .map_err(|e| format!("deserialization error: {:?}", e))?;
        
        Ok(StorageEngine { 
            tables: data.tables,
            indexes: data.indexes,
        })
    }

    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }

    pub fn apply_wal_entry(&mut self, entry: WALEntry) -> Result<(), String> {
        match entry {
            WALEntry::CreateTable { name, columns } => {
                self.create_table(&name, columns);
                Ok(())
            }
            WALEntry::InsertRow { table, row_id: _, row } => {
                self.insert_row(&table, row)?;
                Ok(())
            }
            WALEntry::Checkpoint => Ok(()),
        }
    }
}

pub struct FileSystem {
    pub storage_engine: StorageEngine,
    file_path: PathBuf,
    wal: WAL,
}

impl FileSystem {
    pub fn new(file_path: &str) -> Result<Self, String> {
        let wal_path = format!("{}.wal", file_path);
        let mut wal = WAL::new(&wal_path, 100)?;
        let mut storage_engine = StorageEngine::new();

        // try to recover from wal first
        if Path::new(&wal_path).exists() {
            let entries = wal.replay()?;
            for entry in entries {
                storage_engine.apply_wal_entry(entry)?;
            }
        }

        // then load main db if it exists
        if Path::new(file_path).exists() {
            storage_engine = Self::load_from_file(file_path)?;
            wal.truncate()?; // clear wal after successful load
        }

        Ok(FileSystem {
            storage_engine,
            file_path: PathBuf::from(file_path),
            wal,
        })
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<String>) -> Result<(), String> {
        let entry = WALEntry::CreateTable {
            name: name.to_string(),
            columns: columns.clone(),
        };
        
        self.wal.append(entry)?;
        self.storage_engine.create_table(name, columns);
        
        if self.wal.should_checkpoint() {
            self.checkpoint()?;
        }
        
        Ok(())
    }

    pub fn create_index(&mut self, table_name: &str, column: &str) -> Result<(), String> {
        self.storage_engine.create_index(table_name, column)?;
        self.save()
    }

    pub fn insert_row(&mut self, table_name: &str, row: Row) -> Result<usize, String> {
        let entry = WALEntry::InsertRow {
            table: table_name.to_string(),
            row_id: 0, // actual id assigned by storage engine
            row: row.clone(),
        };
        
        self.wal.append(entry)?;
        let row_id = self.storage_engine.insert_row(table_name, row)?;
        
        if self.wal.should_checkpoint() {
            self.checkpoint()?;
        }
        
        Ok(row_id)
    }

    fn checkpoint(&mut self) -> Result<(), String> {
        self.save()?;
        self.wal.truncate()?;
        Ok(())
    }

    fn save(&self) -> Result<(), String> {
        let file = File::create(&self.file_path)
            .map_err(|e| format!("failed to create file: {:?}", e))?;
        let mut writer = BufWriter::new(file);
        let mut buffer = Vec::new();
        self.storage_engine.serialize(&mut buffer)?;
        writer.write_all(&buffer)
            .map_err(|e| format!("failed to write to file: {:?}", e))?;
        Ok(())
    }

    fn load_from_file(file_path: &str) -> Result<StorageEngine, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("failed to open file: {:?}", e))?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)
            .map_err(|e| format!("failed to read file: {:?}", e))?;
        StorageEngine::deserialize(&buffer)
    }
}