use serde::{Serialize, Deserialize};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter, BufReader, Read};
use std::path::PathBuf;
use crate::db::schema::Row;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WALEntry {
    CreateTable {
        name: String,
        columns: Vec<String>,
    },
    InsertRow {
        table: String,
        row_id: usize,
        row: Row,
    },
    Checkpoint,
}

pub struct WAL {
    file_path: PathBuf,
    writer: BufWriter<File>,
    entry_count: usize,
    checkpoint_interval: usize,
}

impl WAL {
    pub fn new(file_path: &str, checkpoint_interval: usize) -> Result<Self, String> {
        let path = PathBuf::from(file_path);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| format!("failed to open wal: {:?}", e))?;

        Ok(WAL {
            file_path: path,
            writer: BufWriter::new(file),
            entry_count: 0,
            checkpoint_interval,
        })
    }

    pub fn append(&mut self, entry: WALEntry) -> Result<(), String> {
        let encoded = bincode::serialize(&entry)
            .map_err(|e| format!("wal encode error: {:?}", e))?;

        let len = encoded.len() as u32;
        self.writer.write_all(&len.to_le_bytes())
            .map_err(|e| format!("wal write error: {:?}", e))?;
        self.writer.write_all(&encoded)
            .map_err(|e| format!("wal write error: {:?}", e))?;
        
        self.writer.flush()
            .map_err(|e| format!("wal flush error: {:?}", e))?;

        self.entry_count += 1;
        Ok(())
    }

    pub fn should_checkpoint(&self) -> bool {
        self.entry_count >= self.checkpoint_interval
    }

    pub fn replay(&self) -> Result<Vec<WALEntry>, String> {
        let file = File::open(&self.file_path)
            .map_err(|e| format!("failed to open wal for replay: {:?}", e))?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();

        loop {
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => {},
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(format!("wal read error: {:?}", e)),
            }

            let len = u32::from_le_bytes(len_bytes) as usize;
            let mut buffer = vec![0u8; len];
            reader.read_exact(&mut buffer)
                .map_err(|e| format!("wal read error: {:?}", e))?;

            let entry: WALEntry = bincode::deserialize(&buffer)
                .map_err(|e| format!("wal decode error: {:?}", e))?;

            entries.push(entry);
        }

        Ok(entries)
    }

    pub fn truncate(&mut self) -> Result<(), String> {
        self.writer.flush()
            .map_err(|e| format!("wal flush error: {:?}", e))?;
        
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.file_path)
            .map_err(|e| format!("failed to truncate wal: {:?}", e))?;

        self.writer = BufWriter::new(file);
        self.entry_count = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::Row;

    #[test]
    fn test_wal_append_and_replay() {
        let wal_path = "/tmp/test.wal";
        let _ = std::fs::remove_file(wal_path);

        let mut wal = WAL::new(wal_path, 10).unwrap();
        
        let entry = WALEntry::CreateTable {
            name: "test".to_string(),
            columns: vec!["id".to_string()],
        };
        
        wal.append(entry.clone()).unwrap();

        let entries = wal.replay().unwrap();
        assert_eq!(entries.len(), 1);
        
        std::fs::remove_file(wal_path).unwrap();
    }
}