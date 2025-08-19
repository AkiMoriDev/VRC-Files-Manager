use anyhow::Result;
use rusqlite::{params, Connection};

pub struct TagStore {
  conn: Connection,
}

impl TagStore {
  pub fn new(conn: Connection) -> Self {
    Self { conn }
  }

  /// Crea la tabla `tags` si no existe
  pub fn init(&self) -> Result<()> {
    self.conn.execute_batch(
      r#" CREATE TABLE IF NOT EXISTS tags (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT UNIQUE NOT NULL); "#,
    )?;
    Ok(())
  }

  /// INSERT - crear un nuevo tag
  pub fn insert_tag(&self, name: &str) -> Result<()> {
    self.conn.execute(
      "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
      params![name],
    )?;
    Ok(())
  }

  /// SELECT - obtener todos los tags
  pub fn get_tags(&self) -> Result<Vec<String>> {
    let mut stmt = self.conn.prepare("SELECT name FROM tags ORDER BY name ASC")?;
    let rows = stmt.query_map([], |r| r.get(0))?;

    let mut v = Vec::new();
    for r in rows {
      v.push(r?);
    }
    Ok(v)
  }

  /// UPDATE - editar un tag existente
  pub fn update_tag(&self, old_name: &str, new_name: &str) -> Result<()> {
    self.conn.execute("UPDATE tags SET name = ?1 WHERE name = ?2", params![new_name, old_name])?;
    Ok(())
  }

  /// DELETE - eliminar un tag
  pub fn delete_tag(&self, name: &str) -> Result<()> {
    self.conn.execute("DELETE FROM tags WHERE name = ?1", params![name])?;
    Ok(())
  }
}