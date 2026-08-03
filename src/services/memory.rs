use rusqlite::{Connection, Result};

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
}