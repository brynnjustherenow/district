use rusqlite::Connection;
use crate::handlers::District;

pub fn init_db() -> Result<Connection, rusqlite::Error> {
    let conn = Connection::open("districts.db")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS districts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            city TEXT NOT NULL,
            area TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            image TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            UNIQUE(city, area)
        );"
    )?;
    Ok(conn)
}

pub fn list_all(conn: &Connection) -> Result<Vec<District>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, city, area, description, image, created_at, updated_at FROM districts ORDER BY city, area"
    )?;
    let rows = stmt.query_map([], |row| District::from_row(row))?;
    rows.collect()
}

pub fn find_one(conn: &Connection, city: &str, area: &str) -> Result<Option<District>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, city, area, description, image, created_at, updated_at FROM districts WHERE city = ?1 AND area = ?2"
    )?;
    let mut rows = stmt.query(rusqlite::params![city, area])?;
    match rows.next()? {
        Some(row) => Ok(Some(District::from_row(&row)?)),
        None => Ok(None),
    }
}

pub fn insert(conn: &Connection, city: &str, area: &str, description: &str, image: &str) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO districts (city, area, description, image) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![city, area, description, image],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(conn: &Connection, city: &str, area: &str, description: &str, image: &str) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "UPDATE districts SET description = ?1, image = ?2, updated_at = datetime('now', 'localtime') WHERE city = ?3 AND area = ?4",
        rusqlite::params![description, image, city, area],
    )
}

pub fn delete(conn: &Connection, city: &str, area: &str) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "DELETE FROM districts WHERE city = ?1 AND area = ?2",
        rusqlite::params![city, area],
    )
}

pub fn update_image(conn: &Connection, city: &str, area: &str, image: &str) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "UPDATE districts SET image = ?1, updated_at = datetime('now', 'localtime') WHERE city = ?2 AND area = ?3",
        rusqlite::params![image, city, area],
    )
}
