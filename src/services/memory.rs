use chrono::Utc;
use rusqlite::{params, Connection, Result};

/*
Connection link between the SQLite
database and the program
 */
pub struct MemoryService{
    connection: Connection,
}

impl MemoryService {

    /*
    Created struct and init connection with
    database
     */
    pub fn new(database_path: String) -> Result<Self> {
        let connection = Connection::open(database_path)?;

        /*
        Create table if not exist
         */
        connection.execute("\
            CREATE TABLE IF NOT EXISTS events (\
                id INTEGER PRIMARY KEY AUTOINCREMENT, \
                role TEXT NOT NULL, \
                content TEXT NOT NULL, \
                tool_name TEXT,\
                created_at TIMESTAMP NOT NULL)\
        ", [],)?;

        Ok(Self {connection})
    }

    /*
    Adds an event to the database
     */
    pub fn log_event(&self, role: &str, content: &str, tool_name: Option<&str>) -> Result<()> {
        let query = "INSERT INTO events (role, content, tool_name, created_at) VALUES (?1, ?2, ?3, ?4)";
        self.connection.execute(query, params![role, content, tool_name, Utc::now().to_rfc3339()]);
        Ok(())
    }

    /*
    Retrieve all wind data from the database.
     */
    pub fn get_all(&self) -> Result<Vec<(String, String)>> {
        let query = "SELECT role, content FROM events";
        let mut prepared_statement = self.connection.prepare(query)?;
        let results = prepared_statement.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        Ok(results.filter_map(|res| res.ok()).collect())
    }
}