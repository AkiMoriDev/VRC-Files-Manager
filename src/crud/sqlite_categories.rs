use rusqlite::{params, Connection};
use anyhow::Result;

pub struct CategoryStore {
  conn: Connection,
}

impl CategoryStore {
  pub fn new(conn: Connection) -> Self {
    Self { conn }
  }

  pub fn init(&self) -> Result<()> {
    self.conn.execute_batch(
      r#"
        CREATE TABLE IF NOT EXISTS categories (name TEXT PRIMARY KEY);
      "#,
    )?;
    Ok(())
  }

  pub fn insert_category(&self, name: &str) -> Result<()> {
    self.conn.execute(
      "INSERT OR IGNORE INTO categories (name) VALUES (?1)",
      params![name],
    )?;
    Ok(())
  }

  pub fn update_category(&self, old_name: &str, new_name: &str) -> Result<()> {
    self.conn.execute(
      "UPDATE categories SET name = ?1 WHERE name = ?2",
      params![new_name, old_name],
    )?;
    Ok(())
  }

  pub fn delete_category(&self, name: &str) -> Result<()> {
    self.conn.execute("DELETE FROM categories WHERE name = ?1",
      params![name],
    )?;
    Ok(())
  }

  pub fn get_categories(&self) -> Result<Vec<String>> {
    let mut stmt = self.conn.prepare("SELECT name FROM categories ORDER BY name")?;
    let rows = stmt.query_map([], |r| Ok(r.get::<_, String>(0)?))?;

    let mut v = Vec::new();
    for r in rows {
      v.push(r?);
    }
    Ok(v)
  }
}