use rusqlite::Connection;

pub struct SubcategoryStore {
    conn: Connection,
}

impl SubcategoryStore {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub fn init(&self) -> rusqlite::Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS subcategories (
                id INTEGER PRIMARY KEY,
                category TEXT NOT NULL,
                name TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert_subcategory(&self, category: &str, name: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO subcategories (category, name) VALUES (?1, ?2)",
            [category, name],
        )?;
        Ok(())
    }

    pub fn delete_subcategory(&self, category: &str, name: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM subcategories WHERE category = ?1 AND name = ?2",
            [category, name],
        )?;
        Ok(())
    }

    pub fn get_subcategories(&self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT category, name FROM subcategories")?;
        let subcats = stmt
            .query_map([], |row| {
                let category: String = row.get(0)?;
                let name: String = row.get(1)?;
                Ok(format!("{}/{}", category, name))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(subcats)
    }

    pub fn get_subcategories_by_category(&self, category: &str) -> rusqlite::Result<Vec<String>> {
      let mut stmt = self.conn.prepare(
          "SELECT name FROM subcategories WHERE category = ? ORDER BY name"
      )?;
      let rows = stmt.query_map([category], |row| row.get::<_, String>(0))?;

      let mut subcategories = Vec::new();
      for sub in rows {
          subcategories.push(sub?);
      }
      Ok(subcategories)
  }
}
