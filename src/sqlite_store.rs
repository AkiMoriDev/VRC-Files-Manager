use super::models::IndexedFile;
use anyhow::Result;
use rusqlite::{params, Connection};

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]'
            );
            "#,
        )?;
        Ok(Self { conn })
    }

    pub fn insert_file(&mut self, f: &IndexedFile) -> Result<()> {
        let tags_json = serde_json::to_string(&f.tags)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO files (path, name, tags) VALUES (?1, ?2, ?3)",
            params![f.path, f.name, tags_json],
        )?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<Vec<IndexedFile>> {
        let mut stmt = self.conn.prepare("SELECT path, name, tags FROM files WHERE name LIKE ?1")?;
        let rows = stmt.query_map([format!("%{}%", query)], |r| {
            let tags_json: String = r.get(2)?;
            let tags = serde_json::from_str(&tags_json).unwrap_or_default();
            Ok(IndexedFile {
                path: r.get(0)?,
                name: r.get(1)?,
                tags,
            })
        })?;
        
        let mut v = Vec::new();
        for r in rows { v.push(r?); }
        Ok(v)
    }
}