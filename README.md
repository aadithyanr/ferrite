<img src="assets/logo.svg" alt="rust_db" width="50" />

# ferrite

a minimal database written in rust from scratch

## features

- b-tree indexing for fast lookups
- write-ahead logging for crash recovery
- sql parser (select, where clauses)
- binary serialization with bincode
- persistent storage

## demo

https://github.com/aadithyanr/attachments/assets/demo.mov

## how it works

**b-tree indexes**
- o(log n) lookups vs o(n) table scans
- automatically used by executor when available
- supports range scans

**write-ahead log**
- all writes logged before applying
- replays on startup for crash recovery
- checkpoints every 100 operations
- prevents data corruption

**storage**
- tables stored as hashmap<row_id, row>
- binary serialization with bincode
- indexes stored alongside tables

## architecture

```
query → parser → ast → query plan → executor → storage
                                        ↓
                                   btree index
                                        ↓
                                   wal → disk
```